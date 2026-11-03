use super::*;
use crate::components::sync::domain::OperationStatus;
use std::collections::BTreeMap;

pub(super) struct Intent {
    pub operation_id: String,
    pub kind: String,
    pub professional: String,
    pub service: Option<String>,
    pub action: String,
    pub version: i64,
    pub draft: Value,
}

pub(super) fn load(c: &Connection, s: &OperationContext) -> Result<Vec<Intent>> {
    let mut q = c.prepare("SELECT operation_id,entity_type,professional_id,service_id,action,expected_version,draft,created_at FROM service_local WHERE account_id=?1 AND user_id=?2 AND device_id=?3 UNION ALL SELECT operation_id,'service_offer',professional_id,service_id,action,expected_version,draft,created_at FROM service_offer_local WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY created_at,operation_id").map_err(storage_error)?;
    let rows = q
        .query_map(
            params![
                s.account_id.as_str(),
                s.user_id.as_str(),
                s.device_id.as_str()
            ],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get::<_, String>(6)?,
                ))
            },
        )
        .map_err(storage_error)?;

    rows.map(|row| {
        let (operation_id, kind, professional, service, action, version, draft) =
            row.map_err(storage_error)?;

        Ok(Intent {
            operation_id,
            kind,
            professional,
            service,
            action,
            version,
            draft: serde_json::from_str(&draft)
                .map_err(|_| ServicesError::new("INTERNAL_ERROR", "borrador local inválido"))?,
        })
    })
    .collect()
}

pub(super) fn state(
    c: &Connection,
    access: &AccountAccess,
    intent: &Intent,
) -> Result<LocalChange> {
    let mut change = LocalChange {
        operation_id: intent.operation_id.clone(),
        action: intent.action.clone(),
        expected_version: intent.version,
        sync_status: "review_required".into(),
        code: None,
    };

    if let Some(result) = SqliteOperationStore.operation_result(
        c,
        &access.context,
        &OperationId::try_from(intent.operation_id.clone())?,
    )? {
        change.sync_status = if result.review_code.is_some() {
            "review_required"
        } else {
            result.status.as_str()
        }
        .into();
        change.code = result
            .result
            .as_ref()
            .and_then(|v| v.get("error"))
            .and_then(|v| v.get("code"))
            .and_then(Value::as_str)
            .map(str::to_owned);

        if let Some(code) = result.review_code {
            change.code = Some(
                serde_json::to_value(code)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "UNKNOWN".into()),
            );
        }
    }

    Ok(change)
}

fn projected_version(c: &Connection, s: &OperationContext, intent: &Intent) -> Result<Option<i64>> {
    Ok(match intent.kind.as_str() {
        "service" => read_services(c, s)?
            .iter()
            .find(|v| {
                Some(v.service_id.as_str()) == intent.service.as_deref()
                    && v.professional_id == intent.professional
            })
            .map(|v| v.version),
        "service_offer" => read_offers(c, s)?
            .iter()
            .find(|v| {
                Some(v.service_id.as_str()) == intent.service.as_deref()
                    && v.professional_id == intent.professional
            })
            .map(|v| v.version),
        "professional" => read_professionals(c, s)?
            .iter()
            .find(|v| v.professional_id == intent.professional)
            .map(|v| v.version),
        _ => None,
    })
}

pub(super) fn resolve(c: &Connection, access: &AccountAccess) -> Result<()> {
    let s = &access.context;
    for intent in load(c, s)? {
        let Some(result) = SqliteOperationStore.operation_result(
            c,
            s,
            &OperationId::try_from(intent.operation_id.clone())?,
        )?
        else {
            continue;
        };
        if result.status != OperationStatus::Accepted || result.review_code.is_some() {
            continue;
        }
        let accepted = result
            .result
            .as_ref()
            .and_then(|r| r.get("entityVersion"))
            .and_then(Value::as_i64)
            .filter(|v| *v > 0);
        if let (Some(receipt), Some(projected)) = (accepted, projected_version(c, s, &intent)?)
            && projected >= receipt
        {
            // Retire only the component's confirmed provisional row. The immutable
            // operation and receipt remain owned by sync, including failed intents.
            let sql = if intent.kind == "service_offer" {
                "DELETE FROM service_offer_local WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND operation_id=?4"
            } else {
                "DELETE FROM service_local WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND operation_id=?4"
            };
            c.execute(
                sql,
                params![
                    s.account_id.as_str(),
                    s.user_id.as_str(),
                    s.device_id.as_str(),
                    intent.operation_id
                ],
            )
            .map_err(storage_error)?;
        }
    }
    Ok(())
}

fn row<T, D>(accepted: Option<T>) -> CatalogRow<T, D> {
    CatalogRow {
        accepted,
        provisional: None,
        is_provisional: false,
        local_changes: vec![],
    }
}
fn draft<T: serde::de::DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value)
        .map_err(|_| ServicesError::new("INTERNAL_ERROR", "borrador local inválido"))
}

pub(super) fn screen(c: &Connection, access: &AccountAccess) -> Result<CatalogScreen> {
    let s = &access.context;
    let mut services: BTreeMap<String, CatalogRow<ServiceView, ServiceDraft>> =
        read_services(c, s)?
            .into_iter()
            .map(|v| (v.service_id.clone(), row(Some(v))))
            .collect();
    let mut offers: BTreeMap<(String, String), CatalogRow<ServiceOfferView, ServiceOfferDraft>> =
        read_offers(c, s)?
            .into_iter()
            .map(|v| {
                (
                    (v.professional_id.clone(), v.service_id.clone()),
                    row(Some(v)),
                )
            })
            .collect();
    let mut professionals: BTreeMap<String, CatalogRow<ProfessionalView, ProfessionalDraft>> =
        read_professionals(c, s)?
            .into_iter()
            .map(|v| (v.professional_id.clone(), row(Some(v))))
            .collect();
    for intent in load(c, s)? {
        let state = state(c, access, &intent)?;
        match intent.kind.as_str() {
            "service" => {
                let d: ServiceDraft = draft(intent.draft)?;
                let entry = services
                    .entry(d.service_id.clone())
                    .or_insert_with(|| row(None));
                entry.provisional = Some(d);
                entry.is_provisional = true;
                entry.local_changes.push(state);
            }
            "service_offer" => {
                let d: ServiceOfferDraft = draft(intent.draft)?;
                let entry = offers
                    .entry((d.professional_id.clone(), d.service_id.clone()))
                    .or_insert_with(|| row(None));
                entry.provisional = Some(d);
                entry.is_provisional = true;
                entry.local_changes.push(state);
            }
            "professional" => {
                let d: ProfessionalDraft = draft(intent.draft)?;
                let entry = professionals
                    .entry(d.professional_id.clone())
                    .or_insert_with(|| row(None));
                entry.provisional = Some(d);
                entry.is_provisional = true;
                entry.local_changes.push(state);
            }
            _ => {
                return Err(ServicesError::new(
                    "INTERNAL_ERROR",
                    "tipo local de catálogo inválido",
                ));
            }
        }
    }
    Ok(CatalogScreen {
        services: services.into_values().collect(),
        offers: offers.into_values().collect(),
        professionals: professionals.into_values().collect(),
        can_manage: access.role == crate::components::identity::domain::AccessRole::Owner
            && [
                "CreateService",
                "UpdateService",
                "SetServiceStatus",
                "OfferService",
                "UpdateServiceOffer",
                "SetServiceOfferStatus",
                "UpdateProfessionalProfile",
            ]
            .iter()
            .all(|action| access.allows_offline(action)),
    })
}
