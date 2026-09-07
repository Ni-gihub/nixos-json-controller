# NixOS JSON Controller

## Project

`nixos-json-controller` is a Rust CLI that safely applies structured JSON commands to a NixOS configuration.

The command is `nxc`.

The main pipeline is:

```text
JSON
  ↓
Command
  ↓
Validator
  ↓
Resolver
  ↓
Planner
  ↓
Executor
  ↓
NixOS module generation
  ↓
nixos-rebuild
```

## Design principles

* Rust is the safety and execution boundary.
* External inputs such as natural language or LLM-generated JSON must not directly control system execution.
* JSON input must be strictly validated.
* Do not silently guess, complete, or reinterpret ambiguous input.
* Do not add unsupported actions or targets without explicitly changing the command specification.
* NixOS configuration changes must be deterministic and intentional.
* Do not hardcode user-specific absolute paths.
* Do not introduce unnecessary abstractions or large refactors for small changes.
* Preserve the separation between command parsing, validation, resolution, planning, execution, and NixOS operations.

## Current supported actions

* `install_package`
* `remove_package`
* `enable_service`
* `disable_service`

## Development

Use the Rust toolchain provided by the Nix flake.

Run tests with:

```bash
cargo test
```

Check compilation with:

```bash
cargo check
```

Check formatting with:

```bash
cargo fmt --all -- --check
```

Run Clippy with:

```bash
cargo clippy --all-targets --all-features
```

Build with:

```bash
cargo build
```

## Testing guidelines

When changing behavior:

* Add or update tests where appropriate.
* Test invalid input as well as valid input.
* Pay particular attention to error handling and unexpected input.
* Do not introduce tests that modify the real NixOS system or require interactive `sudo` execution unless explicitly intended.

## Important

Before changing the architecture, understand the existing responsibility of each module and preserve the existing safety boundary.
