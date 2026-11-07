<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { Project } from "../../../../src-tauri/bindings/Project";
  import type { AllStringsData } from "../../../../src-tauri/bindings/AllStringsData";
  import { currentProject, editingProject } from '$lib/stores/project';
  import { showNewProjectDialog } from '$lib/stores/dialog';
  import { projectDataStore } from '$lib/stores/project-data';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import AlertCircle from "@lucide/svelte/icons/alert-circle";
  import Check from "@lucide/svelte/icons/check";
  // MVP: API key section commented out - users cannot use their own API key in MVP
  // TODO: Uncomment when allowing users to provide their own API keys
  // import ApiKeySection from "$lib/components/ApiKeySection.svelte";
  
  export let isOpen = false;
  
  let projectName = "";
  let projectPath = "";
  let contextPrompt = "";
  // MVP: API key variables commented out - not used in MVP but keep for future use
  // TODO: Uncomment when allowing users to provide their own API keys
  // let apiKey = "";
  let protectedNamesText = "";
  let isLoading = false;
  let isDeleting = false;
  let error = "";
  let showDeleteConfirm = false;
  let deleteConfirmTimeout: any = null;
  // MVP: API key tracking variables commented out - not used in MVP but keep for future use
  // TODO: Uncomment when allowing users to provide their own API keys
  // let originalApiKey = "";
  // let apiKeyTested = false;
  let projectScanSummary: AllStringsData | null = null;
  let isScanning = false;
  let scanError = "";
  
  $: isEditMode = $editingProject !== null;
  $: dialogTitle = isEditMode ? "Edit Project" : "Add New Project";

  $: stringCount = projectScanSummary
    ? (projectScanSummary.strings_by_language['default']?.length || 0) + (projectScanSummary.plurals_by_language['default']?.reduce((s, p) => s + p.items.length, 0) || 0)
    : 0;
  $: submitButtonText = isEditMode ? "Save Changes" : "Add Project";
  
  // Reset form when dialog opens/closes or when editingProject changes
  $: if (isOpen) {
    resetForm();
  }
  
  // Also reset when editingProject becomes null (new project)
  $: if (isOpen && $editingProject === null) {
    resetForm();
  }
  
  async function resetForm() {
    if ($editingProject) {
      // Populate form with existing project data
      // Truncate project name to 30 characters if longer
      const name = $editingProject.name;
      projectName = name.length > 30 ? name.substring(0, 30) : name;
      projectPath = $editingProject.path;
      contextPrompt = $editingProject.context_prompt || "";
      // MVP: API key assignment commented out - not used in MVP but keep for future use
      // TODO: Uncomment when allowing users to provide their own API keys
      // apiKey = $editingProject.api_key || "";
      // originalApiKey = $editingProject.api_key || "";
      protectedNamesText = $editingProject.protected_names.join(", ");
      // apiKeyTested = true; // Assume existing API key was tested
    } else {
      // Reset form for new project
      projectName = "";
      projectPath = "";
      contextPrompt = "";
      // MVP: API key reset commented out - not used in MVP but keep for future use
      // TODO: Uncomment when allowing users to provide their own API keys
      // apiKey = "";
      // originalApiKey = "";
      protectedNamesText = "";
      // apiKeyTested = false;
    }
    error = "";
    showDeleteConfirm = false;
    projectScanSummary = null;
    isScanning = false;
    scanError = "";
    
    // Auto-scan project if path is not empty
    if (projectPath.trim()) {
      await scanProject(projectPath);
    }
  }

  async function pickFolder() {
    try {
      const selectedPath = await invoke<string | null>("pick_project_folder");
      if (selectedPath) {
        projectPath = selectedPath;
        // Auto-fill project name from folder name if empty and not editing
        if (!projectName && !isEditMode) {
          const folderName = selectedPath.split(/[/\\]/).pop() || "";
          // Truncate to 30 characters if longer
          projectName = folderName.length > 30 ? folderName.substring(0, 30) : folderName;
        }
        
        // Scan the project to get string and language counts
        await scanProject(selectedPath);
      }
    } catch (err) {
      error = `Failed to pick folder: ${err}`;
    }
  }
  
  async function scanProject(path: string) {
    isScanning = true;
    scanError = "";
    projectScanSummary = null;
    
    try {
      const summary = await invoke<AllStringsData>("load_project_data", { projectPath: path });
      projectScanSummary = summary;
    } catch (err) {
      scanError = `Failed to scan project: ${err}`;
    } finally {
      isScanning = false;
    }
  }
  
  // Validation logic
  // MVP: API key validation commented out - API key not required in MVP but keep for future use
  // TODO: Uncomment when allowing users to provide their own API keys
  // $: apiKeyChanged = isEditMode && apiKey !== originalApiKey;
  // $: requiresApiKeyTest = !isEditMode || apiKeyChanged;
  // MVP: Form validation updated to not require API key
  $: isFormValid = projectName.trim() && projectPath.trim(); // Removed: && apiKey.trim() && (!requiresApiKeyTest || apiKeyTested)

  async function saveProject() {
    if (!projectName.trim() || !projectPath.trim()) {
      error = "Please fill in all required fields";
      return;
    }
    
    // MVP: API key validation commented out - API key not required in MVP but keep for future use
    // TODO: Uncomment when allowing users to provide their own API keys
    // if (!apiKey.trim()) {
    //   error = "API key is required";
    //   return;
    // }
    // 
    // if (requiresApiKeyTest && !apiKeyTested) {
    //   error = "Please test your API key before saving";
    //   return;
    // }
    
    isLoading = true;
    error = "";
    
    try {
      const protectedNames = protectedNamesText
        .split(',')
        .map(name => name.trim())
        .filter(name => name.length > 0);
      
      let project: Project;
      
      if (isEditMode) {
        // Update existing project
        const trimmedName = projectName.trim();
        project = {
          ...$editingProject!,
          name: trimmedName.length > 30 ? trimmedName.substring(0, 30) : trimmedName,
          path: projectPath.trim(),
          context_prompt: contextPrompt.trim(),
          // MVP: API key assignment commented out - not used in MVP but keep for future use
          // TODO: Uncomment when allowing users to provide their own API keys
          // api_key: apiKey.trim(),
          protected_names: protectedNames
        };
      } else {
        // Add new project
        const trimmedName = projectName.trim();
        project = await invoke<Project>("create_project", {
          name: trimmedName.length > 30 ? trimmedName.substring(0, 30) : trimmedName,
          path: projectPath.trim(),
          contextPrompt: contextPrompt.trim(),
          // MVP: API key parameter commented out - not used in MVP but keep for future use
          // TODO: Uncomment when allowing users to provide their own API keys
          // apiKey: apiKey.trim(),
          protectedNames
        });
        
        // For new projects, select all available languages by default
        let availableLanguages: string[] = [];
        
        if (projectScanSummary && projectScanSummary.languages.length > 0) {
          // Use languages from scan summary, filtering out "default"
          availableLanguages = projectScanSummary.languages.filter(
            lang => lang !== "default"
          );
        } else {
          // Fallback: detect languages if scan wasn't performed
          try {
            const detectedLanguages = await invoke<string[]>("detect_available_languages", {
              projectPath: projectPath.trim()
            });
            availableLanguages = detectedLanguages;
          } catch (err) {
            console.warn("Failed to detect languages for new project:", err);
            // Continue with empty languages if detection fails
          }
        }
        
        if (availableLanguages.length > 0) {
          project.selected_languages = availableLanguages;
        }
      }
      
      // Save the project to disk
      await invoke("save_project", { project });
      
      if (isEditMode) {
        currentProject.set(project);  // Update current project if we're editing it
        // Refresh project data to reflect any changes (like new path) - non-blocking
        projectDataStore.loadProjectData(project.path);
      } else {
        // Set as current project only for new projects
        await invoke("set_current_project", { projectId: project.id });
        currentProject.set(project);
      }
      
      // Clear editing state
      editingProject.set(null);
      closeDialog();
    } catch (err) {
      error = `Failed to ${isEditMode ? 'update' : 'create'} project: ${err}`;
    } finally {
      isLoading = false;
    }
  }
  
  async function deleteProject() {
    if (!$editingProject) return;
    
    isDeleting = true;
    error = "";
    
    try {
      await invoke("delete_project", { projectId: $editingProject.id });
      
      // If we're deleting the current project, select another one
      if ($currentProject && $currentProject.id === $editingProject.id) {
        // Load remaining projects and select the first one
        const remainingProjects = await invoke<Project[]>("load_projects");
        if (remainingProjects.length > 0) {
          const firstProject = remainingProjects[0];
          await invoke("set_current_project", { projectId: firstProject.id });
          currentProject.set(firstProject);
        } else {
          // No projects left, clear current project
          currentProject.set(null);
        }
      }
      
      // Clear editing state and confirmation
      editingProject.set(null);
      showDeleteConfirm = false;
      if (deleteConfirmTimeout !== null) {
        window.clearTimeout(deleteConfirmTimeout);
        deleteConfirmTimeout = null;
      }
      closeDialog();
    } catch (err) {
      error = `Failed to delete project: ${err}`;
    } finally {
      isDeleting = false;
    }
  }
  
  function showDeleteConfirmation() {
    showDeleteConfirm = true;
    
    // Clear existing timeout
    if (deleteConfirmTimeout !== null) {
      window.clearTimeout(deleteConfirmTimeout);
    }
    
    // Reset after 3 seconds
    deleteConfirmTimeout = window.setTimeout(() => {
      showDeleteConfirm = false;
      deleteConfirmTimeout = null;
    }, 3000);
  }
  
  // MVP: API key test handler commented out - not used in MVP but keep for future use
  // TODO: Uncomment when allowing users to provide their own API keys
  // function handleApiKeyTestResult(success: boolean) {
  //   apiKeyTested = success;
  // }

  function closeDialog() {
    showNewProjectDialog.set(false);
    editingProject.set(null);
    // Reset form will be called by the reactive statement when isOpen changes
  }
