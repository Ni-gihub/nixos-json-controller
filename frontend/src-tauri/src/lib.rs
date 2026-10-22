use serde::Serialize;

const MAX_INSTALL_PACKAGE_BYTES: usize = 64;
const MAX_PASSWORD_BYTES: usize = 1024;
#[cfg(debug_assertions)]
const MAX_ICON_DEBUG_MESSAGE_BYTES: usize = 16 * 1024;
#[cfg(debug_assertions)]
const MAX_ICON_DEBUG_MESSAGE_CHARS: usize = 4096;
const MAX_CATALOG_QUERY_BYTES: usize = 512;
const MAX_CATALOG_ID_BYTES: usize = 256;
const MAX_CATALOG_ID_BATCH: usize = 50;
const MAX_PACKAGE_STATE_BATCH: usize = 100;

#[derive(Debug, Serialize)]
struct InstallResult {
    package: String,
    installed: bool,
    active_state_verified: bool,
}

/// Validate a string at the WebView-to-Rust boundary without including its contents in errors.
fn validate_bounded_input(value: &str, field: &str, max_bytes: usize) -> Result<(), String> {
    if value.len() > max_bytes {
        return Err(format!("{field} exceeds the {max_bytes}-byte limit"));
    }
    if value.contains('\0') {
        return Err(format!("{field} must not contain a NUL byte"));
    }
    Ok(())
}

fn validate_required_input(value: &str, field: &str, max_bytes: usize) -> Result<(), String> {
    validate_bounded_input(value, field, max_bytes)?;
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if value.trim() != value {
        return Err(format!("{field} must not have leading or trailing whitespace"));
    }
    Ok(())
}

fn validate_input_batch(
    values: &[String],
    field: &str,
    max_items: usize,
    max_item_bytes: usize,
) -> Result<(), String> {
    if values.len() > max_items {
        return Err(format!("{field} contains more than {max_items} items"));
    }

    for (index, value) in values.iter().enumerate() {
        validate_required_input(value, &format!("{field}[{index}]"), max_item_bytes)?;
    }
    Ok(())
}

fn validate_sudo_password(password: Option<&str>) -> Result<(), String> {
    let Some(password) = password else {
        return Ok(());
    };

    validate_bounded_input(password, "password", MAX_PASSWORD_BYTES)?;
    if password.is_empty() {
        return Err("password must not be empty".to_string());
    }
    if password.chars().any(|character| matches!(character, '\n' | '\r')) {
        return Err("password must not contain line breaks".to_string());
    }
    Ok(())
}

/// Forward opt-in WebView icon diagnostics to the terminal running the Tauri dev process.
/// Release builds deliberately do not emit WebView-supplied diagnostic messages.
#[tauri::command]
fn log_icon_debug(message: String) {
    #[cfg(debug_assertions)]
    {
        if !matches!(
            std::env::var("NXC_ICON_DEBUG").as_deref(),
            Ok("1" | "true" | "yes")
        ) || message.len() > MAX_ICON_DEBUG_MESSAGE_BYTES
        {
            return;
        }

        let safe_message = message
            .chars()
            .filter(|character| !character.is_control() || *character == '\t')
            .take(MAX_ICON_DEBUG_MESSAGE_CHARS)
            .collect::<String>();
        eprintln!("[nxc-icon-debug][webview] {safe_message}");
    }

    #[cfg(not(debug_assertions))]
    let _ = message;
}

#[tauri::command]
fn sudo_available() -> bool {
    nixos_json_controller::nixos::rebuild::sudo_cached()
}

#[tauri::command]
async fn install_app(package: String, password: Option<String>) -> Result<InstallResult, String> {
    // The core repeats its own validation before resolving or applying a NixOS change.
    validate_required_input(&package, "package", MAX_INSTALL_PACKAGE_BYTES)?;
    validate_sudo_password(password.as_deref())?;

    tauri::async_runtime::spawn_blocking(move || {
        let outcome = nixos_json_controller::install_package_with_password_outcome(
            &package,
            password.as_deref(),
        )?;

        Ok(InstallResult {
            package: outcome.package,
            installed: true,
            active_state_verified: outcome.active_state_verified,
        })
    })
    .await
    .map_err(|error| format!("installation task failed: {error}"))?
}

#[tauri::command]
async fn refresh_local_icon_cache() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        nixos_json_controller::nixos::appstream::refresh_local_icon_cache();
    })
    .await
    .map_err(|error| format!("icon cache refresh task failed: {error}"))
}

#[tauri::command]
async fn get_icon_candidates(package: String, name: String) -> Result<Vec<String>, String> {
    validate_required_input(&package, "package", MAX_CATALOG_ID_BYTES)?;
    validate_required_input(&name, "name", MAX_CATALOG_ID_BYTES)?;

    tauri::async_runtime::spawn_blocking(move || {
        Ok(
            nixos_json_controller::nixos::appstream::icon_candidates_for_package(&[
                package.as_str(),
                name.as_str(),
            ]),
        )
    })
    .await
    .map_err(|error| format!("icon candidate lookup task failed: {error}"))?
}

