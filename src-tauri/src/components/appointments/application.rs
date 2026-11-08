//! Public availability/occupancy contracts provided to other components.
//! This module deliberately contains no appointment write use cases: only the
//! accepted occupancy projection and its read port live here for now.
use crate::shared_kernel::{NightingaleError, Result};
use jiff::Timestamp;

/// An accepted occupancy interval of one appointment. Half-open `[startsAt,endsAt)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccupancyInterval {
    pub appointment_id: String,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub status: String,
}

impl OccupancyInterval {
    pub fn new(
        appointment_id: String,
        starts_at: Timestamp,
        ends_at: Timestamp,
        status: &str,
    ) -> Result<Self> {
        if starts_at >= ends_at {
            return Err(NightingaleError::validation(
                "la ocupación exige inicio menor que fin",
            ));
        }

        match status {
            "scheduled" | "confirmed" | "completed" | "no_show" => Ok(Self {
                appointment_id,
                starts_at,
                ends_at,
                status: status.into(),
            }),
            _ => Err(NightingaleError::validation(
                "estado de ocupación inválido; cancelled no ocupa",
            )),
        }
    }
}

/// Read-only occupancy port owned by appointments. Availability subtracts the
/// intervals returned here; provisional occupancy from local intentions enters
/// through the same port without changing consumers.
pub trait ScheduleOccupancyQuery {
    /// Accepted occupancy of one professional intersecting `[from,until)`.
    /// Contiguous appointments (end == next start) are both returned and are
    /// not treated as overlapping.
    fn list_occupancy(
        &self,
        professional_id: &str,
        from: Timestamp,
        until: Timestamp,
    ) -> Result<Vec<OccupancyInterval>>;
}

use super::domain::{
    ActorKind, AppointmentId, AppointmentInterval, AppointmentStatus, ChangeReason, Snapshots,
};
use crate::components::identity::domain::AccountAccess;
use crate::shared_kernel::{Clock, IdGenerator};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Versioned base for an edit: creation expects version 0; edits use either a
/// positive accepted version or the last local operation as base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditBase {
    Accepted { version: i64 },
    LocalOperation { operation_id: String },
}

impl EditBase {
    pub fn accepted(version: i64) -> Self {
        Self::Accepted { version }
    }

    pub fn local(operation_id: String) -> Self {
        Self::LocalOperation { operation_id }
    }
}

/// Business state of one appointment as shown by merged reads. Snapshots are
/// captured at creation and never recalculated locally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentState {
    pub appointment_id: String,
    pub patient_id: String,
    pub professional_id: String,
    pub service_id: String,
    pub responsible_contact_id: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub status: String,
    pub confirmed_at: Option<String>,
    pub service_name: String,
    pub offered_duration_minutes: i64,
    pub professional_name: String,
    pub location_name: String,
    pub location_address: String,
    pub location_country: String,
    pub location_state: String,
    pub location_city: String,
    pub time_zone: String,
}

impl AppointmentState {
    pub fn snapshots(&self) -> Snapshots {
        Snapshots {
            service_name: self.service_name.clone(),
            offered_duration_minutes: self.offered_duration_minutes,
            professional_name: self.professional_name.clone(),
            location_name: self.location_name.clone(),
            location_address: self.location_address.clone(),
            location_country: self.location_country.clone(),
            location_state: self.location_state.clone(),
            location_city: self.location_city.clone(),
            time_zone: self.time_zone.clone(),
        }
    }

