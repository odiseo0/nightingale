fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_diagnostics",
            "storage_get_status",
            "sync_list_pending",
        ]),
    ))
    .expect("failed to build Tauri application")
}
