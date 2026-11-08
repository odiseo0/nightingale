//! SQLite catalog storage. Accepted views and local intentions have distinct writers.
use super::super::{application::*, domain::*};
use crate::{
    components::{
        identity::{
            domain::AccountAccess,
            repository::{AccessStore, SqliteAccessStore},
        },
        sync::{
            adapters::outbound::SqliteOperationStore, application::record_operation,
            domain::NewOperation,
        },
    },
    shared_kernel::{
        Clock, IdGenerator,
        id::{OperationContext, OperationId},
    },
};
use rusqlite::{Connection, TransactionBehavior, params};
use serde_json::{Value, json};

#[path = "intents.rs"]
mod intents;

pub struct SqliteCatalogRepository<'a> {
    pub connection: &'a mut Connection,
}

pub(super) fn storage_error(_: rusqlite::Error) -> ServicesError {
    ServicesError::new("INTERNAL_ERROR", "datos locales de catálogo no disponibles")
}

pub(super) fn read_services(c: &Connection, s: &OperationContext) -> Result<Vec<ServiceView>> {
    let mut q = c.prepare("SELECT service_id,professional_id,name,default_duration_minutes,status,version FROM service_catalog_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY service_id").map_err(storage_error)?;
    q.query_map(
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str()
        ],
        |r| {
            Ok(ServiceView {
                service_id: r.get(0)?,
                professional_id: r.get(1)?,
                name: r.get(2)?,
                default_duration_minutes: r.get(3)?,
                status: r.get(4)?,
                version: r.get(5)?,
            })
        },
    )
    .map_err(storage_error)?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(storage_error)
}
pub(super) fn read_offers(c: &Connection, s: &OperationContext) -> Result<Vec<ServiceOfferView>> {
    let mut q = c.prepare("SELECT professional_id,service_id,name,default_duration_minutes,duration_minutes_override,effective_duration_minutes,service_status,status,version,service_version FROM service_offer_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY professional_id,service_id").map_err(storage_error)?;
    q.query_map(
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str()
        ],
        |r| {
            Ok(ServiceOfferView {
                professional_id: r.get(0)?,
                service_id: r.get(1)?,
                name: r.get(2)?,
                default_duration_minutes: r.get(3)?,
                duration_minutes_override: r.get(4)?,
                effective_duration_minutes: r.get(5)?,
                service_status: r.get(6)?,
                status: r.get(7)?,
                version: r.get(8)?,
                service_version: r.get(9)?,
            })
        },
    )
    .map_err(storage_error)?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(storage_error)
}
pub(super) fn read_professionals(
    c: &Connection,
    s: &OperationContext,
) -> Result<Vec<ProfessionalView>> {
    let mut q = c.prepare("SELECT professional_id,professional_user_id,public_name,status,version FROM professional_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY professional_id").map_err(storage_error)?;
    q.query_map(
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str()
        ],
        |r| {
            Ok(ProfessionalView {
                professional_id: r.get(0)?,
                user_id: r.get(1)?,
                public_name: r.get(2)?,
                status: r.get(3)?,
                version: r.get(4)?,
            })
        },
    )
    .map_err(storage_error)?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(storage_error)
}

impl CatalogQuery for SqliteCatalogRepository<'_> {
    fn list_services(&self, s: &OperationContext) -> Result<Vec<ServiceView>> {
        read_services(self.connection, s)
    }

    fn service(&self, s: &OperationContext, id: &ServiceId) -> Result<Option<ServiceView>> {
        Ok(read_services(self.connection, s)?
            .into_iter()
            .find(|v| v.service_id == id.as_str()))
    }

    fn list_offers(&self, s: &OperationContext) -> Result<Vec<ServiceOfferView>> {
        read_offers(self.connection, s)
    }

    fn offer(
        &self,
        s: &OperationContext,
        p: &ProfessionalId,
        id: &ServiceId,
    ) -> Result<Option<ServiceOfferView>> {
        Ok(read_offers(self.connection, s)?
            .into_iter()
            .find(|v| v.professional_id == p.as_str() && v.service_id == id.as_str()))
    }

    fn list_professionals(&self, s: &OperationContext) -> Result<Vec<ProfessionalView>> {
        read_professionals(self.connection, s)
    }

    fn professional(
        &self,
        s: &OperationContext,
        p: &ProfessionalId,
    ) -> Result<Option<ProfessionalView>> {
        Ok(read_professionals(self.connection, s)?
            .into_iter()
            .find(|v| v.professional_id == p.as_str()))
    }
}

