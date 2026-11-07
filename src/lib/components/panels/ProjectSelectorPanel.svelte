<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { Project } from "../../../../src-tauri/bindings/Project";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import { toast } from "svelte-sonner";
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  
  import { currentProject, editingProject } from '$lib/stores/project';
  import { showNewProjectDialog } from '$lib/stores/dialog';
  
  let projects: Project[] = $state([]);
  let isLoading = $state(false);
  let dropdownOpen = $state(false);
  let wasDialogOpen = $state(false);
  
  // Reload projects when dialog closes (in case a project was deleted or updated)
  $effect(() => {
    const dialogJustClosed = wasDialogOpen && !$showNewProjectDialog;
    wasDialogOpen = $showNewProjectDialog;
    
    if (dialogJustClosed) {
      loadProjects();
    }
  });

  // Load projects when dropdown opens
  $effect(() => {
    if (dropdownOpen) {
      loadProjects();
    }
  });
  
  async function loadProjects() {
    try {
      isLoading = true;
      projects = await invoke<Project[]>("load_projects");
    } catch (err) {
      toast.error("Failed to load projects", {
        description: String(err)
      });
      console.error("Failed to load projects:", err);
    } finally {
      isLoading = false;
    }
  }
  
  async function selectProject(project: Project) {
    if (!isProjectValid(project)) {
      toast.error("Cannot select project", {
        description: "Missing required fields. Please edit the project to fix this."
      });
      dropdownOpen = false;
      return;
    }
    try {
      await invoke("set_current_project", { projectId: project.id });
      currentProject.set(project);
      dropdownOpen = false;
    } catch (err) {
      toast.error("Failed to select project", {
        description: String(err)
      });
    }
  }
  
  function editProject(project: Project, event: Event) {
    event.stopPropagation();
    editingProject.set(project);
    showNewProjectDialog.set(true);
    dropdownOpen = false;
  }
  
  function handleNewProject() {
    editingProject.set(null); // Clear any editing state
    showNewProjectDialog.set(true);
    dropdownOpen = false;
  }
  
  function getProjectDisplayName(project: Project): string {
    return project.name || project.path.split(/[/\\]/).pop() || "Unnamed Project";
  }
  
  function isProjectValid(project: Project): boolean {
    // MVP: API key validation removed - API key not required in MVP but keep for future use
    // TODO: Add back API key requirement when allowing users to provide their own API keys
    // Original: return !!(project.id && project.name && project.path && project.api_key);
    return !!(project.id && project.name && project.path); // Removed: && project.api_key
  }
</script>

<DropdownMenu.Root bind:open={dropdownOpen}>
  <DropdownMenu.Trigger 
    class="project-trigger rounded-lg border bg-card hover:bg-muted text-card-foreground shadow-sm transition-colors" 
    data-open={dropdownOpen}
  >
    <div class="project-trigger-content">
      <FolderOpenIcon class="w-5 h-5 text-primary" />
      <span class="project-name text-base font-semibold">
        {#if $currentProject}
          {getProjectDisplayName($currentProject)}
        {:else}
          Select Project
        {/if}
      </span>
      <ChevronDownIcon class={`w-5 h-5 text-muted-foreground transition-transform ${dropdownOpen ? 'rotate-180' : ''}`} />
    </div>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-80 max-h-[400px] overflow-y-auto">
    <DropdownMenu.Label class="flex items-center gap-2">
      <FolderOpenIcon class="h-5 w-5" />
      Projects
    </DropdownMenu.Label>

    <DropdownMenu.Separator />
    {#if isLoading}
      <div class="px-2 py-4 text-center text-sm text-muted-foreground">
        Loading projects...
      </div>
    {:else if projects.length === 0}
      <div class="px-2 py-4 text-center text-sm text-muted-foreground">
        Add your first project!
      </div>
    {:else}
      <div class="space-y-1">
        {#each projects as project}
          <div class="flex items-center gap-1 group/row min-w-0">
            <DropdownMenu.Item
              onclick={() => selectProject(project)}
              class="flex-1 min-w-0 {$currentProject?.id === project.id ? 'bg-accent' : ''}"
            >
              <div class="flex items-center gap-2 flex-1 min-w-0">
                <div class="font-medium text-sm truncate min-w-0">
                  {getProjectDisplayName(project)}
                </div>
                {#if !isProjectValid(project)}
                  <span class="text-xs text-destructive bg-destructive/10 px-1.5 py-0.5 rounded flex items-center gap-1 flex-shrink-0">
                    <AlertCircleIcon class="h-3 w-3" />
                    ERROR
                  </span>
                {/if}
              </div>
            </DropdownMenu.Item>
            <DropdownMenu.Item
              onclick={(e) => editProject(project, e)}
              class="p-1.5 w-8 h-8 opacity-0 group-hover/row:opacity-100 flex-shrink-0 flex items-center justify-center"
              title="Edit project"
            >
              <SettingsIcon class="h-4 w-4" />
            </DropdownMenu.Item>
          </div>
        {/each}
      </div>
    {/if}

    <DropdownMenu.Separator />

    <DropdownMenu.Item
      onclick={handleNewProject}
      class="font-medium"
    >
      <PlusIcon class="h-4 w-4" />
      Add Project
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>

<style>
  :global([data-slot="dropdown-menu-trigger"].project-trigger) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 2.75rem;
    padding: 0;
    cursor: pointer;
    max-width: 350px;
    min-width: 220px;
  }
  
  :global([data-slot="dropdown-menu-trigger"].project-trigger:focus-visible) {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }

  .project-trigger-content {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0 1rem;
    width: 100%;
    min-width: 0;
  }

  .project-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
