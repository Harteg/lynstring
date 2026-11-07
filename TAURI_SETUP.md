# Tauri Development Setup & Performance Optimization Guide

This guide covers essential setup and optimization tips to speed up Tauri development builds and improve your development experience.

## 🚀 Quick Start

After setting up these optimizations, you should see:
- **50-70% faster linking** (with lld linker)
- **Faster incremental builds** (with sccache)
- **Parallel compilation** using all CPU cores
- **Reduced file lock conflicts** between rust-analyzer and Tauri

## 📋 Table of Contents

1. [Essential Optimizations](#essential-optimizations)
2. [Cargo Configuration](#cargo-configuration)
3. [Build Profile Settings](#build-profile-settings)
4. [VS Code / rust-analyzer Setup](#vs-code--rust-analyzer-setup)
5. [Compilation Caching](#compilation-caching)
6. [Faster Linker Setup](#faster-linker-setup)
7. [Development Workflow](#development-workflow)
8. [Troubleshooting](#troubleshooting)

---

## Essential Optimizations

### 1. Install sccache (Compilation Caching)

sccache caches compiled Rust code, dramatically speeding up rebuilds when dependencies haven't changed.

```bash
# Install sccache
cargo install sccache

# Add to your shell profile (~/.zshrc or ~/.bashrc)
echo 'export RUSTC_WRAPPER=sccache' >> ~/.zshrc
source ~/.zshrc
```

**Benefits:**
- Caches compiled dependencies across builds
- Speeds up rebuilds when only your code changes
- Works automatically once configured

**Verify it's working:**
```bash
sccache --show-stats
```

### 2. Install Faster Linker (macOS)

The `lld` linker is significantly faster than the default linker, especially for large projects.

```bash
# Install llvm (includes lld)
brew install llvm
```

The linker configuration is already set up in `src-tauri/.cargo/config.toml` - it will be used automatically once llvm is installed.

**Benefits:**
- 50-70% faster linking phase
- Especially noticeable on large projects
- No code changes required

---

## Cargo Configuration

### Project-Level Config (`src-tauri/.cargo/config.toml`)

```toml
[build]
# Cargo automatically uses all available CPU cores by default
# No need to set jobs explicitly - it will auto-detect

# Faster linker for macOS (significantly speeds up linking phase)
[target.x86_64-apple-darwin]
linker = "lld"

[target.aarch64-apple-darwin]
linker = "lld"

[profile.dev]
# Enable incremental compilation for faster rebuilds
incremental = true
```

**Key Settings:**
- **No `jobs` setting**: Cargo automatically uses all available CPU cores by default
- `linker = "lld"`: Uses the faster lld linker (requires llvm installation)
- `incremental = true`: Enables incremental compilation

**Note:** Setting `jobs = 0` in config files is invalid. Cargo auto-detects CPU cores by default, or you can use `-j 0` on the command line.

---

## Build Profile Settings

### Optimized Dev Profile (`src-tauri/Cargo.toml`)

```toml
# Optimize build speed for development
[profile.dev]
incremental = true
opt-level = 0              # No optimization for your code (faster compilation)
debug = true               # Full debug info for your code
debug-assertions = true    # Enable debug assertions
overflow-checks = true     # Enable overflow checks
# Use more codegen units for faster compilation (trade-off: slightly larger binaries)
codegen-units = 256

# Optimize dependencies (slightly faster builds)
[profile.dev.package."*"]
opt-level = 1              # Light optimization for dependencies
debug = false              # No debug info for dependencies
# Disable debug info for dependencies to speed up compilation
split-debuginfo = "unpacked"
```

**What This Does:**
- **Your code**: No optimization, full debug info (fast compilation, easy debugging)
- **Dependencies**: Light optimization, no debug info (faster compilation, smaller binaries)
- **Codegen units**: More units = faster compilation (slightly larger binaries)

**Trade-offs:**
- Faster compilation vs. runtime performance (dev only)
- Faster compilation vs. binary size (dev only)
- Production builds use different profile with full optimization

---

## VS Code / rust-analyzer Setup

### Separate Build Directories

Prevent file lock conflicts between `rust-analyzer` and `tauri dev` by using separate build directories.

**`.vscode/settings.json`:**
```json
{
  "rust-analyzer.cargo.targetDir": "target/analyzer",
  "rust-analyzer.check.command": "cargo",
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints.enable": true
}
```

**Benefits:**
- No file lock conflicts between IDE and Tauri
- Prevents environment variable mismatches
- rust-analyzer can check code while Tauri is running

**How It Works:**
- `tauri dev` uses: `src-tauri/target/debug`
- `rust-analyzer` uses: `target/analyzer`
- Both caches are independent, preventing conflicts

---

## Compilation Caching

### sccache Configuration

sccache automatically caches compilation results. Check cache statistics:

```bash
# Show cache statistics
sccache --show-stats

# Clear cache if needed
sccache --stop-server
```

**Cache Location:**
- macOS: `~/Library/Caches/Mozilla.sccache`
- Linux: `~/.cache/sccache`
- Windows: `%LOCALAPPDATA%\Mozilla\sccache`

**Cache Size:**
- Default: 10 GiB
- Adjust with: `SCCACHE_MAX_CACHE_SIZE=20G`

### When Cache Helps Most

✅ **Cache hits (fast rebuilds):**
- Only your code changed
- Dependencies unchanged
- Same Rust version

❌ **Cache misses (full rebuilds):**
- Dependencies updated
- Rust version changed
- `Cargo.toml` or `Cargo.lock` changed
- `build.rs` changed

---

## Faster Linker Setup

### macOS: lld Linker

The `lld` linker is significantly faster than the default macOS linker.

**Installation:**
```bash
brew install llvm
```

**Configuration:**
Already configured in `src-tauri/.cargo/config.toml`:
```toml
[target.x86_64-apple-darwin]
linker = "lld"

[target.aarch64-apple-darwin]
linker = "lld"
```

**Benefits:**
- 50-70% faster linking phase
- Especially noticeable on large projects
- No code changes required

**Note:** If lld is not installed, Cargo will fall back to the default linker automatically.

### Linux: mold Linker (Alternative)

For Linux users, `mold` is even faster than lld:

```bash
# Install mold
cargo install mold

# Configure in src-tauri/.cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

---

## Development Workflow

### Recommended Commands

```bash
# Start development server (auto-rebuilds on changes)
pnpm tauri:dev

# Clean build artifacts (if needed)
pnpm run tauri:clean

# Production build
pnpm build
```

### First Build vs. Subsequent Builds

**First build (after clean):**
- Compiles all dependencies
- Takes longer (5-10 minutes depending on project)
- Populates sccache

**Subsequent builds:**
- Only recompiles changed code
- Uses cached dependencies
- Much faster (10-30 seconds for code changes)

### When to Clean

Clean build artifacts when:
- ✅ Dependencies updated but build fails
- ✅ Strange compilation errors
- ✅ File lock conflicts persist
- ✅ Switching Rust versions

```bash
pnpm run tauri:clean
```

---

## Troubleshooting

### "Blocking waiting for file lock on build directory"

**Solution 1: Separate build directories**
- Ensure `.vscode/settings.json` has `rust-analyzer.cargo.targetDir: "target/analyzer"`
- Restart VS Code

**Solution 2: Clean and restart**
```bash
pnpm run tauri:clean
# Close VS Code
pnpm tauri:dev
# Restart VS Code
```

**Solution 3: Disable rust-analyzer checking**
- VS Code: Uncheck "rust-analyzer: Check all targets"
- Only do this temporarily while debugging

### Slow Builds Despite Optimizations

**Check:**
1. Is sccache running?
   ```bash
   sccache --show-stats
   ```

2. Are you using all CPU cores?
   - Cargo automatically uses all cores by default
   - You can verify with `cargo build -j` (shows number of jobs)

3. Is lld installed? (macOS)
   ```bash
   which lld
   ```

4. Are dependencies being recompiled unnecessarily?
   - Check if `Cargo.toml` or `build.rs` changed
   - Check if Rust version changed

### sccache Not Working

**Verify:**
```bash
# Check if RUSTC_WRAPPER is set
echo $RUSTC_WRAPPER

# Should output: sccache
# If not, add to ~/.zshrc and reload:
source ~/.zshrc
```

**Check cache stats:**
```bash
sccache --show-stats
```

### Linker Errors (lld)

If you get linker errors with lld:

1. **Verify llvm is installed:**
   ```bash
   brew list llvm
   ```

2. **Check lld path:**
   ```bash
   which lld
   # Should be: /opt/homebrew/opt/llvm/bin/lld (Apple Silicon)
   # Or: /usr/local/opt/llvm/bin/lld (Intel)
   ```

3. **Fallback to default linker:**
   - Comment out linker settings in `src-tauri/.cargo/config.toml`
   - Cargo will use default linker

---

## Performance Benchmarks

Based on typical Tauri projects:

| Optimization | Build Time Reduction |
|-------------|---------------------|
| Incremental compilation | 60-80% faster rebuilds |
| sccache | 50-70% faster (when cache hits) |
| lld linker | 50-70% faster linking |
| Parallel jobs (all cores) | 2-4x faster (depends on CPU) |
| Optimized dependency profile | 10-20% faster |

**Combined effect:** Typical rebuilds go from 60+ seconds to 10-15 seconds.

---

## Additional Tips

### 1. Minimize Dependencies

- Only include dependencies you actually use
- Use feature flags to exclude unused features
- Consider lighter alternatives for heavy dependencies

### 2. Avoid Changing build.rs Frequently

- `build.rs` changes trigger full rebuilds
- Cache build script results when possible
- Only include necessary build-time code

### 3. Use Workspace Features Wisely

- Feature flags can cause more recompilation
- Group related features together
- Avoid feature combinations that require multiple dependency versions

### 4. Keep Rust Up to Date

- Newer Rust versions often have performance improvements
- Use `rustup update` regularly
- Test with stable, beta, and nightly (if needed)

### 5. Monitor Build Times

```bash
# Time a build
time cargo build

# Show detailed build times
cargo build --timings
```

### 6. Use `cargo check` for Quick Validation

```bash
# Fast syntax/type checking (no linking)
cargo check

# Much faster than full build
# Good for quick validation during development
```

---

## References

- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [sccache GitHub](https://github.com/mozilla/sccache)
- [Tauri Documentation](https://tauri.app/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Yuexun's Tauri Speed Guide](https://yuexun.me/how-to-make-your-tauri-dev-faster/)

---

## Summary Checklist

- [ ] Install sccache: `cargo install sccache`
- [ ] Add `RUSTC_WRAPPER=sccache` to shell profile
- [ ] Install llvm (for lld linker): `brew install llvm`
- [ ] Configure `src-tauri/.cargo/config.toml` with parallel jobs and linker
- [ ] Configure `Cargo.toml` with optimized dev profile
- [ ] Set up `.vscode/settings.json` with separate build directory
- [ ] Reload shell: `source ~/.zshrc`
- [ ] Test: `pnpm tauri:dev` and verify faster builds

---

**Happy coding! 🚀**

