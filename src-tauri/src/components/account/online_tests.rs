use super::*;
use crate::{
    components::{
        account::{
            application::{self, MutationStatus},
            domain::{AccountDetailsPatch, AccountPolicy, AccountStatus},
        },
        identity::domain::{AccessRole, AccessStatus},
    },
    platform::{config::APIConfig, test_api::TestApi},
    shared_kernel::UuidV7Generator,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::Duration};

const ACCOUNT: &str = "019a1234-0000-7000-8000-000000000001";
const USER: &str = "019a1234-0000-7000-8000-000000000002";
const DEVICE: &str = "019a1234-0000-7000-8000-000000000003";
const CORRELATION: &str = "019a1234-0000-7000-8000-000000000004";

fn access() -> AccountAccess {
    AccountAccess {
        context: serde_json::from_value(
            json!({"accountId":ACCOUNT,"userId":USER,"deviceId":DEVICE,"permissionEpoch":1}),
        )
        .unwrap(),
        role: AccessRole::Owner,
        status: AccessStatus::Active,
        allowed_actions: vec![
            "UpdateAccountDetails".into(),
            "UpdateAccountPolicy".into(),
            "SetAccountStatus".into(),
        ],
        checked_at_millis: 1,
    }
}

fn access_response() -> Value {
    json!({"accountId":ACCOUNT,"userId":USER,"deviceId":DEVICE,"permissionEpoch":1,"role":"owner","status":"active","allowedActions":["UpdateAccountDetails","UpdateAccountPolicy","SetAccountStatus"]})
}

fn operation(request: &str) -> Value {
    serde_json::from_str::<Value>(request.split("\r\n\r\n").nth(1).unwrap()).unwrap()["operations"]
        [0]
    .clone()
}

fn reply(request: &str, status: &str, code: Option<&str>) -> Value {
    let operation = operation(request);
    let error = code.map(|code| json!({"code":code,"correlationId":CORRELATION}));
    json!({"correlationId":CORRELATION,"results":[{"operationId":operation["operationId"],"status":status,"entityVersion":2,"state":null,"error":error}]})
}

