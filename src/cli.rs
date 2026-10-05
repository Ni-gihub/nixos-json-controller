use std::env;

use clap::Parser;
use nixos_json_controller::{
    command::{Action, Command, Target},
    executor::Executor,
    nixos::{
        config::ConfigState,
        discovery::{
            DiscoveryStatus, SearchOptions, discover_candidates, inspect_candidates,
            save_discovery, select_candidate,
        },
        provenance::evaluate_option,
        status::{explain_package, explain_service},
        system::SystemState,
    },
    planner::Planner,
    resolver::Resolver,
    validator::Validator,
};

#[derive(Debug, Parser)]
#[command(
    name = "nxc",
    version,
    about = "Safe NixOS configuration controller",
    after_help = "Commands:\n  nxc <package>       install package\n  nxc i <package>     install package\n  nxc r <package>     remove package\n  nxc e <service>     enable service\n  nxc d <service>     disable service\n  nxc list            list installed system commands/apps\n  nxc status          show current system status\n  nxc explain package <name>  explain package state and provenance\n  nxc explain service <name>  explain service state and provenance\n  nxc discover        discover NixOS flake\n\nOptions:\n  --dry-run           preview the execution plan without changes"
)]
struct CliArgs {
    /// Preview the execution plan without changing configuration or rebuilding.
    #[arg(long)]
    dry_run: bool,

    /// NXC command and its arguments.
    #[arg(value_name = "COMMAND", num_args = 0..)]
    command: Vec<String>,
}

pub fn run() -> Result<(), String> {
    let args = CliArgs::parse();

    if args.command.is_empty() {
        print_help();
        return Ok(());
    }

    if args.dry_run
        && matches!(
            args.command.first().map(String::as_str),
            Some("discover" | "list" | "status" | "explain")
        )
    {
        return Err("--dry-run is only valid for configuration actions".to_string());
    }

    match args.command.as_slice() {
        [command] if command == "discover" => return run_discover(),
        [command] if command == "list" => return run_list(),
        [command] if command == "status" => return run_status(),
        [command, kind, target] if command == "explain" => return run_explain(kind, target),
        [command, ..] if matches!(command.as_str(), "discover" | "list" | "status" | "explain") => {
            return Err("invalid command arguments. use nxc --help for usage".to_string());
        }
        _ => {}
    }

    let (action, target) = match args.command.as_slice() {
        [target] => (Action::InstallPackage, target.clone()),
        [operation, target] => {
            let action = match operation.as_str() {
                "i" => Action::InstallPackage,
                "r" => Action::RemovePackage,
                "e" => Action::EnableService,
                "d" => Action::DisableService,
                _ => {
                    return Err(
                        "unknown operation. use i, r, e, or d; or nxc --help for usage".to_string(),
                    );
                }
            };

            (action, target.clone())
        }
        _ => {
            return Err("usage: nxc [i|r|e|d] <target> [--dry-run]".to_string());
        }
    };

    let command = Command {
        action,
        target: Target { raw: target },
    };

    Validator::validate(&command).map_err(|e| e.to_string())?;

    let target = Resolver::resolve(command.action.clone(), command.target)?;

    let mut plan = Planner::create(command.action, target);
    plan.dry_run = args.dry_run;
    plan = Planner::prepare(plan).map_err(|e| format!("failed to create execution plan: {e}"))?;

    print_plan(&plan);

    Executor::execute(plan).map_err(|e| e.to_string())?;

    Ok(())
}

fn print_help() {
    println!("Use nxc --help for full command-line help.");
}

fn run_list() -> Result<(), String> {
    let context = nixos_json_controller::nixos::discovery::DiscoveryContext::load()?;
    let provenance = evaluate_option(
        context.flake_root(),
        context.configuration_name(),
        "environment.systemPackages",
    )?;
    let local_files = provenance.local_files(context.flake_root());
    let config = ConfigState::discover(context.flake_root())?;
    let packages = config.declared_packages_in(&local_files);

    println!("Configured system packages");
    println!();

    for package in &packages {
        println!("  {}", package);
    }

    println!();
    println!("Total: {}", packages.len());

    Ok(())
}

