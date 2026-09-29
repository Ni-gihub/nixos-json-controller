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
    pub fn files(&self) -> Vec<PathBuf> {
        let mut files = self
            .definitions
            .iter()
            .map(|definition| definition.file.clone())
            .collect::<BTreeSet<_>>();

        files.retain(|path| path.exists());

        files.into_iter().collect()
    }

    pub fn contains_package(&self, package: &str) -> bool {
        self.definitions
            .iter()
            .any(|definition| value_contains_package(&definition.value, package))
    }

    pub fn contains_boolean(&self, expected: bool) -> bool {
        self.definitions.iter().any(|definition| {
            definition
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
    let provenance = evaluate_option(
        flake_root,
        configuration_name,
        "environment.systemPackages",
    )?;

    if provenance.contains_package(package) {
        Ok(provenance)
    } else {
        Ok(EvaluatedProvenance::default())
    }
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

        assert!(provenance.contains_boolean(true));
        assert!(!provenance.contains_boolean(false));
    }

    #[test]
    fn collects_existing_definition_files() {
        let path = std::env::temp_dir().join(format!(
            "nxc-provenance-{}-config.nix",
            std::process::id()
        ));
        std::fs::write(&path, "{}").unwrap();

        let provenance = EvaluatedProvenance {
            option: "services.openssh.enable".to_string(),
            definitions: vec![OptionDefinition {
                file: path.clone(),
                value: Value::Bool(true),
            }],
        };

        assert_eq!(provenance.files(), vec![path.clone()]);
        let _ = std::fs::remove_file(path);
    }
}
