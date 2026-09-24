use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::{
    inspect_flake,
    DiscoverySource,
    Evidence,
    FlakeCandidate,
};

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub roots: Vec<PathBuf>,
    pub max_depth: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            max_depth: 4,
        }
    }
}

/// Flake候補を探索する。
pub fn discover_candidates(
    options: &SearchOptions,
) -> Vec<FlakeCandidate> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    for root in &options.roots {
        if !root.is_dir() {
            continue;
        }

        search_directory(
            root,
            0,
            options.max_depth,
            &mut seen,
            &mut candidates,
        );
    }

    candidates
}

/// 発見した候補をNixまで含めて検査する。
pub fn inspect_candidates(
    candidates: &mut [FlakeCandidate],
) {
    for candidate in &mut *candidates {
        let inspection =
            inspect_flake(candidate);

        candidate.inspection =
            Some(inspection);

        candidate.calculate_score();
    }

    candidates.sort_by(|a, b| {
        b.score
            .total
            .cmp(&a.score.total)
            .then_with(|| {
                a.flake_root
                    .cmp(&b.flake_root)
            })
    });
}

fn search_directory(
    directory: &Path,
    depth: usize,
    max_depth: usize,
    seen: &mut HashSet<PathBuf>,
    candidates: &mut Vec<FlakeCandidate>,
) {
    if depth > max_depth {
        return;
    }

    let flake_file =
        directory.join("flake.nix");

    if flake_file.is_file() {
        let root =
            directory.to_path_buf();

        if seen.insert(root.clone()) {
            let candidate =
                inspect_candidate(
                    root,
                    flake_file,
                );

            candidates.push(candidate);
        }
    }

    let entries =
        match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(_) => return,
        };

    for entry in entries.flatten() {
        let path =
            entry.path();

        if !path.is_dir() {
            continue;
        }

        if should_skip_directory(&path) {
            continue;
        }

        search_directory(
            &path,
            depth + 1,
            max_depth,
            seen,
            candidates,
        );
    }
}

fn inspect_candidate(
    flake_root: PathBuf,
    flake_file: PathBuf,
) -> FlakeCandidate {
    let mut candidate =
        FlakeCandidate::new(
            flake_root.clone(),
            flake_file,
        );

    candidate
        .sources
        .push(
            DiscoverySource::HomeDirectory
        );

    candidate
        .evidence
        .push(Evidence::FlakeFile);

    let lock_file =
        flake_root.join("flake.lock");

    if lock_file.is_file() {
        candidate
            .evidence
            .push(Evidence::FlakeLock);
    }

    if flake_root
        .join(".git")
        .is_dir()
    {
        candidate
            .evidence
            .push(Evidence::GitRepository);
    }

    if flake_root
        .join("hosts")
        .is_dir()
    {
        candidate
            .evidence
            .push(Evidence::HostsDirectory);
    }

    if flake_root
        .join("modules")
        .is_dir()
    {
        candidate
            .evidence
            .push(Evidence::ModulesDirectory);
    }

    if flake_root
        .join("home")
        .is_dir()
    {
        candidate
            .evidence
            .push(Evidence::HomeDirectory);
    }

    if flake_root
        .join("system")
        .is_dir()
    {
        candidate
            .evidence
            .push(Evidence::SystemDirectory);
    }

    if flake_root
        .join("configuration.nix")
        .is_file()
    {
        candidate
            .evidence
            .push(
                Evidence::ConfigurationNix
            );
    }

    if flake_root
        .join("hardware-configuration.nix")
        .is_file()
    {
        candidate
            .evidence
            .push(
                Evidence::HardwareConfigurationNix
            );
    }

    if contains_nixos_configurations(
        &candidate.flake_file,
    ) {
        candidate
            .evidence
            .push(
                Evidence::NixosConfigurationsText
            );
    }

    candidate
}

fn contains_nixos_configurations(
    path: &Path,
) -> bool {
    let content =
        match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => return false,
        };

    content.contains(
        "nixosConfigurations"
    )
}

/// Discoveryで探索する価値が低いディレクトリ。
///
/// 特に `.nix-defexpr` はNixが生成する巨大な領域なので、
/// HOME全体探索では除外する。
fn should_skip_directory(
    path: &Path,
) -> bool {
    let Some(name) =
        path.file_name()
            .and_then(|name| name.to_str())
    else {
        return true;
    };

    matches!(
        name,
        ".git"
            | ".cache"
            | ".local"
            | ".nix-defexpr"
            | ".direnv"
            | ".cargo"
            | ".rustup"
            | "node_modules"
            | "target"
            | "result"
            | "result-bin"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_candidates_from_home() {
        let home =
            std::env::var_os("HOME")
                .expect(
                    "HOME environment variable is not set"
                );

        let options =
            SearchOptions {
                roots: vec![
                    PathBuf::from(home)
                ],
                max_depth: 4,
            };

        let candidates =
            discover_candidates(&options);

        println!(
            "found {} flake candidates",
            candidates.len()
        );

        for candidate in &candidates {
            println!(
                "--------------------------------"
            );

            println!(
                "flake root: {}",
                candidate
                    .flake_root
                    .display()
            );

            println!(
                "flake file: {}",
                candidate
                    .flake_file
                    .display()
            );

            println!(
                "sources: {:?}",
                candidate.sources
            );

            println!(
                "evidence: {:?}",
                candidate.evidence
            );
        }
    }

    #[test]
    fn inspect_nix_config_candidate() {
        let root =
            PathBuf::from(
                "/home/nakaoku/Projects/nix-config"
            );

        let flake_file =
            root.join("flake.nix");

        let mut candidate =
            FlakeCandidate::new(
                root,
                flake_file,
            );

        candidate
            .evidence
            .push(
                Evidence::NixosConfigurationsText
            );

        inspect_candidates(
            std::slice::from_mut(
                &mut candidate
            )
        );

        let inspection =
            candidate
                .inspection
                .expect(
                    "inspection result not found"
                );

        match inspection.nix_evaluation {
            super::super::NixEvaluation::Success {
                outputs
            } => {
                assert_eq!(
                    outputs
                        .nixos_configurations
                        .len(),
                    1
                );

                let laptop =
                    &outputs
                        .nixos_configurations[0];

                assert_eq!(
                    laptop.name,
                    "laptop"
                );

                assert_eq!(
                    laptop.system.as_deref(),
                    Some("x86_64-linux")
                );

                assert_eq!(
                    laptop.hostname.as_deref(),
                    Some("nixos-laptop")
                );
            }

            super::super::NixEvaluation::Failed {
                error
            } => {
                panic!(
                    "Nix evaluation failed: {error:?}"
                );
            }

            super::super::NixEvaluation::NotEvaluated => {
                panic!(
                    "Nix evaluation was not performed"
                );
            }
        }

        assert_eq!(
            inspection
                .environment_match
                .current_hostname
                .as_deref(),
            Some("nixos-laptop")
        );

        assert!(
            inspection
                .environment_match
                .hostname_matches
                .contains(
                    &"laptop".to_string()
                )
        );

        assert!(
            inspection
                .environment_match
                .system_matches
                .contains(
                    &"laptop".to_string()
                )
        );

        assert!(
            candidate.score.total > 0
        );
    }
}