struct Prepared {
    kind: &'static str,
    key: Value,
    professional: String,
    service: Option<String>,
    payload: Value,
    draft: Value,
    depends_on: Vec<OperationId>,
}

fn service_draft(v: ServiceView) -> ServiceDraft {
    ServiceDraft {
        service_id: v.service_id,
        professional_id: v.professional_id,
        name: v.name,
        default_duration_minutes: v.default_duration_minutes,
        status: v.status,
    }
}

fn offer_draft(v: ServiceOfferView) -> ServiceOfferDraft {
    ServiceOfferDraft {
        professional_id: v.professional_id,
        service_id: v.service_id,
        duration_minutes_override: v.duration_minutes_override,
        status: v.status,
    }
}

fn json_value(value: &impl serde::Serialize) -> Result<Value> {
    serde_json::to_value(value)
        .map_err(|_| ServicesError::new("INTERNAL_ERROR", "borrador de catálogo inválido"))
}

fn missing() -> ServicesError {
    ServicesError::new("NOT_FOUND", "no hay una vista aceptada de esa entidad")
}

fn version(actual: i64, expected: i64) -> Result<()> {
    if actual != expected {
        return Err(ServicesError::new(
            "VERSION_CONFLICT",
            "la versión local cambió; revise el catálogo",
        ));
    }
    Ok(())
}

