# Tauri production security audit

Related issue: #106
Reviewed configuration and source: main as of 2026-10-23.

## Content Security Policy

The production CSP is defined in **frontend/src-tauri/tauri.conf.json** and does not include the Vite development origin.

| Directive | Purpose |
| --- | --- |
| **default-src** | Restricts unspecified resource types to the local app, Tauri's custom protocol, and its asset protocol. |
| **script-src** | Allows only local scripts. No remote script host or unsafe-eval is permitted. Tauri injects the required hashes/nonces for bundled assets at build time. |
| **style-src** | Allows local styles and unsafe-inline for existing React style attributes in the virtualized list and recommendation popularity indicator. This exception is limited to styles; scripts remain restricted. |
| **img-src** | Allows local app/asset URLs and data: images. Rust-resolved local icons and sanitized Iconify SVGs are represented as data URLs. |
| **connect-src** | Allows Tauri IPC (ipc: and http://ipc.localhost) and the only external icon metadata host, https://api.iconify.design. |
| **font-src** | Uses locally bundled font assets; no external font host is needed. |
| **object-src** | Blocks embedded plugin/object content. |
| **base-uri** and **form-action** | Keep document base URLs and form submission local. |

External icon lookup fetches text from Iconify, accepts only strict brand aliases, sanitizes SVG element/attribute content, then converts it to a data URL. The remote host is therefore present in connect-src, not in img-src; remote scripts are never allowed.

The development-only devCsp separately permits http://localhost:5173 and its Vite WebSocket (ws://localhost:5173). Those development sources are absent from production csp. No wildcard host, unsafe-eval, or remote script allowance is configured.

## Tauri capabilities

The main window capability no longer uses core:default. It grants only the four core window operations used by the custom title bar:

- core:window:allow-minimize
- core:window:allow-toggle-maximize
- core:window:allow-close
- core:window:allow-start-dragging

No filesystem, shell, dialog, HTTP, tray, menu, image, or WebView plugin permissions are requested. The application commands available to the local frontend remain explicitly enumerated in the Rust invoke_handler; those commands have their own input checks as described below. Capability restrictions do not replace validation inside application-defined commands.

The Tauri application identifier is now **io.github.ni-gihub.nxc-app**. Treat **frontend/src-tauri/tauri.conf.json** as the source of truth when completing package/desktop integration in #170; do not define a different identifier in the Nix package.

## Registered command review

| Command | Boundary and error handling |
| --- | --- |
| **log_icon_debug(message)** | Emits only in debug builds and only when NXC_ICON_DEBUG is enabled. Rejects oversized messages and removes control characters before output. Release builds do not emit WebView-supplied diagnostics. |
| **sudo_available()** | Takes no input; runs sudo -n -v and returns only a boolean. |
| **install_app(package, password)** | Bounds the package to 64 bytes and rejects blank, surrounding-whitespace, and NUL input before entering the blocking worker. The core validates again before resolution/application. Password input is capped at 1024 bytes, rejects empty/newline/carriage-return/NUL values, and is sent to sudo -S through stdin rather than command-line arguments. Passwords are not included in command output, diagnostic messages, or returned failure strings. |
| **refresh_local_icon_cache()** | Takes no input. Runs blocking filesystem/index work on Tauri's blocking pool. |
| **get_icon_candidates(package, name)** | Requires non-empty values of at most 256 bytes each; does not invoke a shell or execute supplied data. |
| **prepare_catalog(force_refresh)** | Accepts a boolean only; catalog source/cache validation remains in the core implementation. |
| **search_catalog(query, offset, limit)** | Rejects NUL and query strings above 512 bytes. The catalog implementation caps pages at 40 results. |
| **get_catalog_app(id)** | Requires a non-empty ID of at most 256 bytes; core catalog lookup validates the Nix attribute path before invoking Nix. |
| **get_catalog_apps(ids)** | Caps batches at 50 IDs and each ID at 256 bytes; the core filters unsafe attribute paths and de-duplicates them. |
| **get_app_states(packages)** | Caps batches at 100 package IDs and each ID at 256 bytes; Nix option/output-name resolution remains in the core. |
| **get_app_state(package)** | Requires a non-empty package ID of at most 256 bytes; Nix option/output-name resolution remains in the core. |

install_app is the only GUI command accepting a sudo password. The frontend clears the password field after submission (tracked separately by #103); the Rust side does not write the password to logs or error strings. The core rebuild runner sends it to sudo over stdin and returns a generic failure for a failed rebuild.

## Verification status and remaining release gate

Automated Rust unit tests cover boundary size/NUL checks, batch limits, surrounding whitespace, and password line-delimiter handling. GitHub Actions must still confirm the Rust/Frontend/Tauri checks after this branch is pushed.

A built application still needs manual verification on NixOS with the actual WebKitGTK runtime:

- confirm search, package detail, local icon, Iconify fallback, and installation with production CSP enabled;
- inspect WebView console output and ensure no unexpected CSP violation remains;
- check password-required and password-free install paths and confirm no secret appears in the terminal/logs;
- validate launch and window controls from the packaged app.

CI does not prove live sudo prompting, a successful nixos-rebuild switch, or desktop/WebView behavior. The identifier is set here for #170 to reuse; packaging/desktop-launch verification remains under #101/#170.
