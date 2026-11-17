use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::app_error::AppError;

const VERIFY_ENDPOINT: &str = "https://api.nexmo.com/v2/verify";

/// Shared HTTP client and Vonage API operations.
#[derive(Clone)]
pub struct VonageClient {
    http_client: reqwest::Client,
}

// Verify V2
#[derive(Serialize)]
struct VerifyWorkflow {
    channel: &'static str,
    to: String,
}

#[derive(Serialize)]
struct VerifyRequest {
    brand: &'static str,
    workflow: Vec<VerifyWorkflow>,
    channel_timeout: u16,
}

#[derive(Deserialize)]
struct VerifyResponse {
    request_id: String,
}

impl VonageClient {
    /// Create a Vonage client with a bounded request timeout.
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Unable to create Vonage HTTP client");
        Self { http_client }
    }

    /// Start a Verify request and return its request ID.
    pub async fn start_verification(
        &self,
        vonage_token: &str,
        phone_number: &str,
    ) -> Result<String, AppError> {
        let verify_body = VerifyRequest {
            brand: "Rock Solid",
            workflow: vec![VerifyWorkflow {
                channel: "sms",
                to: phone_number.to_owned(),
            }],
            channel_timeout: 600,
        };
        let response = self
            .http_client
            .post(VERIFY_ENDPOINT)
            .bearer_auth(vonage_token)
            .json(&verify_body)
            .send()
            .await?;
        if !response.status().is_success() {
            debug!("Verification provider rejected request with response: {response:?}");
            return Err(AppError::ExternalApiError(
                "Verification provider rejected the request".to_string(),
            ));
        }
        let verify_data: VerifyResponse = response.json().await?;
        debug!("REQUEST_ID=\"{}\"", verify_data.request_id);
        Ok(verify_data.request_id)
    }
}