fn prepare(
    c: &Connection,
    access: &AccountAccess,
    m: &CatalogMutation,
    ids: &impl IdGenerator,
) -> Result<Prepared> {
    let s = &access.context;
    let mut dependencies = vec![];
    let (kind, professional, service, payload, draft) = match m {
        CatalogMutation::CreateService {
            professional,
            name,
            duration,
        } => {
            let id = ids.new_id().to_string();
            let d = ServiceDraft {
                service_id: id.clone(),
                professional_id: professional.as_str().into(),
                name: name.clone(),
                default_duration_minutes: duration.get(),
                status: "active".into(),
            };

            (
                "service",
                d.professional_id.clone(),
                Some(id),
                json!({"professionalId":professional.as_str(),"name":name,"defaultDurationMinutes":duration.get()}),
                json_value(&d)?,
            )
        }

        CatalogMutation::UpdateService { service, patch, .. } => {
            let v = read_services(c, s)?
                .into_iter()
                .find(|v| v.service_id == service.as_str())
                .ok_or_else(missing)?;
            version(v.version, m.version())?;
            let mut d = service_draft(v);
            let mut payload = serde_json::Map::new();

            if let Some(name) = &patch.name {
                d.name = name.clone();
                payload.insert("name".into(), json!(name));
            }

            if let Some(duration) = patch.default_duration_minutes {
                d.default_duration_minutes = duration.get();
                payload.insert("defaultDurationMinutes".into(), json!(duration.get()));
            }

            (
                "service",
                d.professional_id.clone(),
                Some(d.service_id.clone()),
                Value::Object(payload),
                json_value(&d)?,
            )
        }

        CatalogMutation::SetServiceStatus {
            service, status, ..
        } => {
            let v = read_services(c, s)?
                .into_iter()
                .find(|v| v.service_id == service.as_str())
                .ok_or_else(missing)?;
            version(v.version, m.version())?;
            let mut d = service_draft(v);
            d.status = status.as_str().into();
            (
                "service",
                d.professional_id.clone(),
                Some(d.service_id.clone()),
                json!({"status":status.as_str()}),
                json_value(&d)?,
            )
        }

        CatalogMutation::OfferService { key, adjustment } => {
            if read_offers(c, s)?.iter().any(|v| {
                v.professional_id == key.professional_id.as_str()
                    && v.service_id == key.service_id.as_str()
            }) {
                return Err(ServicesError::new(
                    "VERSION_CONFLICT",
                    "el servicio ya tiene una oferta aceptada",
                ));
            }

            let base = read_services(c, s)?
                .into_iter()
                .find(|v| v.service_id == key.service_id.as_str());
            let own = match base {
                Some(base) => base.professional_id == key.professional_id.as_str(),
                None => {
                    let pending = intents::load(c, s)?
                        .into_iter()
                        .rev()
                        .find(|v| {
                            v.kind == "service"
                                && v.service.as_deref() == Some(key.service_id.as_str())
                                && v.action == "CreateService"
                        })
                        .ok_or_else(missing)?;
                    let state = intents::state(c, access, &pending)?;

                    if matches!(
                        state.sync_status.as_str(),
                        "conflict" | "rejected" | "review_required"
                    ) {
                        return Err(ServicesError::new(
                            "DEPENDENCY_FAILED",
                            "revise primero el servicio pendiente",
                        ));
                    }

                    dependencies.push(OperationId::try_from(pending.operation_id.clone())?);
                    pending.professional == key.professional_id.as_str()
                }
            };

            if !own {
                return Err(ServicesError::new(
                    "PERMISSION_DENIED",
                    "el servicio no pertenece al profesional",
                ));
            }

            let d = ServiceOfferDraft {
                professional_id: key.professional_id.as_str().into(),
                service_id: key.service_id.as_str().into(),
                duration_minutes_override: adjustment.map(DurationMinutes::get),
                status: "active".into(),
            };
            (
                "service_offer",
                d.professional_id.clone(),
                Some(d.service_id.clone()),
                json!({"durationMinutesOverride":d.duration_minutes_override}),
                json_value(&d)?,
            )
        }

        CatalogMutation::UpdateServiceOffer { key, patch, .. } => {
            let v = read_offers(c, s)?
                .into_iter()
                .find(|v| {
                    v.professional_id == key.professional_id.as_str()
                        && v.service_id == key.service_id.as_str()
                })
                .ok_or_else(missing)?;
            version(v.version, m.version())?;
            let mut d = offer_draft(v);
            d.duration_minutes_override = patch.duration_minutes_override.map(DurationMinutes::get);
            (
                "service_offer",
                d.professional_id.clone(),
                Some(d.service_id.clone()),
                json!({"durationMinutesOverride":d.duration_minutes_override}),
                json_value(&d)?,
            )
        }

        CatalogMutation::SetServiceOfferStatus { key, status, .. } => {
            let v = read_offers(c, s)?
                .into_iter()
                .find(|v| {
                    v.professional_id == key.professional_id.as_str()
                        && v.service_id == key.service_id.as_str()
                })
                .ok_or_else(missing)?;
            version(v.version, m.version())?;
            let mut d = offer_draft(v);
            d.status = status.as_str().into();
            (
                "service_offer",
                d.professional_id.clone(),
                Some(d.service_id.clone()),
                json!({"status":status.as_str()}),
                json_value(&d)?,
            )
        }

        CatalogMutation::UpdateProfessionalProfile {
            professional,
            public_name,
            ..
        } => {
            let v = read_professionals(c, s)?
                .into_iter()
                .find(|v| v.professional_id == professional.as_str())
                .ok_or_else(missing)?;
            version(v.version, m.version())?;
            let d = ProfessionalDraft {
                professional_id: professional.as_str().into(),
                public_name: public_name.clone(),
            };
            (
                "professional",
                d.professional_id.clone(),
                None,
                json!({"publicName":public_name}),
                json_value(&d)?,
            )
        }
    };
    let owner = read_professionals(c, s)?
        .into_iter()
        .find(|v| v.professional_id == professional)
        .ok_or_else(missing)?;

    if owner.user_id != s.user_id.as_str() {
        return Err(ServicesError::new(
            "PERMISSION_DENIED",
            "solo puede editar su propio catálogo profesional",
        ));
    }

    let key = if kind == "service_offer" {
        json!({"professionalId":professional,"serviceId":service})
    } else if kind == "service" {
        json!(service)
    } else {
        json!(professional)
    };

    for prior in intents::load(c, s)? {
        if prior.kind == kind && prior.professional == professional && prior.service == service {
            let status = intents::state(c, access, &prior)?.sync_status;

            if !matches!(status.as_str(), "conflict" | "rejected") {
                return Err(ServicesError::new(
                    "VERSION_CONFLICT",
                    "sincronice o revise la intención pendiente antes de otro cambio",
                ));
            }
        }
    }

    Ok(Prepared {
        kind,
        key,
        professional,
        service,
        payload,
        draft,
        depends_on: dependencies,
    })
}

