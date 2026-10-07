use serde::Serialize;

#[derive(Debug, Serialize)]
struct InstallResult {
    package: String,
    installed: bool,
}

#[tauri::command]
fn sudo_available() -> bool {
    nixos_json_controller::nixos::rebuild::sudo_cached()
}

#[tauri::command]
fn install_app(package: String, password: Option<String>) -> Result<InstallResult, String> {
    nixos_json_controller::install_package_with_password(&package, password.as_deref())?;

    Ok(InstallResult {
        package,
        installed: true,
    })
}

#[tauri::command]
async fn search_catalog(
    query: String,
) -> Result<Vec<nixos_json_controller::nixos::catalog::CatalogPackage>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        nixos_json_controller::nixos::catalog::search(&query)
    })
    .await
    .map_err(|error| format!("catalog search task failed: {error}"))?
}

#[tauri::command]
async fn get_catalog_app(
    id: String,
) -> Result<Option<nixos_json_controller::nixos::catalog::CatalogPackage>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        nixos_json_controller::nixos::catalog::find(&id)
    })
    .await
    .map_err(|error| format!("catalog lookup task failed: {error}"))?
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
            sudo_available,
            install_app,
            search_catalog,
            get_catalog_app,
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
