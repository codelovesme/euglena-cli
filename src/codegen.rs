use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest::{parse_manifest, AppManifest};

/// Scan `src_dir` for `*.gene.code` files and return their names sorted
/// alphabetically (e.g. `["data_fetcher.gene.code", "state_loader.gene.code"]`).
pub fn scan_genes(src_dir: &Path) -> Vec<String> {
    let mut genes: Vec<String> = Vec::new();

    let entries = match fs::read_dir(src_dir) {
        Ok(e) => e,
        Err(_) => return genes,
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".gene.code") && entry.path().is_file() {
            genes.push(name_str.to_string());
        }
    }

    genes.sort();
    genes
}

/// Generate the content of `src/main.code` from a manifest and a list of gene
/// filenames (as returned by `scan_genes`).
pub fn generate_main_code(manifest: &AppManifest, genes: &[String]) -> String {
    let mut lines = Vec::new();

    lines.push(
        "-> GENERATED — do not edit. Modify manifest.json or src/*.gene.code instead.".to_string(),
    );
    lines.push(String::new());

    // Organelle link statements — always aliased.
    for (alias, entry) in &manifest.organelles {
        lines.push(format!("link {} as {}", entry.path(), alias));
    }

    if !manifest.organelles.is_empty() {
        lines.push(String::new());
    }

    // Gene link statements.
    for gene in genes {
        lines.push(format!("link {}", gene));
    }

    if !genes.is_empty() {
        lines.push(String::new());
    }

    // Sap particles — collected from Full organelle entries.
    // Each Sap now handles both configuration and readiness — the response
    // is captured so the gene can detect Alive / Exception.
    let mut has_sap = false;
    for (alias, entry) in &manifest.organelles {
        if let Some(fields) = entry.sap() {
            lines.push(generate_sap_line(alias, fields));
            has_sap = true;
        }
    }

    if has_sap {
        lines.push(String::new());
    }

    // Boot particle.
    lines.push(format!(
        "emit EuglenaHasBeenBorn {{ cell_name = \"{}\" }} to this",
        manifest.name
    ));
    lines.push(String::new());

    lines.join("\n")
}

fn format_sap_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => {
            format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_sap_value).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(map) => {
            // Anonymous object literal: `{ k = v, ... }`. The receiving Sap
            // field must be declared as `Any` for codegen to accept it.
            let parts: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{} = {}", k, format_sap_value(v)))
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
        serde_json::Value::Null => "Null".to_string(),
    }
}

fn generate_sap_line(target: &str, fields: &serde_json::Map<String, serde_json::Value>) -> String {
    if fields.is_empty() {
        format!("emit {}.Sap {{}} to {} get _sap_{}", target, target, target)
    } else {
        let field_str: Vec<String> = fields
            .iter()
            .map(|(k, v)| format!("{} = {}", k, format_sap_value(v)))
            .collect();
        format!(
            "emit {}.Sap {{ {} }} to {} get _sap_{}",
            target,
            field_str.join(", "),
            target,
            target
        )
    }
}

