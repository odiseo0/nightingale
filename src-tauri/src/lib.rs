pub mod components;
pub mod platform;
pub mod shared_kernel;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let initialize = || -> Result<_, Box<dyn std::error::Error>> {
                let directory = app.path().app_data_dir()?;
                std::fs::create_dir_all(&directory)?;

                platform::sqlite::open(&directory.join("nightingale.sqlite"))
            };
            let connection = initialize().map_err(|_| {
                std::io::Error::other(
                    "No se pudo abrir la base local. Se conserva el archivo existente. Revise los permisos y la compatibilidad del esquema.",
                )
            })?;
            app.manage(std::sync::Mutex::new(connection));
            app.manage(components::sync::coordinator::SyncCoordinator::default());
            app.manage(components::identity::desktop::DesktopSession::default());
            let has_identity = {
                let c = app.state::<std::sync::Mutex<rusqlite::Connection>>();
                let c = c.lock().map_err(|_|std::io::Error::other("estado local no disponible"))?;

                components::identity::repository::AccessStore::context(&components::identity::repository::SqliteAccessStore,&c)?.is_some()};

            if has_identity && platform::config::APIConfig::from_environment().is_ok(){components::identity::desktop::start_watch(app.handle().clone())?;}

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            components::diagnostics::get_diagnostics,
            components::storage::storage_get_status,
            components::sync::adapters::inbound::sync_list_pending,
            components::identity::desktop::identity_get_status,
            components::identity::desktop::identity_begin_login,
            components::identity::desktop::identity_complete_login,
            components::identity::desktop::identity_cancel_login,
            components::identity::desktop::identity_list_accounts,
            components::identity::desktop::identity_activate_account,
            components::identity::desktop::identity_logout,
            components::identity::desktop::sync_now,
            components::account::adapters::inbound::account_get_view,
            components::account::adapters::inbound::account_get_preferences,
            components::account::adapters::inbound::account_set_preference,
            components::account::adapters::inbound::account_update_details,
            components::account::adapters::inbound::account_update_policy,
            components::account::adapters::inbound::account_set_status,
            components::services::adapters::inbound::services_list_catalog,
            components::services::adapters::inbound::services_create_service,
            components::services::adapters::inbound::services_update_service,
            components::services::adapters::inbound::services_set_service_status,
            components::services::adapters::inbound::services_offer_service,
            components::services::adapters::inbound::services_update_service_offer,
            components::services::adapters::inbound::services_set_service_offer_status,
            components::services::adapters::inbound::services_update_professional_profile,
            components::patients::adapters::inbound_profiles::patients_register_patient,
            components::patients::adapters::inbound_profiles::patients_update_patient_profile,
            components::patients::adapters::inbound_profiles::patients_set_patient_completion,
            components::patients::adapters::inbound_profiles::patients_set_patient_profile_status,
            components::patients::adapters::inbound_profiles::patients_add_patient_identifier,
            components::patients::adapters::inbound_profiles::patients_list_patients,
            components::patients::adapters::inbound_profiles::patients_get_patient,
            components::patients::adapters::inbound_profiles::patients_list_patient_identifiers,
            components::patients::adapters::inbound_contacts::patients_create_contact_person,
            components::patients::adapters::inbound_contacts::patients_update_contact_person,
            components::patients::adapters::inbound_contacts::patients_set_contact_person_status,
            components::patients::adapters::inbound_contacts::patients_create_patient_contact_relationship,
            components::patients::adapters::inbound_contacts::patients_update_patient_contact_relationship,
            components::patients::adapters::inbound_contacts::patients_set_patient_contact_relationship_status,
            components::patients::adapters::inbound_contacts::patients_list_contacts,
            components::patients::adapters::inbound_contacts::patients_get_contact,
            components::patients::adapters::inbound_contacts::patients_list_patient_contacts,
            components::patients::adapters::inbound_contacts::patients_list_contact_patients,
            components::scheduling::adapters::inbound_rules::scheduling_create_rule,
            components::scheduling::adapters::inbound_rules::scheduling_update_rule,
            components::scheduling::adapters::inbound_rules::scheduling_set_rule_status,
            components::scheduling::adapters::inbound_exceptions::scheduling_create_exception,
            components::scheduling::adapters::inbound_exceptions::scheduling_update_exception,
            components::scheduling::adapters::inbound_exceptions::scheduling_set_exception_status,
            components::scheduling::adapters::inbound_blocks::scheduling_create_block,
            components::scheduling::adapters::inbound_blocks::scheduling_update_block,
            components::scheduling::adapters::inbound_blocks::scheduling_set_block_status,
            components::scheduling::adapters::inbound_queries::scheduling_list_rules,
            components::scheduling::adapters::inbound_queries::scheduling_list_exceptions,
            components::scheduling::adapters::inbound_queries::scheduling_list_blocks,
            components::scheduling::adapters::inbound_availability::scheduling_get_local_availability,
            components::scheduling::adapters::inbound_availability::scheduling_check_remote_availability,
            components::appointments::adapters::inbound_commands::appointments_create_appointment,
            components::appointments::adapters::inbound_commands::appointments_confirm_appointment,
            components::appointments::adapters::inbound_commands::appointments_reschedule_appointment,
            components::appointments::adapters::inbound_commands::appointments_cancel_appointment,
            components::appointments::adapters::inbound_commands::appointments_complete_appointment,
            components::appointments::adapters::inbound_commands::appointments_mark_no_show,
            components::appointments::adapters::inbound_queries::appointments_list_appointments,
            components::appointments::adapters::inbound_queries::appointments_get_appointment,
            components::appointments::adapters::inbound_queries::appointments_get_history,
            components::appointments::adapters::inbound_queries::appointments_get_booking_context,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app,event|{if matches!(event,tauri::RunEvent::Exit){components::identity::desktop::stop_watch(app);}});
}
