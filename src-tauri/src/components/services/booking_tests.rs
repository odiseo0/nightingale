//! CatalogBookingQuery: booking offers over accepted and chainable provisional catalog data.
use super::{
    adapters::outbound::SqliteCatalogRepository,
    application::{CatalogBookingQuery, CatalogMutation, CatalogReference, CatalogRepository},
    domain::{DurationMinutes, OfferKey, ProfessionalId, ServiceId},
    test_support,
};
use crate::shared_kernel::{Clock, IdGenerator};

struct Ids(u64);

impl Default for Ids {
    fn default() -> Self {
        Self(100)
    }
}

impl IdGenerator for Ids {
    fn new_id(&self) -> uuid::Uuid {
        format!("019a1234-0000-7000-8000-{:012}", self.0 + 1)
            .parse()
            .unwrap()
    }
}

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> std::time::SystemTime {
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(1_793_829_600_000)
    }
}

fn professional() -> ProfessionalId {
    test_support::PROFESSIONAL.to_owned().try_into().unwrap()
}

fn service() -> ServiceId {
    test_support::SERVICE.to_owned().try_into().unwrap()
}

fn booking(connection: &mut rusqlite::Connection) -> Option<super::application::BookingOffer> {
    let access = test_support::access();
    let mut repository = SqliteCatalogRepository { connection };
    repository
        .booking_offer(&access, &professional(), &service())
        .unwrap()
}

#[test]
fn accepted_offer_resolves_with_both_versions_and_effective_duration() {
    let mut connection = test_support::database();
    let scope = test_support::access().context;
    test_support::seed_service(&connection, &scope, test_support::SERVICE, 1);
    test_support::seed_offer(&connection, &scope);

    let view = booking(&mut connection).unwrap();
    assert_eq!(
        view.reference,
        CatalogReference::Accepted {
            version: 1,
            service_version: 1
        }
    );
    assert_eq!(view.service_name, "Synthetic");
    assert_eq!(view.professional_name, "Synthetic");
    assert_eq!(view.effective_duration_minutes, 30);
    assert_eq!(view.status, "active");
    assert_eq!(view.service_status, "active");

    // Servicio sin oferta: nada resoluble.
    let missing: ServiceId = "019a1234-0000-7000-8000-000000000099"
        .to_owned()
        .try_into()
        .unwrap();
    let access = test_support::access();
    let mut repository = SqliteCatalogRepository {
        connection: &mut connection,
    };
    assert!(
        repository
            .booking_offer(&access, &professional(), &missing)
            .unwrap()
            .is_none()
    );
}

#[test]
fn provisional_service_and_offer_created_offline_resolve_for_booking() {
    let mut connection = test_support::database();
    let access = test_support::access();
    let ids = Ids::default();

    let created = {
        let mut repository = SqliteCatalogRepository {
            connection: &mut connection,
        };
        repository
            .persist(
                &access,
                CatalogMutation::CreateService {
                    professional: professional(),
                    name: "Consulta offline".into(),
                    duration: DurationMinutes::new(40).unwrap(),
                },
                &ids,
                &FixedClock,
            )
            .unwrap()
    };
    assert_eq!(created.entity_type, "service");

    let offered = {
        let mut repository = SqliteCatalogRepository {
            connection: &mut connection,
        };
        repository
            .persist(
                &access,
                CatalogMutation::OfferService {
                    key: OfferKey::new(
                        test_support::PROFESSIONAL.into(),
                        created.entity_key.as_str().unwrap().to_owned(),
                    )
                    .unwrap(),
                    adjustment: Some(DurationMinutes::new(45).unwrap()),
                },
                &Ids(200),
                &FixedClock,
            )
            .unwrap()
    };
    assert_eq!(offered.entity_type, "service_offer");

    // La oferta offline usa el servicio offline como nombre.
    let view = {
        let mut repository = SqliteCatalogRepository {
            connection: &mut connection,
        };
        repository
            .booking_offer(&access, &professional(), &{
                created
                    .entity_key
                    .as_str()
                    .unwrap()
                    .to_owned()
                    .try_into()
                    .unwrap()
            })
            .unwrap()
            .unwrap()
    };
    let CatalogReference::Provisional { operation_id } = view.reference else {
        panic!("se esperaba una referencia provisional");
    };
    assert_eq!(operation_id, offered.operation_id);
    assert_eq!(view.service_name, "Consulta offline");
    assert_eq!(view.effective_duration_minutes, 45);
    assert_eq!(view.status, "active");
    assert_eq!(view.service_status, "active");
}

#[test]
fn conflicted_offer_is_excluded_from_booking() {
    let mut connection = test_support::database();
    let access = test_support::access();

    // Service and offer created offline; dispatch returns conflict.
    let created = {
        let mut repository = SqliteCatalogRepository {
            connection: &mut connection,
        };
        repository
            .persist(
                &access,
                CatalogMutation::CreateService {
                    professional: professional(),
                    name: "Servicio en conflicto".into(),
                    duration: DurationMinutes::new(30).unwrap(),
                },
                &Ids(300),
                &FixedClock,
            )
            .unwrap()
    };
    let service: ServiceId = created
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap();
    let offered = {
        let mut repository = SqliteCatalogRepository {
            connection: &mut connection,
        };
        repository
            .persist(
                &access,
                CatalogMutation::OfferService {
                    key: OfferKey::new(test_support::PROFESSIONAL.into(), service.as_str().into())
                        .unwrap(),
                    adjustment: None,
                },
                &Ids(301),
                &FixedClock,
            )
            .unwrap()
    };

    {
        let mut repository = SqliteCatalogRepository {
            connection: &mut connection,
        };
        let view = repository
            .booking_offer(&access, &professional(), &service)
            .unwrap();
        assert!(view.is_some());
    }

    test_support::receipt(
        &connection,
        &offered.operation_id,
        "conflict",
        None,
        Some("VERSION_CONFLICT"),
    );

    let mut repository = SqliteCatalogRepository {
        connection: &mut connection,
    };
    assert!(
        repository
            .booking_offer(&access, &professional(), &service)
            .unwrap()
            .is_none()
    );
}

#[test]
fn inactive_offer_or_service_is_excluded_from_booking() {
    let mut connection = test_support::database();
    let scope = test_support::access().context;
    test_support::seed_service(&connection, &scope, test_support::SERVICE, 1);
    test_support::seed_offer(&connection, &scope);
    connection
        .execute(
            "UPDATE service_offer_view SET status='inactive' WHERE service_id=?1",
            rusqlite::params![test_support::SERVICE],
        )
        .unwrap();

    assert!(booking(&mut connection).is_none());

    connection
        .execute(
            "UPDATE service_offer_view SET status='active', service_status='inactive' WHERE service_id=?1",
            rusqlite::params![test_support::SERVICE],
        )
        .unwrap();
    assert!(booking(&mut connection).is_none());
}