fn run_status() -> Result<(), String> {
    let system = SystemState::discover()?;

    println!("NXC Status");
    println!();
    println!(
        "Current generation: {}",
        system.current_generation.display()
    );
    println!("System commands/apps: {}", system.binaries.len());
    println!("Detected packages: {}", system.packages.len());
    println!("Enabled services: {}", system.enabled_services.len());

    match nixos_json_controller::nixos::discovery::DiscoveryContext::load() {
        Ok(context) => {
            println!();
            println!("Discovery:");
            println!("  Flake: {}", context.flake_root().display());
            println!("  Configuration: {}", context.configuration_name());
        }
        Err(_) => {
            println!();
            println!("Discovery: unavailable");
            println!("  Run nxc discover before configuration-changing commands.");
        }
    }

    Ok(())
}

fn run_explain(kind: &str, target: &str) -> Result<(), String> {
    let context = nixos_json_controller::nixos::discovery::DiscoveryContext::load()?;
    let system = SystemState::discover()?;

    match kind {
        "package" => {
            let dictionary = nixos_json_controller::dictionary::Dictionary::load()?;
            let package = dictionary
                .resolve_package(target)
                .ok_or_else(|| format!("unknown package target: {}", target))?;

            let explanation = explain_package(&context, &system, package)?;

            println!("Package: {}", explanation.name);
            println!("System present: {}", yes_no(explanation.system_present));
            println!(
                "Declared in config: {}",
                yes_no(explanation.declared_in_config)
            );

            print_locations("Declaration locations", &explanation.declaration_locations);
            print_locations(
                "Safe write locations",
                &explanation.safe_declaration_locations,
            );
            print_locations(
                "Unsafe write locations",
                &explanation.unsafe_declaration_locations,
            );
        }

        "service" => {
            let dictionary = nixos_json_controller::dictionary::Dictionary::load()?;
            let service = dictionary
                .resolve_service(target)
                .ok_or_else(|| format!("unknown service target: {}", target))?;

            let explanation = explain_service(&context, &system, service)?;

            println!("Service: {}", explanation.name);
            println!("System enabled: {}", yes_no(explanation.system_enabled));
            println!(
                "Declared in config: {}",
                yes_no(explanation.declared_in_config)
            );
            println!(
                "Enabled in evaluated config: {}",
                yes_no(explanation.enabled_in_config)
            );

            print_locations("Declaration locations", &explanation.declaration_locations);
            print_locations(
                "Safe write locations",
                &explanation.safe_declaration_locations,
            );
            print_locations(
                "Unsafe write locations",
                &explanation.unsafe_declaration_locations,
            );
        }

        _ => {
            return Err("usage: nxc explain [package|service] <target>".to_string());
        }
    }

    Ok(())
}

fn print_plan(plan: &nixos_json_controller::planner::ExecutionPlan) {
    println!("Execution Plan");
    println!();
    println!("  Action: {:?}", plan.action);
    println!("  Target: {}", plan.target.name);

    if let Some(details) = &plan.details {
        println!("  Strategy: {}", details.strategy);
        println!("  Change required: {}", yes_no(details.change_required));
        println!("  Rebuild required: {}", yes_no(details.rebuild_required));
        println!("  Reason: {}", details.reason);
        println!("  Affected files: {}", details.affected_files.len());

        for path in &details.affected_files {
            println!("    {}", path.display());
        }
    }

    if plan.dry_run {
        println!();
        println!("Dry run: no configuration changes or rebuild will be performed.");
    }
}

