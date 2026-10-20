use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::config::is_package_attribute_segment;

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
        self.definitions
            .iter()
            .filter_map(|definition| resolve_definition_file(flake_root, &definition.file))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn contains_local_package(&self, flake_root: &Path, package: &str) -> bool {
        self.definitions.iter().any(|definition| {
            resolve_definition_file(flake_root, &definition.file)
                .is_some_and(|_| value_contains_package(&definition.value, package))
        })
    }

    pub fn contains_package(&self, package: &str) -> bool {
        self.definitions
            .iter()
            .any(|definition| value_contains_package(&definition.value, package))
    }

    /// Match an evaluated derivation name (including its version) against store paths.
    pub fn contains_local_output_name(&self, flake_root: &Path, output_name: &str) -> bool {
        self.definitions.iter().any(|definition| {
            resolve_definition_file(flake_root, &definition.file)
                .is_some_and(|_| value_contains_output_name(&definition.value, output_name))
        })
    }

    /// Match an evaluated derivation name against all evaluated option definitions.
    pub fn contains_output_name(&self, output_name: &str) -> bool {
        self.definitions
            .iter()
            .any(|definition| value_contains_output_name(&definition.value, output_name))
    }

    pub fn contains_local_boolean(&self, flake_root: &Path, expected: bool) -> bool {
        self.definitions.iter().any(|definition| {
            resolve_definition_file(flake_root, &definition.file).is_some_and(|_| {
                definition
                    .value
                    .as_bool()
                    .is_some_and(|value| value == expected)
            })
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

    let definitions: Vec<OptionDefinition> =
        serde_json::from_slice(&output.stdout).map_err(|error| {
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

/// Evaluate the merged, effective boolean value of a NixOS option.
///
/// Unlike definitionsWithLocations, this reads config.<option> after NixOS
/// module priorities (including lib.mkForce) have been applied.
pub fn evaluate_boolean_option(
    flake_root: &Path,
    configuration_name: &str,
    option: &str,
) -> Result<bool, String> {
    let expression = format!(".#nixosConfigurations.{configuration_name}.config.{option}");

    let output = Command::new("nix")
        .args(["eval", "--json", &expression])
        .current_dir(flake_root)
        .output()
        .map_err(|error| format!("failed to execute nix: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "nix eval failed for effective option '{}': {}",
            option,
            stderr.trim()
        ));
    }

    parse_boolean_option_value(option, &output.stdout)
}

/// Parse the JSON boolean returned by `nix eval`.
fn parse_boolean_option_value(option: &str, output: &[u8]) -> Result<bool, String> {
    let value: Value = serde_json::from_slice(output)
        .map_err(|error| format!("failed to parse effective option '{}': {}", option, error))?;

    value.as_bool().ok_or_else(|| {
        format!(
            "effective option '{}' did not evaluate to a boolean",
            option
        )
    })
}

/// Resolve a package attribute path to the derivation name used by Nix store paths.
pub fn evaluate_package_output_name(
    flake_root: &Path,
    configuration_name: &str,
    package: &str,
) -> Result<String, String> {
    if package.is_empty() || !package.split('.').all(is_package_attribute_segment) {
        return Err(format!("unsafe package attribute path: '{}'", package));
    }

    let expression = format!(".#nixosConfigurations.{configuration_name}.pkgs.{package}.name");
    let output = Command::new("nix")
        .args(["eval", "--raw", &expression])
        .current_dir(flake_root)
        .output()
        .map_err(|error| {
            format!(
                "failed to execute nix while resolving package '{}': {}",
                package, error
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "could not resolve package attribute path '{}' to a store output name: {}",
            package,
            stderr.trim()
        ));
    }

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() || name.contains('/') {
        return Err(format!(
            "Nix returned an invalid output name for package '{}'",
            package
        ));
    }

    Ok(name)
}

pub fn evaluate_package_provenance(
    flake_root: &Path,
    configuration_name: &str,
    package: &str,
) -> Result<EvaluatedProvenance, String> {
    let output_name = if package.contains('.') {
        Some(evaluate_package_output_name(
            flake_root,
            configuration_name,
            package,
        )?)
    } else {
        None
    };

    evaluate_package_provenance_with_output_name(
        flake_root,
        configuration_name,
        package,
        output_name.as_deref(),
    )
}

/// Evaluate package provenance using an already-resolved derivation name when available.
/// This avoids evaluating the same dotted attribute path more than once per operation.
pub(crate) fn evaluate_package_provenance_with_output_name(
    flake_root: &Path,
    configuration_name: &str,
    package: &str,
    output_name: Option<&str>,
) -> Result<EvaluatedProvenance, String> {
    let provenance = evaluate_option(flake_root, configuration_name, "environment.systemPackages")?;
    let found = match output_name {
        Some(output_name) => provenance.contains_local_output_name(flake_root, output_name),
        None => provenance.contains_local_package(flake_root, package),
    };

    if found {
        Ok(provenance)
    } else {
        Ok(EvaluatedProvenance::default())
    }
}

/// Locate the local definitions of a service-level or raw systemd enable option.
pub fn evaluate_service_provenance(
    flake_root: &Path,
    configuration_name: &str,
    service: &str,
) -> Result<EvaluatedProvenance, String> {
    let services = evaluate_optional_service_option(
        flake_root,
        configuration_name,
        &format!("services.{service}.enable"),
    )?;

    if !services.local_files(flake_root).is_empty() {
        return Ok(services);
    }

    evaluate_optional_service_option(
        flake_root,
        configuration_name,
        &format!("systemd.services.{service}.enable"),
    )
}

fn is_missing_option_error(error: &str) -> bool {
    error.contains("does not provide attribute")
}

/// Evaluate a service option while treating an undefined option as empty provenance.
fn evaluate_optional_service_option(
    flake_root: &Path,
    configuration_name: &str,
    option: &str,
) -> Result<EvaluatedProvenance, String> {
    match evaluate_option(flake_root, configuration_name, option) {
        Ok(provenance) => Ok(provenance),
        Err(error) if is_missing_option_error(&error) => Ok(EvaluatedProvenance {
            option: option.to_string(),
            definitions: Vec::new(),
        }),
        Err(error) => Err(error),
    }
}

fn resolve_definition_file(flake_root: &Path, definition_file: &Path) -> Option<PathBuf> {
    if definition_file.is_relative() {
        let candidate = flake_root.join(definition_file);
        return candidate.is_file().then_some(candidate);
    }

    if definition_file.starts_with(flake_root) {
        return definition_file
            .is_file()
            .then(|| definition_file.to_path_buf());
    }

    if let Some(relative) = store_source_relative_path(definition_file) {
        let candidate = flake_root.join(relative);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn store_source_relative_path(path: &Path) -> Option<&Path> {
    let mut components = path.components();

    if components.next() != Some(Component::RootDir) {
        return None;
    }

    match components.next() {
        Some(Component::Normal(component)) if component == "nix" => {}
        _ => return None,
    }

    match components.next() {
        Some(Component::Normal(component)) if component == "store" => {}
        _ => return None,
    }

    let source_component = components.next()?;
    let source_name = source_component.as_os_str().to_str()?;
    if !source_name.ends_with("-source") {
        return None;
    }

    let relative = components.as_path();
    (!relative.as_os_str().is_empty()).then_some(relative)
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

fn value_contains_output_name(value: &Value, output_name: &str) -> bool {
    match value {
        Value::String(value) => string_matches_output_name(value, output_name),
        Value::Array(values) => values
            .iter()
            .any(|value| value_contains_output_name(value, output_name)),
        Value::Object(values) => values
            .values()
            .any(|value| value_contains_output_name(value, output_name)),
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

fn string_matches_output_name(value: &str, output_name: &str) -> bool {
    let basename = value.rsplit('/').next().unwrap_or(value);
    if basename == output_name {
        return true;
    }

    basename
        .strip_suffix(output_name)
        .is_some_and(|prefix| prefix.len() > 1 && prefix.ends_with('-'))
}

fn string_matches_package(value: &str, package: &str) -> bool {
    let normalized = value.rsplit('/').next().unwrap_or(value);

    if normalized == package {
        return true;
    }

    let Some(marker) = normalized.find(&format!("-{package}-")) else {
        return false;
    };

    let suffix = &normalized[marker + package.len() + 2..];
    let version = suffix.split('-').next().unwrap_or_default();
    !version.is_empty()
        && version
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_effective_boolean_option_values() {
        assert!(parse_boolean_option_value("services.openssh.enable", b"true").unwrap());
        assert!(!parse_boolean_option_value("services.openssh.enable", b"false").unwrap());
        assert!(
            parse_boolean_option_value("services.openssh.enable", b"null")
                .unwrap_err()
                .contains("did not evaluate to a boolean")
        );
    }

    #[test]
    fn finds_package_in_nested_values() {
        let value = serde_json::json!([
            "/nix/store/example-firefox-1.0",
            {"name": "git"}
        ]);

        assert!(value_contains_package(&value, "firefox"));
        assert!(value_contains_package(&value, "git"));
        assert!(!value_contains_package(&value, "git-lfs"));
        assert!(!value_contains_package(&value, "vim"));

        let git_lfs = serde_json::json!("/nix/store/example-git-lfs-3.6.0");
        assert!(value_contains_package(&git_lfs, "git-lfs"));
        assert!(!value_contains_package(&git_lfs, "git"));
    }

    #[test]
    fn evaluated_provenance_matches_packages_from_any_definition() {
        let provenance = EvaluatedProvenance {
            option: "environment.systemPackages".to_string(),
            definitions: vec![OptionDefinition {
                file: PathBuf::from("/nix/store/nixpkgs-source/nixos/modules/foo.nix"),
                value: serde_json::json!(["/nix/store/example-vlc-3.0"]),
            }],
        };

        assert!(provenance.contains_package("vlc"));
        assert!(!provenance.contains_package("firefox"));
    }

    #[test]
    fn matches_versioned_derivation_names_in_store_paths() {
        let output_name = "python3.13-requests-2.32.4";
        let store_path = serde_json::json!(
            "/nix/store/0123456789abcdef0123456789abcdef-python3.13-requests-2.32.4"
        );

        assert!(value_contains_output_name(&store_path, output_name));
        assert!(!value_contains_output_name(&store_path, "requests"));
        assert!(!value_contains_output_name(
            &serde_json::json!(
                "/nix/store/0123456789abcdef0123456789abcdef-python3.13-urllib3-2.2.0"
            ),
            output_name
        ));
    }

    #[test]
    fn treats_missing_service_option_as_empty_provenance() {
        let error = "flake does not provide attribute 'nixosConfigurations.laptop.options.services.openssh.enable.definitionsWithLocations'";
        assert!(is_missing_option_error(error));
        assert!(!is_missing_option_error("error: evaluation failed"));
    }

    #[test]
    fn resolves_worktree_definition() {
        let root = std::env::temp_dir().join(format!("nxc-provenance-{}", std::process::id()));
        let local = root.join("configuration.nix");

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&local, "{}").unwrap();

        assert_eq!(resolve_definition_file(&root, &local), Some(local.clone()));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resolves_relative_definition() {
        let root =
            std::env::temp_dir().join(format!("nxc-provenance-relative-{}", std::process::id()));
        let local = root.join("nxc/packages.nix");

        std::fs::create_dir_all(local.parent().unwrap()).unwrap();
        std::fs::write(&local, "{}").unwrap();

        assert_eq!(
            resolve_definition_file(&root, Path::new("nxc/packages.nix")),
            Some(local)
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resolves_store_source_definition_without_source_root() {
        let root = std::env::temp_dir().join(format!(
            "nxc-provenance-store-source-{}",
            std::process::id()
        ));
        let local = root.join("nxc/packages.nix");

        std::fs::create_dir_all(local.parent().unwrap()).unwrap();
        std::fs::write(&local, "{}").unwrap();

        assert_eq!(
            resolve_definition_file(
                &root,
                Path::new("/nix/store/0123456789abcdef-source/nxc/packages.nix")
            ),
            Some(local)
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn excludes_unrelated_store_definition() {
        let root =
            std::env::temp_dir().join(format!("nxc-provenance-unrelated-{}", std::process::id()));
        let local = root.join("configuration.nix");

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&local, "{}").unwrap();

        assert_eq!(
            resolve_definition_file(
                &root,
                Path::new("/nix/store/other-source/nixos/modules/foo.nix")
            ),
            None
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
