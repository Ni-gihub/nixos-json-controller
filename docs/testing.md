# Testing and CI coverage

NXC uses separate checks for portable Rust logic, the frontend, the Tauri wrapper, and Nix packaging. Passing CI is not the same as validating a real NixOS activation on a user's machine.

## Checks run in CI

| Area | CI check | What it covers |
| --- | --- | --- |
| Root Rust crate | `cargo fmt --all -- --check` | Formatting for the root crate's Rust source and tests |
| Tauri Rust crate | `cargo fmt --manifest-path frontend/src-tauri/Cargo.toml --all -- --check` | Formatting for the separately managed Tauri crate |
| Rust logic | `cargo test --all-targets` | Root crate unit and integration tests on the CI host |
| Rust static analysis | `cargo clippy --all-targets --all-features` | Clippy diagnostics for root-crate targets and features |
| Root Rust build | `cargo check`, `cargo build` | Root crate compilation and build |
| Tauri wrapper | `cargo check --manifest-path frontend/src-tauri/Cargo.toml` | Compile-checks the Tauri crate and its native Linux build dependencies |
| Frontend tests | `pnpm test` | Discovers and executes Node test files, including newly added `*.test.mjs` tests |
| Frontend lint | `pnpm lint` | ESLint checks for configured TypeScript and TSX sources |
| Frontend build | `pnpm build` | TypeScript project build/type-check followed by Vite production build |
| CLI package | `nix build .#default --no-link --print-out-paths` | Builds the Flake package and validates its packaging/wrapper steps |

Cargo tests are executed by the dedicated Rust test job. The Flake derivation keeps `doCheck = false`, so Nix packaging validation does not silently substitute for the Cargo test job or repeat the suite in the package build.

The separate Nix-availability smoke test runs the ignored `nix_command_is_available` test after installing Nix. This only verifies that the Nix command can be invoked in that environment; it does not run `nixos-rebuild switch` against a real system.

## Not fully covered by hosted CI

The standard GitHub-hosted Linux runner is not a disposable NixOS machine configured like a user's host. CI does not prove the behavior of:

- real `sudo` password prompting and credential caching;
- a full `nixos-rebuild switch` and system activation on NixOS;
- recovery after a rebuild fails part-way through activation on a live system;
- interactive desktop/WebView behavior, including native authentication dialogs.

Changes to those paths still require a NixOS VM or a real NixOS test machine. Keep those checks separate from portable unit tests and record the environment and result when running them.