fn print_locations(label: &str, paths: &[std::path::PathBuf]) {
    println!("{}: {}", label, paths.len());

    for path in paths {
        println!("  {}", path.display());
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn run_discover() -> Result<(), String> {
    println!("NixOS Flake Discovery");
    println!();
    println!("Searching for candidates...");

    let home =
        env::var_os("HOME").ok_or_else(|| "HOME environment variable is not set".to_string())?;

    let options = SearchOptions {
        roots: vec![home.into()],
        max_depth: 4,
    };

    let mut candidates = discover_candidates(&options);

    println!("Found {} candidates.", candidates.len());
    println!();

    if candidates.is_empty() {
        println!("No candidates found.");
        return Ok(());
    }

    println!("Inspecting candidates...");

    inspect_candidates(&mut candidates);

    println!();
    println!("Candidate ranking:");

    for (index, candidate) in candidates.iter().enumerate() {
        println!("[{}] {}", index + 1, candidate.flake_root.display());

        println!("    score: {}", candidate.score.total);

        if let Some(inspection) = &candidate.inspection {
            match &inspection.nix_evaluation {
                nixos_json_controller::nixos::discovery::NixEvaluation::Success { outputs } => {
                    println!(
                        "    NixOS configurations: {}",
                        outputs.nixos_configurations.len()
                    );

                    for configuration in &outputs.nixos_configurations {
                        println!(
                            "      - {} ({}, {})",
                            configuration.name,
                            configuration.hostname.as_deref().unwrap_or("unknown"),
                            configuration.system.as_deref().unwrap_or("unknown")
                        );
                    }
                }

                nixos_json_controller::nixos::discovery::NixEvaluation::Failed { error } => {
                    println!("    Nix evaluation: failed ({:?})", error.category);
                }

                nixos_json_controller::nixos::discovery::NixEvaluation::NotEvaluated => {
                    println!("    Nix evaluation: not evaluated");
                }
            }
        }

        println!();
    }

    let report = select_candidate(&candidates);

    match report.status {
        DiscoveryStatus::Success => {
            let selected = report
                .selected
                .as_ref()
                .ok_or_else(|| "discovery succeeded without a selected result".to_string())?;

            println!("Discovery successful.");
            println!();
            println!("Flake: {}", selected.flake_root.display());
            println!("Configuration: {}", selected.selected_configuration.name);
            println!(
                "Hostname: {}",
                selected
                    .selected_configuration
                    .hostname
                    .as_deref()
                    .unwrap_or("unknown")
            );
            println!(
                "System: {}",
                selected
                    .selected_configuration
                    .system
                    .as_deref()
                    .unwrap_or("unknown")
            );
            println!(
                "Selection: {}",
                selection_method_name(&selected.selection_method)
            );

            save_discovery(selected)?;

            println!();
            println!("Discovery result saved.");
        }

        DiscoveryStatus::MultipleCandidates => {
            println!("Multiple candidates found.");
            println!();

            for diagnostic in &report.diagnostics {
                println!("Diagnostic: {}", diagnostic);
            }

            println!();
            println!("No candidate was selected automatically.");
            println!("Use the discovery information above to identify the intended flake.");
        }

        DiscoveryStatus::NoCandidates => {
            println!("No suitable NixOS flake candidates found.");

            for diagnostic in &report.diagnostics {
                println!("Diagnostic: {}", diagnostic);
            }
        }

        DiscoveryStatus::EvaluationFailed => {
            println!("Flake evaluation failed.");

            for diagnostic in &report.diagnostics {
                println!("Diagnostic: {}", diagnostic);
            }
        }
    }

    Ok(())
}

fn selection_method_name(
    method: &nixos_json_controller::nixos::discovery::SelectionMethod,
) -> &'static str {
    use nixos_json_controller::nixos::discovery::SelectionMethod;

    match method {
        SelectionMethod::ExplicitCli => "explicit CLI",
        SelectionMethod::ConfigFile => "config file",
        SelectionMethod::EnvironmentVariable => "environment variable",
        SelectionMethod::UniqueCandidate => "unique candidate",
        SelectionMethod::HostnameMatch => "hostname match",
        SelectionMethod::SystemMatch => "system match",
        SelectionMethod::UserSelection => "user selection",
    }
}

#[cfg(test)]
mod tests {
    use super::CliArgs;
    use clap::Parser;

    #[test]
    fn dry_run_is_accepted_before_command() {
        let args = CliArgs::try_parse_from(["nxc", "--dry-run", "firefox"]).unwrap();

        assert!(args.dry_run);
        assert_eq!(args.command, vec!["firefox"]);
    }

    #[test]
    fn dry_run_is_accepted_after_command() {
        let args = CliArgs::try_parse_from(["nxc", "i", "firefox", "--dry-run"]).unwrap();

        assert!(args.dry_run);
        assert_eq!(args.command, vec!["i", "firefox"]);
    }

    #[test]
    fn help_and_version_are_handled_by_clap() {
        assert!(CliArgs::try_parse_from(["nxc", "--help"]).is_err());
        assert!(CliArgs::try_parse_from(["nxc", "--version"]).is_err());
    }
}
