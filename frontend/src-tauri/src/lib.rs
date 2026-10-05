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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      sudo_available,
      install_app
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
