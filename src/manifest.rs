use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::BufRead;
use std::path::Path;

/// A single organelle entry — either a plain path string or an object with
/// a path and optional Sap particle configuration.
pub enum OrganelleEntry {
    /// Simple form: `"react": "organelles/react.wasm"`.
    Path(String),
    /// Full form: `"server": { "path": "organelles/server.so", "sap": { ... } }`.
    Full {
        path: String,
        sap: serde_json::Map<String, serde_json::Value>,
    },
}

impl OrganelleEntry {
    /// The organelle's import path.
    pub fn path(&self) -> &str {
        match self {
            OrganelleEntry::Path(p) => p,
            OrganelleEntry::Full { path, .. } => path,
        }
    }

    /// Sap configuration, if any. Returns `None` for the simple `Path` variant.
    pub fn sap(&self) -> Option<&serde_json::Map<String, serde_json::Value>> {
        match self {
            OrganelleEntry::Path(_) => None,
            OrganelleEntry::Full { sap, .. } => Some(sap),
        }
    }
}

/// Parsed subset of a project's manifest.json.
pub struct AppManifest {
    /// Cell name — taken from the `"name"` field.
    pub name: String,
    /// Organelle alias → entry (path string or `{ path, sap }` object).
    pub organelles: BTreeMap<String, OrganelleEntry>,
}

/// Parse the manifest.json at `path` and return an `AppManifest`.
///
/// Before parsing, any `.env` file next to the manifest is loaded into the
/// process environment.  String values in `sap` objects are then subject to
/// `${VAR}` interpolation so secrets stay out of version control.
pub fn parse_manifest(path: &Path) -> Result<AppManifest, String> {
    // Load .env from the manifest directory (if present).
    if let Some(dir) = path.parent() {
        load_dotenv(&dir.join(".env"));
    }

    let text =
        fs::read_to_string(path).map_err(|e| format!("Cannot read '{}': {}", path.display(), e))?;

    let mut json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Invalid JSON in '{}': {}", path.display(), e))?;

    // In mock mode, overlay `mock-organelles` onto `organelles` IN MEMORY —
    // never mutates the manifest file on disk. Mocks must live exclusively
    // under `mock-organelles`; `organelles` always describes the real backends.
    //
    // Selection rules:
    //   - `EUGLENA_MOCK_MODE=true` with no `EUGLENA_MOCK_TYPES` → overlay ALL.
    //   - `EUGLENA_MOCK_TYPES=type1,type2`                       → overlay
    //     ONLY entries whose underlying organelle type matches one of the
    //     listed types. The "type" is derived from the entry's `path`
    //     basename with any extension and trailing `_mock`/`-mock` removed
    //     (e.g. `organelles/mongodb_mock.so` → `mongodb`). User input is
    //     normalized: `-` and `_` are equivalent, lowercase.
    //   - Both unset → no overlay.
    if mock_overlay_enabled() {
        let selection = selected_mock_types();
        if let Some(obj) = json.as_object_mut() {
            let mock = obj.get("mock-organelles").cloned();
            if let Some(serde_json::Value::Object(mock_map)) = mock {
                let org_entry = obj
                    .entry("organelles".to_string())
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                if let serde_json::Value::Object(org_map) = org_entry {
                    for (k, v) in mock_map {
                        if let Some(ref allowed) = selection {
                            match organelle_type_from_value(&v) {
                                Some(t) if allowed.contains(&t) => {
                                    org_map.insert(k, v);
                                }
                                _ => { /* skip: type not in allow-list */ }
                            }
                        } else {
                            // No selection → mock everything (legacy behavior).
                            org_map.insert(k, v);
                        }
                    }
                }
            }
        }
    }

    // Interpolate ${VAR} references throughout the JSON tree.
    interpolate_env_vars(&mut json);

    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!(
                "'name' field missing or not a string in '{}'",
                path.display()
            )
        })?
        .to_string();

    let mut organelles = BTreeMap::new();
    if let Some(org_obj) = json.get("organelles").and_then(|v| v.as_object()) {
        for (alias, val) in org_obj {
            if let Some(s) = val.as_str() {
                organelles.insert(alias.clone(), OrganelleEntry::Path(s.to_string()));
            } else if let Some(obj) = val.as_object() {
                let org_path = obj
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        format!(
                            "organelle '{}' object must have a \"path\" string in '{}'",
                            alias,
                            path.display()
                        )
                    })?
                    .to_string();
                let sap = obj
                    .get("sap")
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_default();
                organelles.insert(
                    alias.clone(),
                    OrganelleEntry::Full {
                        path: org_path,
                        sap,
                    },
                );
            } else {
                return Err(format!(
                    "organelle '{}' must be a string or object in '{}'",
                    alias,
                    path.display()
                ));
            }
        }
    }

    Ok(AppManifest { name, organelles })
}

// ===========================================================================
// .env file support
// ===========================================================================