#[tauri::command]
async fn prepare_catalog(
    force_refresh: bool,
) -> Result<nixos_json_controller::nixos::catalog::CatalogPreparationStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        nixos_json_controller::nixos::catalog::prepare_catalog(force_refresh)
    })
    .await
    .map_err(|error| format!("catalog preparation task failed: {error}"))?
}

#[tauri::command]
async fn search_catalog(
    query: String,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<nixos_json_controller::nixos::catalog::CatalogSearchPage, String> {
    validate_bounded_input(&query, "query", MAX_CATALOG_QUERY_BYTES)?;

    tauri::async_runtime::spawn_blocking(move || {
        nixos_json_controller::nixos::catalog::search_page(
            &query,
            offset.unwrap_or(0),
            limit.unwrap_or(40),
        )
    })
    .await
    .map_err(|error| format!("catalog search task failed: {error}"))?
}

#[tauri::command]
async fn get_catalog_app(
    id: String,
) -> Result<Option<nixos_json_controller::nixos::catalog::CatalogPackage>, String> {
    validate_required_input(&id, "id", MAX_CATALOG_ID_BYTES)?;

    tauri::async_runtime::spawn_blocking(move || nixos_json_controller::nixos::catalog::find(&id))
        .await
        .map_err(|error| format!("catalog lookup task failed: {error}"))?
}

#[tauri::command]
async fn get_catalog_apps(
    ids: Vec<String>,
) -> Result<Vec<nixos_json_controller::nixos::catalog::CatalogPackage>, String> {
    validate_input_batch(&ids, "ids", MAX_CATALOG_ID_BATCH, MAX_CATALOG_ID_BYTES)?;

    tauri::async_runtime::spawn_blocking(move || {
        nixos_json_controller::nixos::catalog::find_many(&ids)
    })
    .await
    .map_err(|error| format!("catalog batch lookup task failed: {error}"))?
}

#[tauri::command]
async fn get_app_states(
    packages: Vec<String>,
) -> Result<Vec<nixos_json_controller::nixos::status::PackageState>, String> {
    validate_input_batch(&packages, "packages", MAX_PACKAGE_STATE_BATCH, MAX_CATALOG_ID_BYTES)?;

    tauri::async_runtime::spawn_blocking(move || {
        nixos_json_controller::nixos::status::package_states(&packages)
    })
    .await
    .map_err(|error| format!("app state lookup task failed: {error}"))?
}

#[tauri::command]
async fn get_app_state(
    package: String,
) -> Result<nixos_json_controller::nixos::status::PackageState, String> {
    validate_required_input(&package, "package", MAX_CATALOG_ID_BYTES)?;

    tauri::async_runtime::spawn_blocking(move || {
        nixos_json_controller::nixos::status::package_state(&package)
    })
    .await
    .map_err(|error| format!("app state lookup task failed: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            log_icon_debug,
            sudo_available,
            install_app,
            refresh_local_icon_cache,
            get_icon_candidates,
            prepare_catalog,
            search_catalog,
            get_catalog_app,
            get_catalog_apps,
            get_app_states,
            get_app_state
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_input_rejects_oversized_and_nul_values_without_echoing_them() {
        assert!(validate_bounded_input("a".repeat(5).as_str(), "query", 4).is_err());
        let secret = "secret\0value";
        let error = validate_bounded_input(secret, "query", 64).unwrap_err();
        assert!(!error.contains(secret));
    }

    #[test]
    fn required_input_rejects_blank_values_and_surrounding_whitespace() {
        assert!(validate_required_input("   ", "package", 64).is_err());
        assert!(validate_required_input(" firefox", "package", 64).is_err());
        assert!(validate_required_input("firefox ", "package", 64).is_err());
        assert!(validate_required_input("firefox", "package", 64).is_ok());
    }

    #[test]
    fn batch_input_limits_item_count_and_item_size() {
        let too_many = vec!["firefox".to_string(); 3];
        assert!(validate_input_batch(&too_many, "ids", 2, 16).is_err());

        let too_large = vec!["a-very-long-id".to_string()];
        assert!(validate_input_batch(&too_large, "ids", 5, 4).is_err());

        let normal = vec!["firefox".to_string(), "chromium".to_string()];
        assert!(validate_input_batch(&normal, "ids", 5, 16).is_ok());
    }

    #[test]
    fn password_input_is_bounded_and_rejects_line_delimiters() {
        assert!(validate_sudo_password(None).is_ok());
        assert!(validate_sudo_password(Some("correct horse")).is_ok());
        assert!(validate_sudo_password(Some("")).is_err());
        assert!(validate_sudo_password(Some("first\nsecond")).is_err());
        let long_password = "x".repeat(MAX_PASSWORD_BYTES + 1);
        assert!(validate_sudo_password(Some(long_password.as_str())).is_err());
    }
}
