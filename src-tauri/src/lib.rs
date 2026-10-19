mod diagnostics;
mod sqlite;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let initialize = || -> Result<_, Box<dyn std::error::Error>> {
                let directory = app.path().app_data_dir()?;
                std::fs::create_dir_all(&directory)?;

                sqlite::open(&directory.join("nightingale.sqlite"))
            };
            let connection = initialize().map_err(|_| {
                std::io::Error::other(
                    "No se pudo abrir la base local. Se conserva el archivo existente. Revise los permisos y la compatibilidad del esquema.",
                )
            })?;
            app.manage(std::sync::Mutex::new(connection));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![diagnostics::get_diagnostics])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
