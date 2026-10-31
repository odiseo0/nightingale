fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_diagnostics",
            "storage_get_status",
            "sync_list_pending",
            "identity_get_status",
            "identity_begin_login",
            "identity_complete_login",
            "identity_cancel_login",
            "identity_list_accounts",
            "identity_activate_account",
            "identity_logout",
            "sync_now",
        ]),
    ))
    .expect("failed to build Tauri application")
}
