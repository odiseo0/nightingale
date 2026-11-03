use super::*;
use crate::shared_kernel::{SystemClock, UuidV7Generator};
use serde_json::json;

pub(super) fn access() -> AccountAccess {
    AccountAccess {
        context: serde_json::from_value(json!({"accountId":"019a1234-0000-7000-8000-000000000001","userId":"019a1234-0000-7000-8000-000000000002","deviceId":"019a1234-0000-7000-8000-000000000003","permissionEpoch":1})).unwrap(),
        role: AccessRole::Owner,status: AccessStatus::Active,checked_at_millis:1,
        allowed_actions: ["CreateService","UpdateService","SetServiceStatus","OfferService","UpdateServiceOffer","SetServiceOfferStatus","UpdateProfessionalProfile"].map(str::to_owned).to_vec(),
    }
}
const ID: &str = "019a1234-0000-7000-8000-000000000004";

fn professional() -> ProfessionalId {
    ID.to_owned().try_into().unwrap()
}

fn service() -> ServiceId {
    ID.to_owned().try_into().unwrap()
}

fn key() -> OfferKey {
    OfferKey::new(ID.into(), ID.into()).unwrap()
}

#[derive(Default)]
struct Fake {
    writes: Vec<String>,
}

impl CatalogQuery for Fake {
    fn list_services(&self, _: &OperationContext) -> Result<Vec<ServiceView>> {
        Ok(vec![])
    }

    fn service(&self, _: &OperationContext, _: &ServiceId) -> Result<Option<ServiceView>> {
        Ok(None)
    }

    fn list_offers(&self, _: &OperationContext) -> Result<Vec<ServiceOfferView>> {
        Ok(vec![])
    }

    fn offer(
        &self,
        _: &OperationContext,
        _: &ProfessionalId,
        _: &ServiceId,
    ) -> Result<Option<ServiceOfferView>> {
        Ok(None)
    }

    fn list_professionals(&self, _: &OperationContext) -> Result<Vec<ProfessionalView>> {
        Ok(vec![])
    }

    fn professional(
        &self,
        _: &OperationContext,
        _: &ProfessionalId,
    ) -> Result<Option<ProfessionalView>> {
        Ok(None)
    }
}

impl CatalogRepository for Fake {
    fn catalog(&mut self, _: &AccountAccess) -> Result<CatalogScreen> {
        Ok(CatalogScreen::default())
    }

    fn persist(
        &mut self,
        _: &AccountAccess,
        mutation: CatalogMutation,
        ids: &impl IdGenerator,
        _: &impl Clock,
    ) -> Result<LocalMutationResult> {
        self.writes.push(mutation.action().into());
        Ok(LocalMutationResult {
            operation_id: ids.new_id().to_string(),
            entity_type: "service".into(),
            entity_key: json!(ID),
            sync_status: "pending".into(),
        })
    }
}

#[test]
fn all_seven_mutations_require_their_exact_offline_action() {
    let mut fake = Fake::default();
    let access = access();
    let mut cases = CatalogUseCases {
        repository: &mut fake,
        access: &access,
        ids: &UuidV7Generator,
        clock: &SystemClock,
    };
    assert!(cases.list_catalog().unwrap().services.is_empty());
    cases
        .create_service(
            professional(),
            "Synthetic".into(),
            DurationMinutes::new(30).unwrap(),
        )
        .unwrap();
    cases
        .update_service(service(), 1, ServicePatch::new(None, Some(15)).unwrap())
        .unwrap();
    cases
        .set_service_status(service(), 1, Status::Inactive)
        .unwrap();
    cases.offer_service(key(), None).unwrap();
    cases
        .update_service_offer(key(), 1, ServiceOfferPatch::new(Some(None)).unwrap())
        .unwrap();
    cases
        .set_service_offer_status(key(), 1, Status::Inactive)
        .unwrap();
    cases
        .update_professional_profile(professional(), 1, "Synthetic".into())
        .unwrap();
    assert_eq!(fake.writes, access.allowed_actions);
    for action in &access.allowed_actions {
        let mut denied = access.clone();
        denied.allowed_actions.retain(|allowed| allowed != action);
        assert_eq!(
            require_write(&denied, action).unwrap_err().code,
            "PERMISSION_DENIED"
        );
    }
}

#[test]
fn assistants_read_locally_but_cannot_write_even_with_incorrect_cached_grants() {
    let mut fake = Fake::default();
    let mut access = access();
    access.role = AccessRole::Assistant;
    {
        let mut cases = CatalogUseCases {
            repository: &mut fake,
            access: &access,
            ids: &UuidV7Generator,
            clock: &SystemClock,
        };
        assert!(cases.list_catalog().is_ok());
        assert_eq!(
            cases
                .create_service(
                    professional(),
                    "Synthetic".into(),
                    DurationMinutes::new(30).unwrap()
                )
                .unwrap_err()
                .code,
            "PERMISSION_DENIED"
        );
    }
    assert!(fake.writes.is_empty());
    access.status = AccessStatus::Revoked;
    assert!(require_read(&access).is_err());
}

#[test]
fn invalid_versions_and_empty_names_never_reach_storage() {
    let mut fake = Fake::default();
    let access = access();
    let mut cases = CatalogUseCases {
        repository: &mut fake,
        access: &access,
        ids: &UuidV7Generator,
        clock: &SystemClock,
    };
    assert!(
        cases
            .set_service_status(service(), 0, Status::Inactive)
            .is_err()
    );
    assert!(
        cases
            .update_professional_profile(professional(), 1, " ".into())
            .is_err()
    );
    assert!(
        cases
            .create_service(professional(), "".into(), DurationMinutes::new(30).unwrap())
            .is_err()
    );
    assert!(fake.writes.is_empty());
}
