use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::inspection::InspectionResult;

/// Flakeの探索元
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoverySource {
    Cli,
    ConfigFile,
    EnvironmentVariable,
    HomeDirectory,
    GitRepository,
    SystemHint,
}

/// 候補から得られた証拠
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Evidence {
    FlakeFile,
    FlakeLock,
    GitRepository,
    HostsDirectory,
    ModulesDirectory,
    HomeDirectory,
    SystemDirectory,
    ConfigurationNix,
    HardwareConfigurationNix,
    NixosConfigurationsText,
}

/// 候補のスコア
///
/// スコアは「自動選択の唯一の根拠」には使用しない。
/// 候補比較や診断情報として利用する。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CandidateScore {
    pub total: u32,
    pub reasons: Vec<String>,
}

/// 探索中に発見されたFlake候補
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeCandidate {
    /// Flakeのルートディレクトリ
    pub flake_root: PathBuf,

    /// flake.nixのパス
    pub flake_file: PathBuf,

    /// この候補が発見された経路
    pub sources: Vec<DiscoverySource>,

    /// 候補から得られた証拠
    pub evidence: Vec<Evidence>,

    /// 検査結果
    pub inspection: Option<InspectionResult>,

    /// 候補スコア
    pub score: CandidateScore,
}

impl FlakeCandidate {
    pub fn new(
        flake_root: PathBuf,
        flake_file: PathBuf,
    ) -> Self {
        Self {
            flake_root,
            flake_file,
            sources: Vec::new(),
            evidence: Vec::new(),
            inspection: None,
            score: CandidateScore::default(),
        }
    }

    /// 候補のスコアを計算する。
    ///
    /// このスコアだけで候補を自動選択することはしない。
    pub fn calculate_score(&mut self) {
        let mut score = CandidateScore::default();

        if self
            .evidence
            .contains(&Evidence::FlakeFile)
        {
            score.total += 5;
            score.reasons.push(
                "flake.nix exists".to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::FlakeLock)
        {
            score.total += 3;
            score.reasons.push(
                "flake.lock exists".to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::GitRepository)
        {
            score.total += 3;
            score.reasons.push(
                "Git repository".to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::HostsDirectory)
        {
            score.total += 2;
            score.reasons.push(
                "hosts/ directory exists".to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::ModulesDirectory)
        {
            score.total += 2;
            score.reasons.push(
                "modules/ directory exists".to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::HomeDirectory)
        {
            score.total += 1;
            score.reasons.push(
                "home/ directory exists".to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::SystemDirectory)
        {
            score.total += 2;
            score.reasons.push(
                "system/ directory exists".to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::ConfigurationNix)
        {
            score.total += 2;
            score.reasons.push(
                "configuration.nix exists".to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::HardwareConfigurationNix)
        {
            score.total += 1;
            score.reasons.push(
                "hardware-configuration.nix exists"
                    .to_string(),
            );
        }

        if self
            .evidence
            .contains(&Evidence::NixosConfigurationsText)
        {
            score.total += 3;
            score.reasons.push(
                "nixosConfigurations text found".to_string(),
            );
        }

        if let Some(inspection) = &self.inspection {
            if let super::NixEvaluation::Success {
                outputs,
            } = &inspection.nix_evaluation
            {
                if !outputs.nixos_configurations.is_empty()
                {
                    score.total += 20;

                    score.reasons.push(
                        "Nix evaluation found NixOS configurations"
                            .to_string(),
                    );
                }

                if !inspection
                    .environment_match
                    .hostname_matches
                    .is_empty()
                {
                    score.total += 50;

                    score.reasons.push(
                        "hostname matches current system"
                            .to_string(),
                    );
                }

                if !inspection
                    .environment_match
                    .system_matches
                    .is_empty()
                {
                    score.total += 20;

                    score.reasons.push(
                        "system matches current system"
                            .to_string(),
                    );
                }
            }
        }

        self.score = score;
    }
}