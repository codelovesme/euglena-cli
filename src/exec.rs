use std::path::Path;
use std::path::PathBuf;
use std::process;

use crate::codegen::generate_main_code_file;
use crate::config;

/// Invoke the `code` interpreter with the given subcommand and optional file argument.
///
/// Exits with the interpreter's exit code on failure.
pub fn run_code(command: &str, file: Option<&str>, release: bool) {
    let binary = find_code_binary_or_exit();

    let project_root = project_root_from_file(file);

    // If the project has a manifest.json, generate the entry in a temp dir
    // so it never appears in the project source tree.
    let generated_entry: Option<PathBuf> = if project_root.join("manifest.json").is_file() {
        match generate_main_code_file(&project_root) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("euglena: failed to generate main.code: {}", e);
                process::exit(1);
            }
        }
    } else {
        None
    };

    let mut cmd = process::Command::new(&binary);
    cmd.arg(command);

    // Prefer the generated temp entry; fall back to the user-supplied path.
    match &generated_entry {
        Some(gen_path) => {
            cmd.arg(gen_path);
        }
        None => {
            if let Some(f) = file {
                cmd.arg(resolve_entry_file(f));
            }
        }
    }

    if command == "build" {
        cmd.args(["--target", "exe"]);
        if release {
            cmd.arg("--release");
        }
    }

    // Build CODE_PATH: organelle search dirs + project src/ for gene files.
    let start_dir = discovery_start_dir(file);
    let mut discovered_paths = discover_euglena_organelles_paths(&start_dir);
    if generated_entry.is_some() {
        // Gene files live in src/; the temp entry cannot find them by relative
        // path, so add src/ explicitly so `link foo.gene.code` resolves there.
        let src_dir = project_root.join("src");
        if src_dir.is_dir() {
            discovered_paths.insert(0, src_dir);
        }
    }
    let merged_code_path = merge_code_path_env(discovered_paths);
    if !merged_code_path.is_empty() {
        cmd.env("CODE_PATH", merged_code_path);
    }

    let status = cmd.status().unwrap_or_else(|e| {
        cleanup_generated_entry(&generated_entry);
        eprintln!("euglena: failed to run '{}': {}", binary, e);
        eprintln!("If this path is wrong, update it with:");
        eprintln!("  euglena code set /absolute/path/to/code");
        process::exit(1);
    });

    // Delete temp entry dir regardless of exit code.
    cleanup_generated_entry(&generated_entry);

    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
}

/// Remove the temp directory containing the generated entry file.
fn cleanup_generated_entry(generated: &Option<PathBuf>) {
    if let Some(path) = generated {
        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }
}

fn discovery_start_dir(file: Option<&str>) -> PathBuf {
    if let Some(f) = file {
        let resolved = resolve_entry_file(f);
        let path = Path::new(&resolved);
        let base = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(Path::new(".")).to_path_buf()
        };
        return std::fs::canonicalize(base)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Return the project root for the given entry file (or the current working
/// directory when no file is specified).
///
/// If `file` points to `src/main.code` (or the `src/` directory one level up
/// from the entry file), walk up one extra level to reach the project root
/// that contains `manifest.json`.
fn project_root_from_file(file: Option<&str>) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let base = if let Some(f) = file {
        let resolved = resolve_entry_file(f);
        let path = Path::new(&resolved);
        // If the resolved path is a directory, that IS the project root.
        if path.is_dir() {
            return std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        }
        // Otherwise go up from the file to find the root.
        // For "src/main.code" the parent is "src/", grandparent is the project root.
        match path.parent() {
            Some(p) if p.ends_with("src") => p.parent().unwrap_or(&cwd).to_path_buf(),
            Some(p) => p.to_path_buf(),
            None => cwd.clone(),
        }
    } else {
        cwd.clone()
    };

    std::fs::canonicalize(&base).unwrap_or(base)
}

fn discover_euglena_organelles_paths(start_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for ancestor in start_dir.ancestors() {
        let candidate = ancestor.join("euglena-organelles");
        if candidate.is_dir() {
            paths.push(candidate);
        }
    }
    paths
}

fn merge_code_path_env(discovered_paths: Vec<PathBuf>) -> String {
    let mut ordered: Vec<String> = Vec::new();

    for p in discovered_paths {
        let s = p.to_string_lossy().to_string();
        if !ordered.contains(&s) {
            ordered.push(s);
        }
    }

    if let Ok(existing) = std::env::var("CODE_PATH") {
        for part in existing.split(':').filter(|s| !s.is_empty()) {
            let p = part.to_string();
            if !ordered.contains(&p) {
                ordered.push(p);
            }
        }
    }

    ordered.join(":")
}

fn resolve_entry_file(input: &str) -> String {
    let path = Path::new(input);
    if path.is_dir() {
        return path.join("src/main.code").to_string_lossy().to_string();
    }
    input.to_string()
}

fn find_code_binary_or_exit() -> String {
    if let Some(path) = config::read_code_binary_path() {
        return path.to_string_lossy().to_string();
    }

    eprintln!("euglena: no Code interpreter configured yet.");
    eprintln!("Run this first:");
    eprintln!("  euglena code set /absolute/path/to/code");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  euglena code set /home/you/code-language/target/debug/code");
    process::exit(1);
}
