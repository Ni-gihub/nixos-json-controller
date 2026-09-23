use std::env;

use nixos_json_controller::{
    command::{Action, Command, Target},
    executor::Executor,
    nixos::discovery::{
        discover_candidates,
        inspect_candidates,
        save_discovery,
        select_candidate,
        DiscoveryStatus,
        SearchOptions,
    },
    planner::Planner,
    resolver::Resolver,
    validator::Validator,
};

pub fn run() -> Result<(), String> {
    let args: Vec<String> =
        env::args().skip(1).collect();

    if args.len() == 1
        && (args[0] == "--help"
            || args[0] == "-h")
    {
        print_help();
        return Ok(());
    }

    if args.len() == 1
        && args[0] == "discover"
    {
        return run_discover();
    }

    let (action, target) =
        match args.as_slice() {
            [target] => {
                (
                    Action::InstallPackage,
                    target.clone(),
                )
            }

            [operation, target] => {
                let action =
                    match operation.as_str() {
                        "i" => {
                            Action::InstallPackage
                        }

                        "r" => {
                            Action::RemovePackage
                        }

                        "e" => {
                            Action::EnableService
                        }

                        "d" => {
                            Action::DisableService
                        }

                        _ => {
                            return Err(
                                "unknown operation. use i, r, e, or d"
                                    .to_string()
                            );
                        }
                    };

                (action, target.clone())
            }

            _ => {
                return Err(
                    "usage: nxc [i|r|e|d] <target>"
                        .to_string()
                );
            }
        };

    let command = Command {
        action,
        target: Target { raw: target },
    };

    Validator::validate(&command)
        .map_err(|e| format!("{:?}", e))?;

    let target =
        Resolver::resolve(
            command.action.clone(),
            command.target,
        )?;

    let plan =
        Planner::create(
            command.action,
            target,
        );

    Executor::execute(plan)
        .map_err(|e| format!("{:?}", e))?;

    Ok(())
}

fn print_help() {
    println!(
        "usage: nxc [i|r|e|d] <target>"
    );
    println!();
    println!("commands:");
    println!(
        "  nxc <package>       install package"
    );
    println!(
        "  nxc i <package>     install package"
    );
    println!(
        "  nxc r <package>     remove package"
    );
    println!(
        "  nxc e <service>     enable service"
    );
    println!(
        "  nxc d <service>     disable service"
    );
    println!(
        "  nxc discover        discover NixOS flake"
    );
}

fn run_discover() -> Result<(), String> {
    println!("NixOS Flake Discovery");
    println!();

    println!(
        "Searching for candidates..."
    );

    let home =
        env::var_os("HOME")
            .ok_or_else(|| {
                "HOME environment variable is not set"
                    .to_string()
            })?;

    let options = SearchOptions {
        roots: vec![home.into()],
        max_depth: 4,
    };

    let mut candidates =
        discover_candidates(&options);

    println!(
        "Found {} candidates.",
        candidates.len()
    );
    println!();

    if candidates.is_empty() {
        println!(
            "No candidates found."
        );

        return Ok(());
    }

    println!(
        "Inspecting candidates..."
    );

    inspect_candidates(
        &mut candidates
    );

    println!();

    let report =
        select_candidate(
            &candidates
        );

    match report.status {
        DiscoveryStatus::Success => {
            let selected =
                report
                    .selected
                    .as_ref()
                    .ok_or_else(|| {
                        "discovery succeeded without a selected result"
                            .to_string()
                    })?;

            println!(
                "Discovery successful."
            );
            println!();

            println!(
                "Flake: {}",
                selected
                    .flake_root
                    .display()
            );

            println!(
                "Configuration: {}",
                selected
                    .selected_configuration
                    .name
            );

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
                selection_method_name(
                    &selected.selection_method
                )
            );

            save_discovery(selected)?;

            println!();
            println!(
                "Discovery result saved."
            );
        }

        DiscoveryStatus::MultipleCandidates => {
            println!(
                "Multiple candidates found."
            );
            println!();

            println!(
                "The target could not be selected automatically."
            );

            println!();

            for (index, candidate)
                in candidates.iter().enumerate()
            {
                println!(
                    "[{}] {}",
                    index + 1,
                    candidate
                        .flake_root
                        .display()
                );
            }
        }

        DiscoveryStatus::NoCandidates => {
            println!(
                "No suitable NixOS flake candidates found."
            );
        }

        DiscoveryStatus::EvaluationFailed => {
            println!(
                "Flake evaluation failed."
            );
        }
    }

    Ok(())
}

fn selection_method_name(
    method: &nixos_json_controller::nixos::discovery::SelectionMethod,
) -> &'static str {
    use nixos_json_controller::nixos::discovery::SelectionMethod;

    match method {
        SelectionMethod::ExplicitCli => {
            "explicit CLI"
        }

        SelectionMethod::ConfigFile => {
            "config file"
        }

        SelectionMethod::EnvironmentVariable => {
            "environment variable"
        }

        SelectionMethod::UniqueCandidate => {
            "unique candidate"
        }

        SelectionMethod::HostnameMatch => {
            "hostname match"
        }

        SelectionMethod::SystemMatch => {
            "system match"
        }

        SelectionMethod::UserSelection => {
            "user selection"
        }
    }
}