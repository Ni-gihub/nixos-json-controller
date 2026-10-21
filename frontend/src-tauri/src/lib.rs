use serde::Serialize;

#[derive(Debug, Serialize)]
struct InstallResult {
    package: String,
    installed: bool,
    active_state_verified: bool,
}

/// Forward opt-in WebView icon diagnostics to the terminal running `tauri dev`.
#[tauri::command]
fn log_icon_debug(message: String) {
    if matches!(
        std::env::var("NXC_ICON_DEBUG").as_deref(),
        Ok("1" | "true" | "yes")
    ) {
        eprintln!("[nxc-icon-debug][webview] {message}");
    }
}

#[tauri::command]
fn sudo_available() -> bool {
    nixos_json_controller::nixos::rebuild::sudo_cached()
}

#[tauri::command]
async fn install_app(package: String, password: Option<String>) -> Result<InstallResult, String> {
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
    tauri::async_runtime::spawn_blocking(move || nixos_json_controller::nixos::catalog::find(&id))
        .await
        .map_err(|error| format!("catalog lookup task failed: {error}"))?
}

#[tauri::command]
async fn get_catalog_apps(
    ids: Vec<String>,
) -> Result<Vec<nixos_json_controller::nixos::catalog::CatalogPackage>, String> {
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