#[test]
fn all_three_account_changes_refresh_access_and_send_without_local_writes() {
    tauri::async_runtime::block_on(async {
        let server = TestApi::new(6, |request| {
            if request.starts_with("GET ") {
                (200, access_response())
            } else {
                (200, reply(request, "accepted", None))
            }
        });
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let token = Secret::new("synthetic-secret".into()).unwrap();
        let access = access();
        let remote = OnlineAccount {
            http: &http,
            token: &token,
            scope: &access.context,
        };
        let connection = crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap();
        let patch =
            AccountDetailsPatch::new(BTreeMap::from([("displayName".into(), "Synthetic".into())]))
                .unwrap();
        let details =
            application::update_account_details(&remote, &access, 1, patch, &UuidV7Generator)
                .await
                .unwrap();
        let policy = application::update_account_policy(
            &remote,
            &access,
            1,
            AccountPolicy::new(0).unwrap(),
            &UuidV7Generator,
        )
        .await
        .unwrap();
        let status = application::set_account_status(
            &remote,
            &access,
            1,
            AccountStatus::Inactive,
            &UuidV7Generator,
        )
        .await
        .unwrap();
        for result in [&details, &policy, &status] {
            assert_eq!(result.status, MutationStatus::Accepted);
        }
        assert_ne!(details.operation_id, policy.operation_id);
        for table in ["sync_outbox", "account_view"] {
            assert_eq!(
                connection
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
        let requests = server.finish();
        for pair in requests.as_chunks::<2>().0 {
            assert!(pair[0].starts_with(&format!("GET /v1/accounts/{ACCOUNT}/access ")));
            let operation = operation(&pair[1]);
            assert_eq!(operation["expectedVersion"], 1);
            assert_eq!(operation["entityKey"], ACCOUNT);
            assert!(operation.get("accountId").is_none());
            assert!(operation.get("userId").is_none());
            assert!(operation.get("deviceId").is_none());
        }
        assert_eq!(
            operation(&requests[1])["payload"],
            json!({"displayName":"Synthetic"})
        );
        assert_eq!(
            operation(&requests[3])["payload"],
            json!({"noShowGraceMinutes":0})
        );
        assert_eq!(
            operation(&requests[5])["payload"],
            json!({"status":"inactive"})
        );
    });
}

#[test]
fn account_timeout_retries_the_same_complete_operation() {
    tauri::async_runtime::block_on(async {
        let mut posts = 0;
        let server = TestApi::new(3, move |request| {
            if request.starts_with("GET ") {
                return (200, access_response());
            }
            posts += 1;
            if posts == 1 {
                std::thread::sleep(Duration::from_millis(300));
                (0, Value::Null)
            } else {
                (200, reply(request, "accepted", None))
            }
        });
        let http = APITransport::with_timeout(
            APIConfig::parse(&server.url).unwrap(),
            Duration::from_millis(200),
        )
        .unwrap();
        let token = Secret::new("synthetic-secret".into()).unwrap();
        let access = access();
        let remote = OnlineAccount {
            http: &http,
            token: &token,
            scope: &access.context,
        };
        let result = application::update_account_policy(
            &remote,
            &access,
            1,
            AccountPolicy::new(15).unwrap(),
            &UuidV7Generator,
        )
        .await
        .unwrap();
        assert_eq!(result.status, MutationStatus::Accepted);
        let requests = server.finish();
        assert_eq!(operation(&requests[1]), operation(&requests[2]));
        assert_eq!(
            operation(&requests[1])["operationId"],
            result.operation_id.as_str()
        );
    });
}

#[test]
fn denial_stops_submission_locally_and_after_access_refresh() {
    tauri::async_runtime::block_on(async {
        let server = TestApi::dynamic(|_| panic!("local denial must not call HTTP"));
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let token = Secret::new("synthetic-secret".into()).unwrap();
        let mut access = access();
        access.role = AccessRole::Assistant;
        let remote = OnlineAccount {
            http: &http,
            token: &token,
            scope: &access.context,
        };
        assert_eq!(
            application::set_account_status(
                &remote,
                &access,
                1,
                AccountStatus::Inactive,
                &UuidV7Generator
            )
            .await
            .unwrap_err()
            .code,
            "PERMISSION_DENIED"
        );
        assert!(server.finish().is_empty());
        for status in [401, 403] {
            let server = TestApi::new(1, move |_| {
                (
                    status,
                    json!({"code":if status == 401 {"AUTHENTICATION_REQUIRED"} else {"PERMISSION_DENIED"},"message":"synthetic-secret must not escape","correlationId":CORRELATION,"details":[]}),
                )
            });
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            access.role = AccessRole::Owner;
            let remote = OnlineAccount {
                http: &http,
                token: &token,
                scope: &access.context,
            };
            let error = application::set_account_status(
                &remote,
                &access,
                1,
                AccountStatus::Inactive,
                &UuidV7Generator,
            )
            .await
            .unwrap_err();
            assert_eq!(
                error.code,
                if status == 401 {
                    "AUTHENTICATION_REQUIRED"
                } else {
                    "PERMISSION_DENIED"
                }
            );
            assert!(
                !serde_json::to_string(&error)
                    .unwrap()
                    .contains("synthetic-secret")
            );
            assert_eq!(server.finish().len(), 1);
        }
    });
}

#[test]
fn duplicate_preserves_original_conflict_or_rejection() {
    tauri::async_runtime::block_on(async {
        for (status, code, expected) in [
            ("conflict", "VERSION_CONFLICT", MutationStatus::Conflict),
            ("rejected", "NO_CHANGES", MutationStatus::Rejected),
        ] {
            let server = TestApi::new(2, move |request| {
                if request.starts_with("GET ") {
                    return (200, access_response());
                }
                let original = reply(request, status, Some(code))["results"][0].clone();
                let mut result = reply(request, "duplicate", None);
                result["results"][0]["originalResult"] = original;
                (200, result)
            });
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let token = Secret::new("synthetic-secret".into()).unwrap();
            let access = access();
            let remote = OnlineAccount {
                http: &http,
                token: &token,
                scope: &access.context,
            };
            let result = application::set_account_status(
                &remote,
                &access,
                1,
                AccountStatus::Inactive,
                &UuidV7Generator,
            )
            .await
            .unwrap();
            assert_eq!(result.status, expected);
            assert_eq!(result.code.as_deref(), Some(code));
            assert_eq!(result.refresh_status, RefreshStatus::NotRequested);
            assert_eq!(server.finish().len(), 2);
        }
    });
}