/// Generate a `main.code` entry file for `project_root` and write it to a
/// temporary directory (`/tmp/euglena_<pid>/main.code`).  The file is named
/// `main.code` so the compiler always outputs `main.wasm` (stem is preserved).
///
/// The caller must delete the returned path's **parent directory** when done:
/// `fs::remove_dir_all(path.parent().unwrap())`.
pub fn generate_main_code_file(project_root: &Path) -> Result<PathBuf, String> {
    let manifest_path = project_root.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(format!(
            "No manifest.json found in '{}'",
            project_root.display()
        ));
    }

    let manifest = parse_manifest(&manifest_path)?;

    let src_dir = project_root.join("src");
    let genes = scan_genes(&src_dir);

    let content = generate_main_code(&manifest, &genes);

    // Place entry in a PID-scoped temp dir so it is always named `main.code`.
    let tmp_dir = std::env::temp_dir().join(format!("euglena_{}", std::process::id()));
    fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("Cannot create temp dir '{}': {}", tmp_dir.display(), e))?;

    let entry_path = tmp_dir.join("main.code");
    fs::write(&entry_path, &content)
        .map_err(|e| format!("Cannot write temp entry '{}': {}", entry_path.display(), e))?;

    Ok(entry_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{AppManifest, OrganelleEntry};
    use std::collections::BTreeMap;

    fn make_manifest(name: &str, organelles: &[(&str, &str)]) -> AppManifest {
        let mut map = BTreeMap::new();
        for (alias, path) in organelles {
            map.insert(alias.to_string(), OrganelleEntry::Path(path.to_string()));
        }
        AppManifest {
            name: name.to_string(),
            organelles: map,
        }
    }

    #[test]
    fn generate_no_organelles_no_genes() {
        let m = make_manifest("my-app", &[]);
        let content = generate_main_code(&m, &[]);
        assert!(content.contains("emit EuglenaHasBeenBorn { cell_name = \"my-app\" } to this"));
        assert!(!content.contains("link"));
    }

    #[test]
    fn generate_organelles_and_genes() {
        let m = make_manifest(
            "todo",
            &[
                ("react", "organelles/react.wasm"),
                ("storage", "organelles/storage.wasm"),
            ],
        );
        let genes = vec![
            "state_loader.gene.code".to_string(),
            "todo_actions.gene.code".to_string(),
        ];
        let content = generate_main_code(&m, &genes);
        assert!(content.contains("link organelles/react.wasm as react"));
        assert!(content.contains("link organelles/storage.wasm as storage"));
        assert!(content.contains("link state_loader.gene.code"));
        assert!(content.contains("link todo_actions.gene.code"));
        assert!(content.contains("emit EuglenaHasBeenBorn { cell_name = \"todo\" } to this"));
    }

    #[test]
    fn generate_header_marks_as_generated() {
        let m = make_manifest("x", &[]);
        let content = generate_main_code(&m, &[]);
        assert!(content.starts_with("-> GENERATED"));
    }

    #[test]
    fn generate_full_organelle_with_sap() {
        let mut fields = serde_json::Map::new();
        fields.insert(
            "base_path".to_string(),
            serde_json::Value::String(".".to_string()),
        );
        let mut map = BTreeMap::new();
        map.insert(
            "fs".to_string(),
            OrganelleEntry::Full {
                path: "organelles/fs.so".to_string(),
                sap: fields,
            },
        );
        let m = AppManifest {
            name: "myapp".to_string(),
            organelles: map,
        };
        let content = generate_main_code(&m, &[]);
        assert!(content.contains("link organelles/fs.so as fs"));
        assert!(content.contains("emit fs.Sap { base_path = \".\" } to fs"));
    }

    #[test]
    fn generate_full_organelle_empty_sap() {
        let mut map = BTreeMap::new();
        map.insert(
            "process".to_string(),
            OrganelleEntry::Full {
                path: "organelles/process.so".to_string(),
                sap: serde_json::Map::new(),
            },
        );
        let m = AppManifest {
            name: "myapp".to_string(),
            organelles: map,
        };
        let content = generate_main_code(&m, &[]);
        assert!(content.contains("link organelles/process.so as process"));
        assert!(content.contains("emit process.Sap {} to process"));
    }

    #[test]
    fn generate_mixed_path_and_full() {
        let mut fields = serde_json::Map::new();
        fields.insert(
            "port".to_string(),
            serde_json::Value::Number(serde_json::Number::from(9800)),
        );
        let mut map = BTreeMap::new();
        map.insert(
            "logger".to_string(),
            OrganelleEntry::Path("organelles/console.so".to_string()),
        );
        map.insert(
            "server".to_string(),
            OrganelleEntry::Full {
                path: "organelles/server.so".to_string(),
                sap: fields,
            },
        );
        let m = AppManifest {
            name: "myapp".to_string(),
            organelles: map,
        };
        let content = generate_main_code(&m, &[]);
        assert!(content.contains("link organelles/console.so as logger"));
        assert!(content.contains("link organelles/server.so as server"));
        assert!(content.contains("emit server.Sap { port = 9800 } to server"));
        assert!(!content.contains("logger.Sap"));
    }

    #[test]
    fn generate_path_organelle_no_sap_emitted() {
        let m = make_manifest("myapp", &[("logger", "organelles/console.so")]);
        let content = generate_main_code(&m, &[]);
        assert!(content.contains("link organelles/console.so as logger"));
        assert!(!content.contains("Sap"));
    }
}
