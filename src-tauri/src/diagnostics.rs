#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    app_version: String,
    platform: &'static str,
    architecture: &'static str,
}

#[tauri::command]
pub fn get_diagnostics(app: tauri::AppHandle) -> Diagnostics {
    Diagnostics {
        app_version: app.package_info().version.to_string(),
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
    }
}