/// Load a `.env` file into the process environment.
///
/// Each non-empty, non-comment line is parsed as `KEY=VALUE`.  Surrounding
/// quotes on the value (`"..."` or `'...'`) are stripped.  Lines that don't
/// contain `=` are silently skipped.
fn load_dotenv(path: &Path) {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return, // Missing .env is fine.
    };

    for line in std::io::BufReader::new(file).lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, raw_val)) = trimmed.split_once('=') {
            let key = key.trim();
            let val = raw_val.trim();
            // Strip surrounding quotes.
            let val = if (val.starts_with('"') && val.ends_with('"'))
                || (val.starts_with('\'') && val.ends_with('\''))
            {
                &val[1..val.len() - 1]
            } else {
                val
            };
            // SAFETY: set_var is not thread-safe, but this runs during
            // single-threaded manifest parsing before any threads are spawned.
            unsafe {
                std::env::set_var(key, val);
            }
        }
    }
}

/// Returns true if `EUGLENA_MOCK_MODE` is set to a truthy value.
fn mock_mode_enabled() -> bool {
    matches!(
        std::env::var("EUGLENA_MOCK_MODE").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("True") | Ok("yes")
    )
}

/// True if any kind of mock overlay should be applied — either the global
/// `EUGLENA_MOCK_MODE` flag is on, OR a non-empty `EUGLENA_MOCK_TYPES` is set.
fn mock_overlay_enabled() -> bool {
    if mock_mode_enabled() {
        return true;
    }
    matches!(std::env::var("EUGLENA_MOCK_TYPES"), Ok(ref s) if !s.trim().is_empty())
}

/// Normalize a user-provided or path-derived organelle type name.
/// Replaces `-` with `_` and lowercases. Empty input → empty string.
fn normalize_type(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace('-', "_")
}

/// Parse `EUGLENA_MOCK_TYPES` into a normalized allow-list.
/// Returns `None` when the env var is unset or empty (meaning "no selection",
/// i.e. mock everything when `EUGLENA_MOCK_MODE` is on); returns `Some(set)`
/// when a non-empty comma-separated list is provided.
fn selected_mock_types() -> Option<HashSet<String>> {
    let raw = std::env::var("EUGLENA_MOCK_TYPES").ok()?;
    let set: HashSet<String> = raw
        .split(',')
        .map(normalize_type)
        .filter(|s| !s.is_empty())
        .collect();
    if set.is_empty() {
        None
    } else {
        Some(set)
    }
}

/// Extract the normalized organelle type from a mock-organelles JSON entry.
/// The entry can be either a plain path string or `{ "path": "...", ... }`.
/// Returns `None` if no path can be found.
///
/// Type derivation: take the file basename, strip any extension, then strip
/// a trailing `_mock` or `-mock` suffix. e.g.:
///   - `organelles/mongodb_mock.so`        → `mongodb`
///   - `organelles/blob_storage_mock.so`   → `blob_storage`
///   - `organelles/cloud-drive-mock.wasm`  → `cloud_drive`
fn organelle_type_from_value(v: &serde_json::Value) -> Option<String> {
    let path_str = match v {
        serde_json::Value::String(s) => s.as_str(),
        serde_json::Value::Object(o) => o.get("path")?.as_str()?,
        _ => return None,
    };
    let base = std::path::Path::new(path_str)
        .file_stem()
        .and_then(|s| s.to_str())?;
    let normalized = normalize_type(base);
    let stripped = normalized.strip_suffix("_mock").unwrap_or(&normalized);
    Some(stripped.to_string())
}

/// Recursively walk a JSON value, replacing `${VAR}` patterns in strings
/// with the corresponding environment variable value.  Unknown variables
/// are left as the literal `${VAR}` string.
fn interpolate_env_vars(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            if s.contains("${") {
                let interpolated = interpolate_str(s);
                // If the result is a pure number (e.g. "${EUGLENA_PORT}" →
                // "9991"), promote the JSON string to a JSON number so
                // downstream consumers (server.Sap `port`, etc.) get the
                // proper type.
                if let Ok(n) = interpolated.parse::<u64>() {
                    *value = serde_json::Value::Number(n.into());
                } else if let Ok(n) = interpolated.parse::<i64>() {
                    *value = serde_json::Value::Number(n.into());
                } else if let Ok(f) = interpolated.parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(f) {
                        *value = serde_json::Value::Number(num);
                    } else {
                        *s = interpolated;
                    }
                } else {
                    *s = interpolated;
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                interpolate_env_vars(item);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                interpolate_env_vars(v);
            }
        }
        _ => {}
    }
}

/// Replace all `${VAR}` occurrences in `input` with env var values.
fn interpolate_str(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                var_name.push(ch);
            }
            match std::env::var(&var_name) {
                Ok(val) => result.push_str(&val),
                Err(_) => {
                    // Unset variable — substitute empty string so that the
                    // generated Code source remains parseable.  The organelle
                    // will fail gracefully at runtime if a connection string
                    // or secret is missing.
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
