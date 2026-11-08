//! Online adapter for the remote availability query. The adapter validates the
//! response against the request before any consumer sees it.
use super::super::application as app;
use super::super::domain::SchedulingError;
use super::api::{RemoteAvailabilityQuery, RemoteAvailabilityResponse, SchedulingAPI};
use crate::components::identity::{api::IdentityAPI, domain::AccountAccess};
use crate::platform::http::{APICode, APIFailure, APITransport};
use crate::platform::secrets::Secret;
use crate::shared_kernel::id::{AccountId, OperationContext};
use jiff::Timestamp;
use std::str::FromStr;

pub struct OnlineScheduling<'a> {
    pub http: &'a APITransport,
    pub token: &'a Secret,
    pub scope: &'a OperationContext,
}

pub(crate) fn remote_error(error: APIFailure) -> SchedulingError {
    let code = serde_json::to_value(error.code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "TEMPORARY_FAILURE".to_owned());
    SchedulingError::new(
        &code,
        "la consulta remota de disponibilidad no se completó; se conserva la vista local",
    )
}

fn invalid_response() -> APIFailure {
    APIFailure::local(APICode::InvalidResponse)
}

/// Response validation: IDs must match the request, durations stay within
/// 1..=1440 and every interval parses as a half-open UTC instant pair.
fn validate_response(
    request: &app::AvailabilityRequest,
    response: &RemoteAvailabilityResponse,
    expected_account: &str,
) -> Result<(), APIFailure> {
    if response.account_id != expected_account
        || response.professional_id != request.professional_id
        || response.service_id != request.service_id
    {
        return Err(invalid_response());
    }

    if !(1..=1440).contains(&response.reference_duration_minutes)
        || !(1..=1440).contains(&response.requested_duration_minutes)
    {
        return Err(invalid_response());
    }

    for interval in &response.available_intervals {
        let starts = Timestamp::from_str(&interval.starts_at).map_err(|_| invalid_response())?;
        let ends = Timestamp::from_str(&interval.ends_at).map_err(|_| invalid_response())?;

        if starts >= ends {
            return Err(invalid_response());
        }
    }

    Ok(())
}

impl app::RemoteAvailabilityPort for OnlineScheduling<'_> {
    async fn refresh_access(&self) -> app::Result<AccountAccess> {
        let access = IdentityAPI(self.http)
            .access(self.token, &self.scope.account_id)
            .await
            .map_err(remote_error)?
            .value;

        Ok(AccountAccess {
            context: OperationContext {
                account_id: access.account_id,
                user_id: access.user_id,
                device_id: access.device_id,
                permission_epoch: access.permission_epoch,
            },
            role: access.role,
            status: access.status,
            allowed_actions: access.allowed_actions,
            checked_at_millis: 0,
        })
    }

    async fn availability(
        &self,
        access: &AccountAccess,
        request: &app::AvailabilityRequest,
    ) -> app::Result<app::RemoteAvailability> {
        let query = RemoteAvailabilityQuery {
            professional_id: request.professional_id.clone(),
            service_id: request.service_id.clone(),
            from: request.from.to_string(),
            until: request.until.to_string(),
            duration_minutes: request.duration_minutes,
        };
        let response = SchedulingAPI(self.http)
            .availability(
                self.token,
                &AccountId::try_from(access.context.account_id.as_str().to_owned())
                    .map_err(|_| SchedulingError::internal("cuenta inválida"))?,
                &query,
            )
            .await
            .map_err(remote_error)?;

        validate_response(request, &response.value, access.context.account_id.as_str())
            .map_err(remote_error)?;

        Ok(app::RemoteAvailability {
            account_id: response.value.account_id,
            professional_id: response.value.professional_id,
            service_id: response.value.service_id,
            reference_duration_minutes: response.value.reference_duration_minutes,
            requested_duration_minutes: response.value.requested_duration_minutes,
            intervals: response
                .value
                .available_intervals
                .into_iter()
                .map(|interval| app::AvailabilityInterval {
                    starts_at: interval.starts_at,
                    ends_at: interval.ends_at,
                })
                .collect(),
            origin: "remote".into(),
        })
    }
}