</script>

<Dialog.Root bind:open={isOpen}>
  <Dialog.Content class="dialog-standard sm:max-w-[520px]">
    <Dialog.Header class="dialog-standard-header">
      <Dialog.Title class="dialog-standard-title">{dialogTitle}</Dialog.Title>
    </Dialog.Header>

    <form on:submit|preventDefault={saveProject} class="space-y-6">
      <div class="space-y-3">
        <label for="project-name" class="text-sm font-medium text-foreground">Project Name *</label>
        <input
          id="project-name"
          type="text"
          bind:value={projectName}
          placeholder="My Android App"
          required
          maxlength="30"
          class="w-full h-10 px-4 bg-background/50 border border-border rounded-lg text-sm placeholder:text-muted-foreground focus:outline-none transition-all duration-200"
        />
      </div>

      <!-- TODO: allow manual input of path. We just need to automatically detect the res folder on the fly.  -->
      <div class="space-y-3">
        <label for="project-path" class="text-sm font-medium text-foreground">Project Path *</label>
        <p class="text-xs text-muted-foreground">Select root of your Android project.</p>
        <div class="flex gap-2">
          <input
            id="project-path"
            type="text"
            bind:value={projectPath}
            placeholder="/path/to/android/project"
            required
            disabled
            class="flex-1 h-10 px-4 bg-background/30 border border-border rounded-lg text-sm placeholder:text-muted-foreground focus:outline-none transition-all duration-200 cursor-not-allowed opacity-60"
          />
          <Button type="button" variant="outline" onclick={pickFolder} class="h-10 px-4">
            Browse
          </Button>
        </div>
        
        <!-- Project Scan Results -->
        {#if isScanning}
          <div class="mt-3 p-3 bg-muted/30 border border-border rounded-lg">
            <div class="flex items-center gap-2 text-sm text-muted-foreground">
              <div class="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
              Scanning project for strings and languages...
            </div>
          </div>
        {/if}
        
        {#if scanError}
          <div class="mt-3 p-3 bg-destructive/10 border border-destructive/20 text-destructive rounded-lg">
            <div class="flex items-center gap-2 text-sm">
              <AlertCircle class="w-4 h-4 flex-shrink-0" />
              This doesn't look like an Android project. Failed to find strings.xml file.
            </div>
          </div>
        {/if}
        
        {#if projectScanSummary}
          <div class="mt-3 p-3 bg-background/50 border border-border rounded-lg">
            <div class="flex items-center gap-3 text-sm text-foreground">
              <div class="rounded-full bg-primary/10 p-1">
                <Check class="w-4 h-4 text-primary flex-shrink-0" />
              </div>
              <span>Found <span class="font-extrabold text-primary">{stringCount}</span> {stringCount === 1 ? 'string' : 'strings'} in <span class="font-extrabold text-primary">{projectScanSummary.languages.length}</span> {projectScanSummary.languages.length === 1 ? 'language' : 'languages'}</span>
            </div>
          </div>
          
          {#if stringCount > 1000}
            <div class="mt-3 p-3 bg-destructive/10 border border-destructive/20 text-destructive rounded-lg">
              <div class="flex items-center gap-2 text-sm">
                <AlertCircle class="w-4 h-4 flex-shrink-0" />
                <span>The UI is not optimized yet for projects with more than 1000 strings. Performance may be affected.</span>
              </div>
            </div>
          {/if}

        {/if}
      </div>

      <div class="space-y-3">
        <label for="context-prompt" class="text-sm font-medium text-foreground">Translation context (optional)</label>
        <textarea
          id="context-prompt"
          bind:value={contextPrompt}
          placeholder="E.g. This is a travel app for booking hotels and flights"
          rows="3"
          class="w-full h-24 px-4 py-3 bg-background/50 border border-border rounded-lg text-sm placeholder:text-muted-foreground focus:outline-none transition-all duration-200 resize-none"
        ></textarea>
      </div>

      <div class="space-y-3">
        <label for="protected-names" class="text-sm font-medium text-foreground">Protected Names</label>
        <input
          id="protected-names"
          type="text"
          bind:value={protectedNamesText}
          placeholder="e.g. app_name, company_name"
          class="w-full h-10 px-4 bg-background/50 border border-border rounded-lg text-sm placeholder:text-muted-foreground focus:outline-none transition-all duration-200"
        />
        <p class="text-xs text-muted-foreground">String names that should not be translated</p>
      </div>

      <!-- MVP: API Key Section commented out - users cannot use their own API key in MVP -->
      <!-- TODO: Uncomment when allowing users to provide their own API keys -->
      <!-- This section should NOT be removed as it will be used later -->
      <!-- <ApiKeySection bind:apiKey onTestResult={handleApiKeyTestResult} /> -->

      {#if error}
        <div class="p-4 text-sm bg-destructive/10 border border-destructive/20 text-destructive rounded-lg">
          <div class="flex items-center gap-2">
            <AlertCircle class="w-4 h-4 flex-shrink-0" />
            {error}
          </div>
        </div>
      {/if}

      <Dialog.Footer class="dialog-standard-footer">
        <div class="flex justify-between w-full">
          <div>
            {#if isEditMode}
              <Button
                type="button"
                variant={showDeleteConfirm ? "destructive" : "ghost"}
                onclick={showDeleteConfirm ? deleteProject : showDeleteConfirmation}
                disabled={isLoading || isDeleting}
                class="px-4 py-2"
              >
                {showDeleteConfirm ? "Confirm Removal" : "Remove project"}
              </Button>
            {/if}
          </div>
          <div class="flex gap-2">
            <Button
              variant="ghost"
              onclick={closeDialog}
              class="px-6 py-2"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={isLoading || isDeleting || !isFormValid}
              class="px-6 py-2"
            >
              {isLoading ? (isEditMode ? "Saving..." : "Creating...") : submitButtonText}
            </Button>
          </div>
        </div>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>