impl CatalogRepository for SqliteCatalogRepository<'_> {
    fn catalog(&mut self, access: &AccountAccess) -> Result<CatalogScreen> {
        require_read(access)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        intents::resolve(&tx, access)?;
        let screen = intents::screen(&tx, access)?;

        tx.commit().map_err(storage_error)?;

        Ok(screen)
    }

    fn persist(
        &mut self,
        access: &AccountAccess,
        m: CatalogMutation,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> Result<LocalMutationResult> {
        require_write(access, m.action())?;
        m.validate()?;

        // Local intent, immutable operation and any receipt resolution share this lock.
        // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let s = &access.context;
        let current = SqliteAccessStore
            .access(&tx, &s.account_id, &s.user_id, &s.device_id)?
            .ok_or_else(|| ServicesError::new("PERMISSION_DENIED", "no hay autorización local"))?;
        require_write(&current, m.action())?;

        if current.context.permission_epoch != s.permission_epoch {
            return Err(ServicesError::new(
                "PERMISSION_DENIED",
                "los permisos locales cambiaron",
            ));
        }

        intents::resolve(&tx, access)?;
        let prepared = prepare(&tx, access, &m, ids)?;
        let now = clock
            .now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|v| i64::try_from(v.as_millis()).ok())
            .ok_or_else(|| ServicesError::new("INTERNAL_ERROR", "reloj local inválido"))?;
        let id = record_operation(
            &tx,
            &SqliteOperationStore,
            ids,
            clock,
            NewOperation {
                context: s.clone(),
                depends_on: prepared.depends_on,
                action: m.action().into(),
                entity_type: prepared.kind.into(),
                entity_key: prepared.key.clone(),
                expected_version: m.version(),
                payload: prepared.payload.clone(),
                occurred_at_local: None,
            },
        )?;

        if prepared.kind == "service_offer" {
            tx.execute("INSERT INTO service_offer_local (account_id,user_id,device_id,operation_id,professional_id,service_id,action,expected_version,payload,draft,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?11)",params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),id.to_string(),prepared.professional,prepared.service,m.action(),m.version(),prepared.payload.to_string(),prepared.draft.to_string(),now]).map_err(storage_error)?;
        } else {
            tx.execute("INSERT INTO service_local (account_id,user_id,device_id,operation_id,entity_type,professional_id,service_id,action,expected_version,payload,draft,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?12)",params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),id.to_string(),prepared.kind,prepared.professional,prepared.service,m.action(),m.version(),prepared.payload.to_string(),prepared.draft.to_string(),now]).map_err(storage_error)?;
        }

        tx.commit().map_err(storage_error)?;

        Ok(LocalMutationResult {
            operation_id: id.to_string(),
            entity_type: prepared.kind.into(),
            entity_key: prepared.key,
            sync_status: "pending".into(),
        })
    }
}

fn row_blocked<T, D>(row: &CatalogRow<T, D>) -> bool {
    row.local_changes
        .iter()
        .any(|change| change.sync_status == "conflict" || change.sync_status == "rejected")
}

/// Pending or unknown local changes can still anchor a dependency; conflict
/// and rejection are filtered by `row_blocked` before this is used.
fn pending_reference<T, D>(row: &CatalogRow<T, D>) -> Option<String> {
    row.local_changes
        .iter()
        .find(|change| matches!(change.sync_status.as_str(), "pending" | "unknown"))
        .map(|change| change.operation_id.clone())
}

