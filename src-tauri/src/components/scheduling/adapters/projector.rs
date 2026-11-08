//! Accepted-view projector for scheduling agenda entities.
//! It translates remote `schedule_rule`/`schedule_exception`/`schedule_block`
//! changes into this scope's views: validates scope, keys, times, states and
//! versions, ignores foreign types, performs no HTTP and never touches local
//! intentions.
use super::super::domain::{ExceptionValue, parse_local_date};
use crate::components::sync::adapters::remote_models::{
    AccountResponseStatus, ProjectionId, RemoteState, ScheduleBlockResponseReasonCode,
    ScheduleExceptionResponseKind,
};
use crate::components::sync::remote::{EntityType, RemoteChange, RemoteProjector};
use crate::shared_kernel::{NightingaleError, Result, id::OperationContext};
use jiff::Timestamp;
use rusqlite::{Transaction, params};
use std::str::FromStr;

pub struct SchedulingProjector;

fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("proyección local de agenda no disponible")
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

fn instant(value: &str) -> Result<Timestamp> {
    Timestamp::from_str(value).map_err(|_| NightingaleError::validation("instante remoto inválido"))
}

impl RemoteProjector for SchedulingProjector {
    fn reset(&self, tx: &Transaction, s: &OperationContext) -> Result<()> {
        // Reset only clears accepted views; local intentions always survive.
        for table in [
            "schedule_rule_view",
            "schedule_exception_view",
            "schedule_block_view",
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
                RemoteState::ScheduleRule(v) => {
                    if id(&v.account_id) != s.account_id.as_str() {
                        return Err(NightingaleError::validation("regla remota fuera de ámbito"));
                    }

                    // Domain validation: ISO weekday, minute bounds, inclusive validity.
                    let professional = id(&v.professional_id);
                    let _ = super::super::domain::RuleValue::new(
                        professional.as_str().to_owned().try_into()?,
                        v.iso_weekday,
                        v.local_start_minute,
                        v.local_end_minute,
                        parse_local_date(&v.valid_from).map_err(|_| {
                            NightingaleError::validation("vigencia remota inválida")
                        })?,
                        v.valid_until
                            .as_deref()
                            .map(parse_local_date)
                            .transpose()
                            .map_err(|_| {
                                NightingaleError::validation("vigencia remota inválida")
                            })?,
                    )
                    .map_err(|_| NightingaleError::validation("regla remota inválida"))?;

                    tx.execute(
                        "INSERT INTO schedule_rule_view (account_id,user_id,device_id,rule_id,professional_id,iso_weekday,local_start_minute,local_end_minute,valid_from,valid_until,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
                        ON CONFLICT(account_id,user_id,device_id,rule_id) DO UPDATE SET professional_id=excluded.professional_id,iso_weekday=excluded.iso_weekday,local_start_minute=excluded.local_start_minute,local_end_minute=excluded.local_end_minute,valid_from=excluded.valid_from,valid_until=excluded.valid_until,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version
                        WHERE excluded.version>schedule_rule_view.version",
                        params![
                            s.account_id.as_str(),
                            s.user_id.as_str(),
                            s.device_id.as_str(),
                            id(&v.id),
                            professional,
                            v.iso_weekday,
                            v.local_start_minute,
                            v.local_end_minute,
                            v.valid_from,
                            v.valid_until,
                            status(&v.status),
                            v.created_at,
                            v.updated_at,
                            v.version
                        ],
                    )
                    .map_err(storage_error)?;
                }
                RemoteState::ScheduleException(v) => {
                    if id(&v.account_id) != s.account_id.as_str() {
                        return Err(NightingaleError::validation(
                            "excepción remota fuera de ámbito",
                        ));
                    }

                    let professional = id(&v.professional_id);
                    let _ = ExceptionValue::new(
                        professional.as_str().to_owned().try_into()?,
                        parse_local_date(&v.local_date)
                            .map_err(|_| NightingaleError::validation("fecha remota inválida"))?,
                        match v.kind {
                            ScheduleExceptionResponseKind::Closed => {
                                super::super::domain::ExceptionKind::Closed
                            }
                            ScheduleExceptionResponseKind::Replacement => {
                                super::super::domain::ExceptionKind::Replacement
                            }
                        },
                        v.local_start_minute,
                        v.local_end_minute,
                    )
                    .map_err(|_| NightingaleError::validation("excepción remota inválida"))?;

                    tx.execute(
                        "INSERT INTO schedule_exception_view (account_id,user_id,device_id,exception_id,professional_id,local_date,kind,local_start_minute,local_end_minute,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
                        ON CONFLICT(account_id,user_id,device_id,exception_id) DO UPDATE SET professional_id=excluded.professional_id,local_date=excluded.local_date,kind=excluded.kind,local_start_minute=excluded.local_start_minute,local_end_minute=excluded.local_end_minute,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version
                        WHERE excluded.version>schedule_exception_view.version",
                        params![
                            s.account_id.as_str(),
                            s.user_id.as_str(),
                            s.device_id.as_str(),
                            id(&v.id),
                            professional,
                            v.local_date,
                            match v.kind {
                                ScheduleExceptionResponseKind::Closed => "closed",
                                ScheduleExceptionResponseKind::Replacement => "replacement",
                            },
                            v.local_start_minute,
                            v.local_end_minute,
                            status(&v.status),
                            v.created_at,
                            v.updated_at,
                            v.version
                        ],
                    )
                    .map_err(storage_error)?;
                }
                RemoteState::ScheduleBlock(v) => {
                    // Global blocks arrive with null accountId; scoped blocks
                    // must belong to this account. The reason may be hidden by
                    // audience, so it stays nullable end to end.
                    if v.account_id
                        .as_ref()
                        .is_some_and(|account| id(account) != s.account_id.as_str())
                    {
                        return Err(NightingaleError::validation(
                            "bloqueo remoto fuera de ámbito",
                        ));
                    }

                    if instant(&v.starts_at)? >= instant(&v.ends_at)? {
                        return Err(NightingaleError::validation(
                            "intervalo remoto de bloqueo inválido",
                        ));
                    }

                    let reason = v
                        .reason_code
                        .as_ref()
                        .map(|reason| match reason {
                            ScheduleBlockResponseReasonCode::Vacation => "vacation",
                            ScheduleBlockResponseReasonCode::Administrative => "administrative",
                            ScheduleBlockResponseReasonCode::Unavailable => "unavailable",
                        })
                        .map(str::to_owned);

                    tx.execute(
                        "INSERT INTO schedule_block_view (account_id,user_id,device_id,block_id,professional_id,block_account_id,starts_at,ends_at,reason_code,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
                        ON CONFLICT(account_id,user_id,device_id,block_id) DO UPDATE SET professional_id=excluded.professional_id,block_account_id=excluded.block_account_id,starts_at=excluded.starts_at,ends_at=excluded.ends_at,reason_code=excluded.reason_code,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version
                        WHERE excluded.version>schedule_block_view.version",
                        params![
                            s.account_id.as_str(),
                            s.user_id.as_str(),
                            s.device_id.as_str(),
                            id(&v.id),
                            id(&v.professional_id),
                            v.account_id.as_ref().map(id),
                            v.starts_at,
                            v.ends_at,
                            reason,
                            status(&v.status),
                            v.created_at,
                            v.updated_at,
                            v.version
                        ],
                    )
                    .map_err(storage_error)?;
                }
                _ => {}
            },
            RemoteChange::Remove {
                entity_type,
                key: entity_key,
            } => match entity_type {
                EntityType::ScheduleRule => {
                    let rule = key(entity_key)?;
                    tx.execute(
                        "DELETE FROM schedule_rule_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND rule_id=?4",
                        params![s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(), rule],
                    )
                    .map_err(storage_error)?;
                }
                EntityType::ScheduleException => {
                    let exception = key(entity_key)?;
                    tx.execute(
                        "DELETE FROM schedule_exception_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND exception_id=?4",
                        params![s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(), exception],
                    )
                    .map_err(storage_error)?;
                }
                EntityType::ScheduleBlock => {
                    let block = key(entity_key)?;
                    tx.execute(
                        "DELETE FROM schedule_block_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND block_id=?4",
                        params![s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(), block],
                    )
                    .map_err(storage_error)?;
                }
                _ => {}
            },
        }

        Ok(())
    }
}
