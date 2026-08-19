use std::fs;
use std::path::{Path, PathBuf};

const APP_DIR: &str = ".config/euglena-cli";
const CODE_PATH_FILE: &str = "code_binary_path";

pub fn set_code_binary_path(path: &str) {
    let path_obj = Path::new(path);
    if !path_obj.is_file() {
        eprintln!("euglena: '{}' is not a file", path);
        std::process::exit(1);
    }

    let canonical = fs::canonicalize(path_obj).unwrap_or_else(|e| {
        eprintln!("euglena: failed to resolve '{}': {}", path, e);
        std::process::exit(1);
    });

    let cfg_file = config_file_path();
    if let Some(parent) = cfg_file.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!(
                "euglena: failed to create config directory '{}': {}",
                parent.display(),
                e
            );
            std::process::exit(1);
        });
    }

    fs::write(&cfg_file, canonical.to_string_lossy().as_ref()).unwrap_or_else(|e| {
        eprintln!(
            "euglena: failed to write config '{}': {}",
            cfg_file.display(),
            e
        );
        std::process::exit(1);
    });

    println!("Configured Code interpreter:");
    println!("  {}", canonical.display());
}

pub fn show_code_binary_path() {
    match read_code_binary_path() {
        Some(path) => {
            println!("Code interpreter:");
            println!("  {}", path.display());
        }
        None => {
            println!("No Code interpreter configured — euglena will use `cdlvsm-code`");
            println!("from your PATH when you run an app.");
            println!("To use a `code` binary not installed via cdlvsm, set it explicitly:");
            println!("  euglena code set /absolute/path/to/code");
        }
    }
}

pub fn clear_code_binary_path() {
    let cfg_file = config_file_path();
    if cfg_file.exists() {
        fs::remove_file(&cfg_file).unwrap_or_else(|e| {
            eprintln!(
                "euglena: failed to remove config '{}': {}",
                cfg_file.display(),
                e
            );
            std::process::exit(1);
        });
        println!("Cleared Code interpreter path.");
    } else {
        println!("No Code interpreter path was configured.");
    }
}

pub fn read_code_binary_path() -> Option<PathBuf> {
    let cfg_file = config_file_path();
    let content = fs::read_to_string(cfg_file).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

fn config_file_path() -> PathBuf {
    home_dir().join(APP_DIR).join(CODE_PATH_FILE)
}

fn home_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home);
    }

    eprintln!("euglena: HOME is not set; cannot resolve config path");
    std::process::exit(1);
}