impl CatalogBookingQuery for SqliteCatalogRepository<'_> {
    fn booking_offer(
        &mut self,
        access: &AccountAccess,
        professional: &ProfessionalId,
        service: &ServiceId,
    ) -> Result<Option<BookingOffer>> {
        require_read(access)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        intents::resolve(&tx, access)?;
        let screen = intents::screen(&tx, access)?;
        tx.commit().map_err(storage_error)?;

        let professional_row = screen
            .professionals
            .iter()
            .find(|row| {
                row.accepted
                    .as_ref()
                    .map(|view| view.professional_id.as_str())
                    == Some(professional.as_str())
            })
            .ok_or_else(|| ServicesError::new("NOT_FOUND", "profesional no disponible"))?;
        let professional_view = professional_row
            .accepted
            .as_ref()
            .ok_or_else(|| ServicesError::new("NOT_FOUND", "profesional no disponible"))?;

        if professional_view.status != "active" || row_blocked(professional_row) {
            return Ok(None);
        }

        let offer_row = match screen.offers.iter().find(|row| {
            row.accepted
                .as_ref()
                .map(|view| {
                    view.professional_id.as_str() == professional.as_str()
                        && view.service_id.as_str() == service.as_str()
                })
                .unwrap_or(false)
                || row
                    .provisional
                    .as_ref()
                    .map(|draft| {
                        draft.professional_id.as_str() == professional.as_str()
                            && draft.service_id.as_str() == service.as_str()
                    })
                    .unwrap_or(false)
        }) {
            Some(row) => row,
            None => return Ok(None),
        };

        if row_blocked(offer_row) {
            return Ok(None);
        }

        // Default name and duration: accepted service view or provisional draft.
        let service_row = screen.services.iter().find(|row| {
            row.accepted
                .as_ref()
                .map(|view| view.service_id.as_str() == service.as_str())
                .unwrap_or(false)
                || row
                    .provisional
                    .as_ref()
                    .map(|draft| draft.service_id.as_str() == service.as_str())
                    .unwrap_or(false)
        });

        if service_row.map(row_blocked).unwrap_or(false) {
            return Ok(None);
        }

        let (service_name, default_duration, service_status) = match service_row {
            Some(row) => {
                let name = row
                    .accepted
                    .as_ref()
                    .map(|view| view.name.clone())
                    .or_else(|| row.provisional.as_ref().map(|draft| draft.name.clone()));
                let duration = row
                    .accepted
                    .as_ref()
                    .map(|view| view.default_duration_minutes)
                    .or_else(|| {
                        row.provisional
                            .as_ref()
                            .map(|draft| draft.default_duration_minutes)
                    });
                let status = row
                    .accepted
                    .as_ref()
                    .map(|view| view.status.clone())
                    .or_else(|| row.provisional.as_ref().map(|draft| draft.status.clone()));

                match (name, duration, status) {
                    (Some(name), Some(duration), Some(status)) => (name, duration, status),
                    _ => return Ok(None),
                }
            }
            None => return Ok(None),
        };

        let booking = match &offer_row.accepted {
            Some(view) => {
                let (status, effective_duration_minutes, reference) = match &offer_row.provisional {
                    Some(draft) => (
                        draft.status.clone(),
                        draft
                            .duration_minutes_override
                            .unwrap_or(view.default_duration_minutes),
                        CatalogReference::Provisional {
                            operation_id: pending_reference(offer_row).unwrap_or_default(),
                        },
                    ),
                    None => (
                        view.status.clone(),
                        view.effective_duration_minutes,
                        CatalogReference::Accepted {
                            version: view.version,
                            service_version: view.service_version,
                        },
                    ),
                };
                BookingOffer {
                    professional_id: professional.as_str().into(),
                    service_id: service.as_str().into(),
                    service_name: view.name.clone(),
                    professional_name: professional_view.public_name.clone(),
                    effective_duration_minutes,
                    status,
                    service_status: view.service_status.clone(),
                    reference,
                }
            }
            None => {
                // Offer created offline: name and duration come from the service.
                let draft = offer_row.provisional.as_ref().unwrap();
                BookingOffer {
                    professional_id: professional.as_str().into(),
                    service_id: service.as_str().into(),
                    service_name,
                    professional_name: professional_view.public_name.clone(),
                    effective_duration_minutes: draft
                        .duration_minutes_override
                        .unwrap_or(default_duration),
                    status: draft.status.clone(),
                    service_status,
                    reference: CatalogReference::Provisional {
                        operation_id: pending_reference(offer_row).unwrap_or_default(),
                    },
                }
            }
        };

        if booking.status != "active" || booking.service_status != "active" {
            return Ok(None);
        }

        Ok(Some(booking))
    }
}

#[cfg(test)]
#[path = "../repository_tests.rs"]
mod tests;
