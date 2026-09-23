use super::{
    DiscoveryReport,
    DiscoveryResult,
    DiscoveryStatus,
    FlakeCandidate,
    FlakeOutputs,
    InspectionResult,
    NixEvaluation,
    NixosConfiguration,
    SelectionMethod,
};

/// Discovery候補から最終的なFlake / NixOS configurationを選択する。
pub fn select_candidate(
    candidates: &[FlakeCandidate],
) -> DiscoveryReport {
    let mut report = DiscoveryReport::new(DiscoveryStatus::NoCandidates);

    report.candidates = candidates.to_vec();

    // 候補が存在しない
    if candidates.is_empty() {
        report
            .diagnostics
            .push("No flake candidates were found.".to_string());

        return report;
    }

    // Nix評価に成功している候補だけを対象にする
    let evaluated: Vec<&FlakeCandidate> = candidates
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.inspection.as_ref().map(|inspection| {
                    &inspection.nix_evaluation
                }),
                Some(NixEvaluation::Success { .. })
            )
        })
        .collect();

    if evaluated.is_empty() {
        report.status = DiscoveryStatus::EvaluationFailed;
        report
            .diagnostics
            .push(
                "No candidates could be successfully evaluated by Nix."
                    .to_string(),
            );

        return report;
    }

    // ------------------------------------------------------------
    // 1. hostname一致
    // ------------------------------------------------------------

    let hostname_matches = matching_candidates(
        &evaluated,
        |candidate| {
            candidate
                .inspection
                .as_ref()
                .map(|inspection| {
                    !inspection
                        .environment_match
                        .hostname_matches
                        .is_empty()
                })
                .unwrap_or(false)
        },
    );

    if hostname_matches.len() == 1 {
        return build_success_report(
            report,
            hostname_matches[0],
            SelectionMethod::HostnameMatch,
        );
    }

    if hostname_matches.len() > 1 {
        report.status = DiscoveryStatus::MultipleCandidates;
        report.diagnostics.push(
            "Multiple candidates match the current hostname."
                .to_string(),
        );

        return report;
    }

    // ------------------------------------------------------------
    // 2. system一致
    // ------------------------------------------------------------

    let system_matches = matching_candidates(
        &evaluated,
        |candidate| {
            candidate
                .inspection
                .as_ref()
                .map(|inspection| {
                    !inspection
                        .environment_match
                        .system_matches
                        .is_empty()
                })
                .unwrap_or(false)
        },
    );

    if system_matches.len() == 1 {
        return build_success_report(
            report,
            system_matches[0],
            SelectionMethod::SystemMatch,
        );
    }

    if system_matches.len() > 1 {
        report.status = DiscoveryStatus::MultipleCandidates;
        report.diagnostics.push(
            "Multiple candidates match the current system."
                .to_string(),
        );

        return report;
    }

    // ------------------------------------------------------------
    // 3. 一意な候補
    // ------------------------------------------------------------

    if evaluated.len() == 1 {
        return build_success_report(
            report,
            evaluated[0],
            SelectionMethod::UniqueCandidate,
        );
    }

    // ------------------------------------------------------------
    // 4. 複数候補
    // ------------------------------------------------------------

    report.status = DiscoveryStatus::MultipleCandidates;

    report.diagnostics.push(format!(
        "{} evaluated NixOS candidates remain after environment matching.",
        evaluated.len()
    ));

    report
}

/// 条件に一致する候補を抽出する。
fn matching_candidates<'a, F>(
    candidates: &[&'a FlakeCandidate],
    predicate: F,
) -> Vec<&'a FlakeCandidate>
where
    F: Fn(&FlakeCandidate) -> bool,
{
    candidates
        .iter()
        .copied()
        .filter(|candidate| predicate(candidate))
        .collect()
}

/// 最終的なDiscoveryResultを作る。
fn build_success_report(
    mut report: DiscoveryReport,
    candidate: &FlakeCandidate,
    selection_method: SelectionMethod,
) -> DiscoveryReport {
    let Some(inspection) = candidate.inspection.as_ref() else {
        report.status = DiscoveryStatus::EvaluationFailed;
        report
            .diagnostics
            .push(
                "Selected candidate has no inspection result."
                    .to_string(),
            );

        return report;
    };

    let configuration = match &inspection.nix_evaluation {
        NixEvaluation::Success { outputs } => {
            find_matching_configuration(outputs, inspection)
        }
        _ => None,
    };

    let Some(configuration) = configuration else {
        report.status = DiscoveryStatus::EvaluationFailed;

        report.diagnostics.push(format!(
            "Candidate '{}' was selected, but no matching NixOS configuration was found.",
            candidate.flake_root.display()
        ));

        return report;
    };

    report.status = DiscoveryStatus::Success;

    report.selected = Some(DiscoveryResult {
        flake_root: candidate.flake_root.clone(),
        flake_file: candidate.flake_file.clone(),
        selected_configuration: configuration,
        selection_method,
    });

    report
}

/// 環境に対応するNixOS configurationを取得する。
fn find_matching_configuration(
    outputs: &FlakeOutputs,
    inspection: &InspectionResult,
) -> Option<NixosConfiguration> {
    // hostname一致を優先
    for configuration in &outputs.nixos_configurations {
        if inspection
            .environment_match
            .hostname_matches
            .contains(&configuration.name)
        {
            return Some(configuration.clone());
        }
    }

    // hostnameが一致しなければsystem一致
    for configuration in &outputs.nixos_configurations {
        if inspection
            .environment_match
            .system_matches
            .contains(&configuration.name)
        {
            return Some(configuration.clone());
        }
    }

    // configurationが1つだけならそれを使う
    if outputs.nixos_configurations.len() == 1 {
        return outputs.nixos_configurations.first().cloned();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::nixos::discovery::{
        inspect_candidates,
        Evidence,
    };

    #[test]
    fn select_nix_config_by_hostname() {
        let root =
            PathBuf::from("/home/nakaoku/Projects/nix-config");

        let flake_file = root.join("flake.nix");

        let mut candidate =
            FlakeCandidate::new(root, flake_file);

        candidate
            .evidence
            .push(Evidence::NixosConfigurationsText);

        inspect_candidates(
            std::slice::from_mut(&mut candidate),
        );

        let report = select_candidate(&[candidate]);

        assert_eq!(
            report.status,
            DiscoveryStatus::Success
        );

        let selected = report
            .selected
            .expect("candidate should be selected");

        assert_eq!(
            selected.flake_root,
            PathBuf::from(
                "/home/nakaoku/Projects/nix-config"
            )
        );

        assert_eq!(
            selected.selected_configuration.name,
            "laptop"
        );

        assert_eq!(
            selected
                .selected_configuration
                .hostname
                .as_deref(),
            Some("nixos-laptop")
        );

        assert_eq!(
            selected.selection_method,
            SelectionMethod::HostnameMatch
        );
    }

    #[test]
    fn select_empty_candidates() {
        let report = select_candidate(&[]);

        assert_eq!(
            report.status,
            DiscoveryStatus::NoCandidates
        );

        assert!(report.selected.is_none());
        assert!(!report.diagnostics.is_empty());
    }
}