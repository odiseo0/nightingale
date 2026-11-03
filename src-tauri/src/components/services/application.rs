//! Native catalog use cases and accepted-data ports.
use super::domain::{
    DurationMinutes, OfferKey, ProfessionalId, ServiceId, ServiceOfferPatch, ServicePatch, Status,
    validate_name,
};
use crate::{
    components::identity::domain::{AccessRole, AccessStatus, AccountAccess},
    shared_kernel::{Clock, IdGenerator, NightingaleError, id::OperationContext},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ServicesError {
    pub code: String,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, ServicesError>;

impl ServicesError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl From<NightingaleError> for ServicesError {
    fn from(error: NightingaleError) -> Self {
        let code = serde_json::to_value(error.code)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| "INTERNAL_ERROR".into());

        if code == "INTERNAL_ERROR" {
            Self::new(&code, "datos locales de catálogo no disponibles")
        } else {
            Self::new(&code, &error.message)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceView {
    pub service_id: String,
    pub professional_id: String,
    pub name: String,
    pub default_duration_minutes: i64,
    pub status: String,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOfferView {
    pub professional_id: String,
    pub service_id: String,
    pub name: String,
    pub default_duration_minutes: i64,
    pub duration_minutes_override: Option<i64>,
    pub effective_duration_minutes: i64,
    pub service_status: String,
    pub status: String,
    pub version: i64,
    pub service_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfessionalView {
    pub professional_id: String,
    pub user_id: String,
    pub public_name: String,
    pub status: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDraft {
    pub service_id: String,
    pub professional_id: String,
    pub name: String,
    pub default_duration_minutes: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOfferDraft {
    pub professional_id: String,
    pub service_id: String,
    pub duration_minutes_override: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfessionalDraft {
    pub professional_id: String,
    pub public_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalChange {
    pub operation_id: String,
    pub action: String,
    pub expected_version: i64,
    pub sync_status: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogRow<T, D> {
    pub accepted: Option<T>,
    pub provisional: Option<D>,
    pub is_provisional: bool,
    pub local_changes: Vec<LocalChange>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogScreen {
    pub services: Vec<CatalogRow<ServiceView, ServiceDraft>>,
    pub offers: Vec<CatalogRow<ServiceOfferView, ServiceOfferDraft>>,
    pub professionals: Vec<CatalogRow<ProfessionalView, ProfessionalDraft>>,
    pub can_manage: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMutationResult {
    pub operation_id: String,
    pub entity_type: String,
    pub entity_key: Value,
    pub sync_status: String,
}

/// Future consumers see accepted values only; implementations own their storage.
pub trait CatalogQuery {
    fn list_services(&self, scope: &OperationContext) -> Result<Vec<ServiceView>>;

    fn service(&self, scope: &OperationContext, service: &ServiceId)
    -> Result<Option<ServiceView>>;

    fn list_offers(&self, scope: &OperationContext) -> Result<Vec<ServiceOfferView>>;

    fn offer(
        &self,
        scope: &OperationContext,
        professional: &ProfessionalId,
        service: &ServiceId,
    ) -> Result<Option<ServiceOfferView>>;

    fn list_professionals(&self, scope: &OperationContext) -> Result<Vec<ProfessionalView>>;

    fn professional(
        &self,
        scope: &OperationContext,
        professional: &ProfessionalId,
    ) -> Result<Option<ProfessionalView>>;
}

pub enum CatalogMutation {
    CreateService {
        professional: ProfessionalId,
        name: String,
        duration: DurationMinutes,
    },
    UpdateService {
        service: ServiceId,
        version: i64,
        patch: ServicePatch,
    },
    SetServiceStatus {
        service: ServiceId,
        version: i64,
        status: Status,
    },
    OfferService {
        key: OfferKey,
        adjustment: Option<DurationMinutes>,
    },
    UpdateServiceOffer {
        key: OfferKey,
        version: i64,
        patch: ServiceOfferPatch,
    },
    SetServiceOfferStatus {
        key: OfferKey,
        version: i64,
        status: Status,
    },
    UpdateProfessionalProfile {
        professional: ProfessionalId,
        version: i64,
        public_name: String,
    },
}
impl CatalogMutation {
    pub fn action(&self) -> &'static str {
        match self {
            Self::CreateService { .. } => "CreateService",
            Self::UpdateService { .. } => "UpdateService",
            Self::SetServiceStatus { .. } => "SetServiceStatus",
            Self::OfferService { .. } => "OfferService",
            Self::UpdateServiceOffer { .. } => "UpdateServiceOffer",
            Self::SetServiceOfferStatus { .. } => "SetServiceOfferStatus",
            Self::UpdateProfessionalProfile { .. } => "UpdateProfessionalProfile",
        }
    }

    pub fn version(&self) -> i64 {
        match self {
            Self::CreateService { .. } | Self::OfferService { .. } => 0,
            Self::UpdateService { version, .. }
            | Self::SetServiceStatus { version, .. }
            | Self::UpdateServiceOffer { version, .. }
            | Self::SetServiceOfferStatus { version, .. }
            | Self::UpdateProfessionalProfile { version, .. } => *version,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !matches!(self, Self::CreateService { .. } | Self::OfferService { .. })
            && self.version() < 1
        {
            return Err(ServicesError::new(
                "VALIDATION_ERROR",
                "se requiere una versión aceptada positiva",
            ));
        }

        match self {
            Self::CreateService { name, .. } => validate_name(name)?,
            Self::UpdateProfessionalProfile { public_name, .. } => validate_name(public_name)?,
            Self::UpdateService { patch, .. } => {
                ServicePatch::new(
                    patch.name.clone(),
                    patch.default_duration_minutes.map(DurationMinutes::get),
                )?;
            }
            _ => {}
        }

        Ok(())
    }
}

pub fn require_read(access: &AccountAccess) -> Result<()> {
    if access.status != AccessStatus::Active {
        return Err(ServicesError::new(
            "PERMISSION_DENIED",
            "la cuenta no tiene acceso activo",
        ));
    }

    Ok(())
}
pub fn require_write(access: &AccountAccess, action: &str) -> Result<()> {
    if access.role != AccessRole::Owner || !access.allows_offline(action) {
        return Err(ServicesError::new(
            "PERMISSION_DENIED",
            "la acción requiere permiso de propietario",
        ));
    }

    Ok(())
}

pub trait CatalogRepository: CatalogQuery {
    fn catalog(&mut self, access: &AccountAccess) -> Result<CatalogScreen>;

    /// Persist local intent and Outbox in the same transaction, after ownership checks.
    fn persist(
        &mut self,
        access: &AccountAccess,
        mutation: CatalogMutation,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> Result<LocalMutationResult>;
}

pub struct CatalogUseCases<'a, R, I, C> {
    pub repository: &'a mut R,
    pub access: &'a AccountAccess,
    pub ids: &'a I,
    pub clock: &'a C,
}
impl<R: CatalogRepository, I: IdGenerator, C: Clock> CatalogUseCases<'_, R, I, C> {
    pub fn list_catalog(&mut self) -> Result<CatalogScreen> {
        require_read(self.access)?;
        self.repository.catalog(self.access)
    }

    fn write(&mut self, mutation: CatalogMutation) -> Result<LocalMutationResult> {
        require_write(self.access, mutation.action())?;
        mutation.validate()?;
        self.repository
            .persist(self.access, mutation, self.ids, self.clock)
    }

    pub fn create_service(
        &mut self,
        professional: ProfessionalId,
        name: String,
        duration: DurationMinutes,
    ) -> Result<LocalMutationResult> {
        self.write(CatalogMutation::CreateService {
            professional,
            name,
            duration,
        })
    }

    pub fn update_service(
        &mut self,
        service: ServiceId,
        version: i64,
        patch: ServicePatch,
    ) -> Result<LocalMutationResult> {
        self.write(CatalogMutation::UpdateService {
            service,
            version,
            patch,
        })
    }

    pub fn set_service_status(
        &mut self,
        service: ServiceId,
        version: i64,
        status: Status,
    ) -> Result<LocalMutationResult> {
        self.write(CatalogMutation::SetServiceStatus {
            service,
            version,
            status,
        })
    }

    pub fn offer_service(
        &mut self,
        key: OfferKey,
        adjustment: Option<DurationMinutes>,
    ) -> Result<LocalMutationResult> {
        self.write(CatalogMutation::OfferService { key, adjustment })
    }

    pub fn update_service_offer(
        &mut self,
        key: OfferKey,
        version: i64,
        patch: ServiceOfferPatch,
    ) -> Result<LocalMutationResult> {
        self.write(CatalogMutation::UpdateServiceOffer {
            key,
            version,
            patch,
        })
    }

    pub fn set_service_offer_status(
        &mut self,
        key: OfferKey,
        version: i64,
        status: Status,
    ) -> Result<LocalMutationResult> {
        self.write(CatalogMutation::SetServiceOfferStatus {
            key,
            version,
            status,
        })
    }

    pub fn update_professional_profile(
        &mut self,
        professional: ProfessionalId,
        version: i64,
        public_name: String,
    ) -> Result<LocalMutationResult> {
        self.write(CatalogMutation::UpdateProfessionalProfile {
            professional,
            version,
            public_name,
        })
    }
}

#[cfg(test)]
#[path = "application_tests.rs"]
mod tests;
