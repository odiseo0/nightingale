use super::super::domain::{DurationMinutes, OfferKey, ProfessionalId, ServiceId};
use crate::{
    components::sync::{
        adapters::remote_models::{AccountResponseStatus, ProjectionId, RemoteState},
        remote::{EntityType, RemoteChange, RemoteProjector},
    },
    shared_kernel::{NightingaleError, Result, id::OperationContext},
};
use rusqlite::{Transaction, params};

pub struct ServicesProjector;

fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("proyección local de catálogo no disponible")
}

fn id(value: &ProjectionId) -> String {
    value.clone().into()
}

fn status(value: &AccountResponseStatus) -> &'static str {
    match value {
        AccountResponseStatus::Active => "active",
        AccountResponseStatus::Inactive => "inactive",
    }
}

fn key(value: &serde_json::Value) -> Result<String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| NightingaleError::validation("clave remota inválida"))
}

impl RemoteProjector for ServicesProjector {
    fn reset(&self, tx: &Transaction, s: &OperationContext) -> Result<()> {
        for table in [
            "service_catalog_view",
            "service_offer_view",
            "professional_view",
        ] {
            tx.execute(
                &format!("DELETE FROM {table} WHERE account_id=?1 AND user_id=?2 AND device_id=?3"),
                params![
                    s.account_id.as_str(),
                    s.user_id.as_str(),
                    s.device_id.as_str()
                ],
            )
            .map_err(storage_error)?;
        }

        Ok(())
    }

    fn project(&self, tx: &Transaction, s: &OperationContext, change: &RemoteChange) -> Result<()> {
        match change {
            RemoteChange::Upsert { state, .. } => match state.as_ref() {
                RemoteState::Service(v) => {
                    DurationMinutes::new(v.default_duration_minutes)?;
                    tx.execute("INSERT INTO service_catalog_view (account_id,user_id,device_id,service_id,professional_id,name,default_duration_minutes,status,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(account_id,user_id,device_id,service_id) DO UPDATE SET professional_id=excluded.professional_id,name=excluded.name,default_duration_minutes=excluded.default_duration_minutes,status=excluded.status,version=excluded.version WHERE excluded.version>service_catalog_view.version",
                        params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),id(&v.id),id(&v.professional_id),v.name,v.default_duration_minutes,status(&v.status),v.version]).map_err(storage_error)?;
                }
                RemoteState::ServiceOffer(v) => {
                    if id(&v.account_id) != s.account_id.as_str() {
                        return Err(NightingaleError::validation(
                            "oferta remota fuera de ámbito",
                        ));
                    }

                    let base = DurationMinutes::new(v.default_duration_minutes)?;
                    let adjustment = v
                        .duration_minutes_override
                        .map(DurationMinutes::new)
                        .transpose()?;

                    if DurationMinutes::effective(base, adjustment).get()
                        != v.effective_duration_minutes
                    {
                        return Err(NightingaleError::validation(
                            "duración remota de oferta inválida",
                        ));
                    }

                    // Equal offer versions can carry a later accepted catalog version.
                    // https://www.sqlite.org/lang_upsert.html
                    tx.execute("INSERT INTO service_offer_view (account_id,user_id,device_id,professional_id,service_id,name,default_duration_minutes,duration_minutes_override,effective_duration_minutes,service_status,status,version,service_version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
                        ON CONFLICT(account_id,user_id,device_id,professional_id,service_id) DO UPDATE SET name=excluded.name,default_duration_minutes=excluded.default_duration_minutes,duration_minutes_override=excluded.duration_minutes_override,effective_duration_minutes=excluded.effective_duration_minutes,service_status=excluded.service_status,status=excluded.status,version=excluded.version,service_version=excluded.service_version
                        WHERE excluded.version>service_offer_view.version OR (excluded.version=service_offer_view.version AND excluded.service_version>service_offer_view.service_version)",
                        params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),id(&v.professional_id),id(&v.service_id),v.name,v.default_duration_minutes,v.duration_minutes_override,v.effective_duration_minutes,status(&v.service_status),status(&v.status),v.version,v.service_version]).map_err(storage_error)?;
                }
                RemoteState::Professional(v) => {
                    tx.execute("INSERT INTO professional_view (account_id,user_id,device_id,professional_id,professional_user_id,public_name,status,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
                        ON CONFLICT(account_id,user_id,device_id,professional_id) DO UPDATE SET professional_user_id=excluded.professional_user_id,public_name=excluded.public_name,status=excluded.status,version=excluded.version WHERE excluded.version>professional_view.version",
                        params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),id(&v.id),id(&v.user_id),v.public_name,status(&v.status),v.version]).map_err(storage_error)?;
                }
                _ => {}
            },
            RemoteChange::Remove {
                entity_type,
                key: entity_key,
            } => match entity_type {
                EntityType::Service => {
                    let service = ServiceId::try_from(key(entity_key)?)?;
                    tx.execute("DELETE FROM service_catalog_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND service_id=?4",params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),service.as_str()]).map_err(storage_error)?;
                }
                EntityType::Professional => {
                    let professional = ProfessionalId::try_from(key(entity_key)?)?;
                    tx.execute("DELETE FROM professional_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND professional_id=?4",params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),professional.as_str()]).map_err(storage_error)?;
                }
                EntityType::ServiceOffer => {
                    if entity_key.as_object().is_none_or(|key| key.len() != 2) {
                        return Err(NightingaleError::validation(
                            "clave compuesta remota inválida",
                        ));
                    }
                    let offer = OfferKey::new(
                        key(&entity_key["professionalId"])?,
                        key(&entity_key["serviceId"])?,
                    )?;
                    tx.execute("DELETE FROM service_offer_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND professional_id=?4 AND service_id=?5",params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),offer.professional_id.as_str(),offer.service_id.as_str()]).map_err(storage_error)?;
                }
                _ => {}
            },
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "../projection_tests.rs"]
mod tests;
