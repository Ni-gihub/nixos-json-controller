use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptionDefinition {
    pub file: PathBuf,
    pub value: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvaluatedProvenance {
    pub option: String,
    pub definitions: Vec<OptionDefinition>,
}

impl EvaluatedProvenance {
    pub fn local_files(&self, flake_root: &Path) -> Vec<PathBuf> {
        let root = match flake_root.canonicalize() {
            Ok(root) => root,
            Err(_) => return Vec::new(),
        };

        self.definitions
            .iter()
            .filter_map(|definition| {
                let path = definition.file.canonicalize().ok()?;
                path.starts_with(&root).then_some(path)
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn contains_local_package(&self, flake_root: &Path, package: &str) -> bool {
        self.definitions.iter().any(|definition| {
            definition.file.starts_with(flake_root)
                && value_contains_package(&definition.value, package)
        })
    }

    pub fn contains_local_boolean(&self, flake_root: &Path, expected: bool) -> bool {
        self.definitions.iter().any(|definition| {
            definition.file.starts_with(flake_root)
                && definition
                    .value
                    .as_bool()
                    .is_some_and(|value| value == expected)
        })
    }
}

pub fn evaluate_option(
    flake_root: &Path,
    configuration_name: &str,
    option: &str,
) -> Result<EvaluatedProvenance, String> {
    let expression = format!(
        ".#nixosConfigurations.{configuration_name}.options.{option}.definitionsWithLocations"
    );

    let output = Command::new("nix")
        .args(["eval", "--json", &expression])
        .current_dir(flake_root)
        .output()
        .map_err(|error| format!("failed to execute nix: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "nix eval failed for {}: {}",
            expression,
            stderr.trim()
        ));
    }

    let definitions: Vec<OptionDefinition> = serde_json::from_slice(&output.stdout)
        .map_err(|error| {
            format!(
                "failed to parse definitionsWithLocations for {}: {}",
                option, error
            )
        })?;

    Ok(EvaluatedProvenance {
        option: option.to_string(),
        definitions,
    })
}

pub fn evaluate_package_provenance(
    flake_root: &Path,
    configuration_name: &str,
    package: &str,
) -> Result<EvaluatedProvenance, String> {
    evaluate_option(
        flake_root,
        configuration_name,
        "environment.systemPackages",
    )
    .map(|provenance| {
        if provenance.contains_local_package(flake_root, package) {
            provenance
        } else {
            EvaluatedProvenance {
                option: "environment.systemPackages".to_string(),
                definitions: Vec::new(),
            }
        }
    })
}

pub fn evaluate_service_provenance(
    flake_root: &Path,
    configuration_name: &str,
    service: &str,
) -> Result<EvaluatedProvenance, String> {
    evaluate_option(
        flake_root,
        configuration_name,
        &format!("services.{service}.enable"),
    )
}

fn value_contains_package(value: &Value, package: &str) -> bool {
    match value {
        Value::String(value) => string_matches_package(value, package),
        Value::Array(values) => values
            .iter()
            .any(|value| value_contains_package(value, package)),
        Value::Object(values) => values
            .values()
            .any(|value| value_contains_package(value, package)),
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

fn string_matches_package(value: &str, package: &str) -> bool {
    let normalized = value.rsplit('/').next().unwrap_or(value);

    normalized == package
        || normalized.starts_with(&format!("{package}-"))
        || normalized.ends_with(&format!("-{package}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_package_in_nested_values() {
        let value = serde_json::json!([
            "/nix/store/example-firefox-1.0",
            {"name": "git"}
        ]);

        assert!(value_contains_package(&value, "firefox"));
        assert!(value_contains_package(&value, "git"));
        assert!(!value_contains_package(&value, "vim"));
    }

    #[test]
    fn recognizes_boolean_option_definition() {
        let provenance = EvaluatedProvenance {
            option: "services.openssh.enable".to_string(),
            definitions: vec![OptionDefinition {
                file: PathBuf::from("/tmp/configuration.nix"),
                value: Value::Bool(true),
            }],
        };

        assert!(provenance.contains_local_boolean(Path::new("/tmp"), true));
        assert!(!provenance.contains_local_boolean(Path::new("/tmp"), false));
    }

    #[test]
    fn excludes_nix_store_definition_files() {
        let root = std::env::temp_dir().join(format!("nxc-provenance-{}", std::process::id()));
        let local = root.join("configuration.nix");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&local, "{}").unwrap();

        let provenance = EvaluatedProvenance {
            option: "services.openssh.enable".to_string(),
            definitions: vec![
                OptionDefinition {
                    file: local.clone(),
                    value: Value::Bool(true),
                },
                OptionDefinition {
                    file: PathBuf::from("/nix/store/nixos-module.nix"),
                    value: Value::Bool(false),
                },
            ],
        };

        assert_eq!(provenance.local_files(&root), vec![local]);
        let _ = std::fs::remove_dir_all(root);
    }
}