    pub fn status(
        &self,
    ) -> std::result::Result<AppointmentStatus, super::domain::AppointmentsError> {
        AppointmentStatus::try_from(self.status.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accepted<T> {
    pub state: T,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provisional<T> {
    pub state: T,
    pub operation_id: String,
    pub expected_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationView {
    pub operation_id: String,
    pub depends_on: Vec<String>,
    pub status: String,
    pub error: Option<String>,
}

/// Merged row: accepted view, provisional draft and pending operations stay separate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRow<T> {
    pub accepted: Option<Accepted<T>>,
    pub provisional: Option<Provisional<T>>,
    pub operations: Vec<OperationView>,
}

impl<T> EntityRow<T> {
    pub fn visible(&self) -> Option<&T> {
        self.provisional
            .as_ref()
            .map(|v| &v.state)
            .or_else(|| self.accepted.as_ref().map(|v| &v.state))
    }
}

/// Reference data captured for a new appointment. Booking validation and
/// snapshot assembly from the ports arrive in T08; the mutation already
/// carries the full state so drafts stay complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAppointment {
    pub patient_id: String,
    pub professional_id: String,
    pub service_id: String,
    pub responsible_contact_id: Option<String>,
    pub interval: AppointmentInterval,
    pub snapshots: Snapshots,
}

/// The six contract actions as domain intents. Semantic checks (booking rules,
/// transition table, clocks) are layered per task; the payload shape and the
/// base/dependency plumbing are fixed here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppointmentMutation {
    Create {
        data: Box<NewAppointment>,
        /// Booking references (patient/contact/service/offer/agenda operations)
        /// the new operation declares, up to eight, without duplicates.
        depends_on: Vec<String>,
    },
    Confirm {
        appointment: AppointmentId,
        base: EditBase,
    },
    Reschedule {
        appointment: AppointmentId,
        base: EditBase,
        interval: AppointmentInterval,
    },
    Cancel {
        appointment: AppointmentId,
        base: EditBase,
        reason: Option<ChangeReason>,
    },
    Complete {
        appointment: AppointmentId,
        base: EditBase,
        reason: Option<ChangeReason>,
    },
    NoShow {
        appointment: AppointmentId,
        base: EditBase,
        reason: Option<ChangeReason>,
    },
}

impl AppointmentMutation {
    pub fn action(&self) -> &'static str {
        match self {
            Self::Create { .. } => "CreateAppointment",
            Self::Confirm { .. } => "ConfirmAppointment",
            Self::Reschedule { .. } => "RescheduleAppointment",
            Self::Cancel { .. } => "CancelAppointment",
            Self::Complete { .. } => "CompleteAppointment",
            Self::NoShow { .. } => "MarkAppointmentNoShow",
        }
    }

    pub fn appointment_id(&self) -> Option<&AppointmentId> {
        match self {
            Self::Create { .. } => None,
            Self::Confirm { appointment, .. }
            | Self::Reschedule { appointment, .. }
            | Self::Cancel { appointment, .. }
            | Self::Complete { appointment, .. }
            | Self::NoShow { appointment, .. } => Some(appointment),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMutationResult {
    pub operation_id: String,
    pub entity_type: String,
    pub entity_key: Value,
    pub expected_version: i64,
    pub depends_on: Vec<String>,
    pub provisional: Value,
    pub provisional_operation_id: String,
}

pub type AppResult<T> = std::result::Result<T, super::domain::AppointmentsError>;

/// Write-side repository for appointment intentions. Implementations persist
/// the provisional draft and the Outbox record in one transaction.
pub trait AppointmentRepository {
    fn appointment_row(
        &mut self,
        access: &AccountAccess,
        appointment: &AppointmentId,
    ) -> AppResult<EntityRow<AppointmentState>>;

    fn persist(
        &mut self,
        access: &AccountAccess,
        mutation: AppointmentMutation,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> AppResult<LocalMutationResult>;
}

/// Requires an active account; appointment writes are staff actions.
pub fn require_active(access: &AccountAccess) -> AppResult<()> {
    if access.status != crate::components::identity::domain::AccessStatus::Active {
        return Err(super::domain::AppointmentsError::new(
            "PERMISSION_DENIED",
            "la cuenta no tiene acceso activo",
        ));
    }

    Ok(())
}

#[allow(dead_code)]
fn _actor(_: ActorKind) {}

use crate::components::account::domain::AccountView;
use crate::components::patients::application::BookingPatient;
use crate::components::services::application::BookingOffer;

/// Command input for creating an appointment. Reference edits after creation
/// are not supported: there is no free appointment patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAppointmentInput {
    pub patient_id: String,
    pub professional_id: String,
    pub service_id: String,
    pub responsible_contact_id: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
}

impl CreateAppointmentInput {
    pub fn interval(&self) -> AppResult<AppointmentInterval> {
        let starts = super::domain::MinuteInstant::parse(&self.starts_at)?;
        let ends = super::domain::MinuteInstant::parse(&self.ends_at)?;
        AppointmentInterval::new(starts, ends)
    }
}

/// Validates a new appointment against the accepted account view, the merged
/// booking references and a local availability evaluation. The local check
/// never proves the shared reservation: API revalidates everything.
pub fn validate_new_appointment(
    account: &AccountView,
    patient: &BookingPatient,
    offer: &BookingOffer,
    input: &CreateAppointmentInput,
    evaluation: &crate::components::scheduling::application::AvailabilityEvaluation,
) -> AppResult<NewAppointment> {
    use crate::components::account::domain::AccountStatus;

    if account.status != AccountStatus::Active {
        return Err(super::domain::AppointmentsError::new(
            "PERMISSION_DENIED",
            "la cuenta no está activa",
        ));
    }

    let interval = input.interval()?;
    let zone = jiff::tz::TimeZone::get(&account.time_zone).map_err(|_| {
        super::domain::AppointmentsError::validation("zona horaria de la cuenta inválida")
    })?;
    let local_date = interval.starts_at.timestamp().to_zoned(zone).date();

    validate_patient(patient, input, &local_date)?;

    if offer.professional_id != input.professional_id || offer.service_id != input.service_id {
        return Err(super::domain::AppointmentsError::new(
            "NOT_FOUND",
            "la oferta no corresponde al profesional y servicio elegidos",
        ));
    }

    // Coverage stays explicit: insufficient evidence never becomes "free".
    if evaluation.coverage == "insufficient_coverage" {
        return Err(super::domain::AppointmentsError::new(
            "INSUFFICIENT_COVERAGE",
            "la ventana de ocupación descargada no cubre el intervalo; sincronice antes de reservar",
        ));
    }

    let start = interval.starts_at.timestamp().to_string();
    let end = interval.ends_at.timestamp().to_string();
    let fits = evaluation
        .intervals
        .iter()
        .any(|available| available.starts_at <= start && end <= available.ends_at);

    if !fits {
        return Err(super::domain::AppointmentsError::new(
            "SLOT_UNAVAILABLE",
            "el intervalo queda fuera del horario disponible o dentro de una ocupación",
        ));
    }

    let snapshots = Snapshots::new(
        offer.service_name.clone(),
        offer.effective_duration_minutes,
        offer.professional_name.clone(),
        account.location_name.clone(),
        account.location_address.clone(),
        account.location_country.clone(),
        account.location_state.clone(),
        account.location_city.clone(),
        account.time_zone.clone(),
    )?;

    Ok(NewAppointment {
        patient_id: input.patient_id.clone(),
        professional_id: input.professional_id.clone(),
        service_id: input.service_id.clone(),
        responsible_contact_id: input.responsible_contact_id.clone(),
        interval,
        snapshots,
    })
}

fn validate_patient(
    patient: &BookingPatient,
    input: &CreateAppointmentInput,
    local_date: &jiff::civil::Date,
) -> AppResult<()> {
    use super::domain::{AppointmentsError, age_on};

    let profile = patient.profile.visible().ok_or_else(|| {
        AppointmentsError::new("NOT_FOUND", "el paciente no tiene estado visible")
    })?;

    if profile.status != "active" {
        return Err(AppointmentsError::validation(
            "el paciente debe estar activo para reservar",
        ));
    }

    if profile.completion_status != "complete" {
        return Err(AppointmentsError::validation(
            "la ficha del paciente está incompleta",
        ));
    }

    let name_ok = profile
        .full_name
        .as_deref()
        .map(|name| !name.trim().is_empty())
        .unwrap_or(false);

    if !name_ok {
        return Err(AppointmentsError::validation(
            "el paciente requiere un nombre para reservar",
        ));
    }

    let responsible = selected_responsible(patient, input, local_date)?;

    match profile.identification_basis.as_deref() {
        Some("own_document") => {
            let document_ok = patient.identifiers.iter().any(|row| {
                row.visible()
                    .map(|id| {
                        id.verification_status == "declared" || id.verification_status == "verified"
                    })
                    .unwrap_or(false)
            });
            let own_phone = profile.phone_code.is_some() && profile.phone.is_some();

            if !document_ok {
                return Err(AppointmentsError::validation(
                    "el paciente requiere un documento declarado o verificado",
                ));
            }

            if !own_phone && responsible.is_none() {
                return Err(AppointmentsError::validation(
                    "se requiere teléfono propio o un responsable con teléfono",
                ));
            }
        }
        Some("minor_without_document") => {
            let birth = profile.birth_date.as_deref().ok_or_else(|| {
                AppointmentsError::validation("el menor requiere fecha de nacimiento")
            })?;
            let birth_date = super::domain::parse_local_date(birth)?;

            if age_on(birth_date, *local_date) >= 18 {
                return Err(AppointmentsError::validation(
                    "el paciente es mayor de edad en la fecha local de la cita",
                ));
            }

            if responsible.is_none() {
                return Err(AppointmentsError::validation(
                    "el menor requiere un responsable vigente con teléfono",
                ));
            }
        }
        _ => {
            return Err(AppointmentsError::validation(
                "la base de identificación del paciente no permite reservar",
            ));
        }
    }

    Ok(())
}

/// The selected responsible link must be active, marked responsible, valid on
/// the local appointment date and carry a phone. Representation/tutelage is
/// never inferred or verified here.
fn selected_responsible<'a>(
    patient: &'a BookingPatient,
    input: &CreateAppointmentInput,
    local_date: &jiff::civil::Date,
) -> AppResult<Option<&'a crate::components::patients::application::RelationshipView>> {
    use super::domain::AppointmentsError;

    let Some(selected) = &input.responsible_contact_id else {
        return Ok(None);
    };

    let row = patient
        .responsible_contacts
        .iter()
        .find(|row| {
            row.visible()
                .map(|view| view.relationship.contact_person_id == *selected)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            AppointmentsError::validation("el contacto elegido no es un responsable del paciente")
        })?;
    let view = row.visible().unwrap();

    let valid_from = super::domain::parse_local_date(&view.relationship.valid_from)?;

    if valid_from > *local_date {
        return Err(AppointmentsError::validation(
            "el vínculo del responsable no está vigente en la fecha de la cita",
        ));
    }

    if let Some(until) = &view.relationship.valid_until {
        let valid_until = super::domain::parse_local_date(until)?;

        if valid_until < *local_date {
            return Err(AppointmentsError::validation(
                "el vínculo del responsable no está vigente en la fecha de la cita",
            ));
        }
    }

    if view.contact_phone.is_none() {
        return Err(AppointmentsError::validation(
            "el responsable elegido no tiene teléfono",
        ));
    }

    Ok(Some(view))
}
