//! Typed access to the remote availability endpoint. No local state is read or
//! written here; the desktop gate keeps the session stable around the call.
use crate::platform::http::{APIResponse, APIResult, APITransport};
use crate::platform::secrets::Secret;
use crate::shared_kernel::id::AccountId;
use reqwest::Method;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAvailabilityQuery {
    pub professional_id: String,
    pub service_id: String,
    pub from: String,
    pub until: String,
    pub duration_minutes: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteInterval {
    pub starts_at: String,
    pub ends_at: String,
}

/// Exact contract shape of `GET /v1/accounts/{account}/availability`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteAvailabilityResponse {
    pub account_id: String,
    pub professional_id: String,
    pub service_id: String,
    pub reference_duration_minutes: i64,
    pub requested_duration_minutes: i64,
    pub available_intervals: Vec<RemoteInterval>,
}

pub struct SchedulingAPI<'a>(pub &'a APITransport);

impl SchedulingAPI<'_> {
    pub async fn availability(
        &self,
        token: &Secret,
        account: &AccountId,
        query: &RemoteAvailabilityQuery,
    ) -> APIResult<APIResponse<RemoteAvailabilityResponse>> {
        let mut params = vec![
            ("professionalId", query.professional_id.as_str()),
            ("serviceId", query.service_id.as_str()),
            ("from", query.from.as_str()),
            ("until", query.until.as_str()),
        ];
        let duration;

        if let Some(value) = query.duration_minutes {
            duration = value.to_string();
            params.push(("durationMinutes", duration.as_str()));
        }

        self.0
            .request_with_query(
                Method::GET,
                &format!("v1/accounts/{}/availability", account.as_str()),
                Some(token),
                None,
                &params,
            )
            .await
    }
}
