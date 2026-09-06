use crate::model::{Deeplink, Envelope, Token};
use crate::{ApiError, Environment, SourceInfo, Transaction, TxStatus};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::fmt;
use std::sync::RwLock;
use std::time::Duration;

const MAX_BATCH: usize = 50;
const NOT_FOUND: i64 = 1;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// A client for the Bakong Open API.
///
/// Nothing here sleeps or retries on a timer. Payment polling is driven by the
/// caller, with [`crate::Backoff`] deciding the gaps.
pub struct BakongClient {
    http: reqwest::Client,
    base_url: String,
    token: RwLock<String>,
    renewal_email: Option<String>,
}

impl BakongClient {
    /// Talks to production or the sandbox with an already issued token.
    pub fn new(environment: Environment, token: impl Into<String>) -> Self {
        Self::with_base_url(environment.base_url(), token)
    }

    /// Talks to any host, which is how the test suite points it at a mock.
    pub fn with_base_url(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
                .unwrap_or_default(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: RwLock::new(token.into()),
            renewal_email: None,
        }
    }

    /// Lets the client renew its own token when Bakong rejects it.
    pub fn with_renewal_email(mut self, email: impl Into<String>) -> Self {
        self.renewal_email = Some(email.into());
        self
    }

    /// The token currently in use, which changes after a renewal.
    pub fn token(&self) -> String {
        self.read_token()
    }

    /// Asks for a fresh token and stores it. Tokens last about 90 days.
    pub async fn renew_token(&self) -> Result<String, ApiError> {
        let email = self.renewal_email.clone().ok_or(ApiError::NoRenewalEmail)?;

        let response = self
            .http
            .post(self.url("/v1/renew_token"))
            .json(&json!({ "email": email }))
            .send()
            .await?;

        let status = response.status();
        let envelope: Envelope<Token> = response.json().await?;

        if !status.is_success() || envelope.response_code != 0 {
            return Err(ApiError::Unauthorized {
                message: envelope.response_message.unwrap_or_default(),
            });
        }

        let token = envelope
            .data
            .ok_or(ApiError::MissingData {
                endpoint: "renew_token",
            })?
            .token;

        if token.is_empty() {
            return Err(ApiError::MissingData {
                endpoint: "renew_token",
            });
        }

        self.write_token(&token);

        Ok(token)
    }

    /// Whether `name@bank` is a real Bakong account.
    pub async fn check_bakong_account(&self, account_id: &str) -> Result<bool, ApiError> {
        let envelope: Envelope<Value> = self
            .post(
                "/v1/check_bakong_account",
                json!({ "accountId": account_id }),
            )
            .await?;

        if envelope.response_code == 0 {
            return Ok(true);
        }
        if envelope.error_code.is_some() {
            return Ok(false);
        }

        Err(bakong(envelope.error_code, envelope.response_message))
    }

    /// Asks about one payment by its MD5 handle.
    pub async fn check_transaction_by_md5(&self, md5: &str) -> Result<TxStatus, ApiError> {
        self.single("/v1/check_transaction_by_md5", json!({ "md5": md5 }))
            .await
    }

    /// Asks about up to 50 payments by MD5 handle, answered in the order given.
    pub async fn check_transaction_by_md5_list(
        &self,
        md5s: &[String],
    ) -> Result<Vec<TxStatus>, ApiError> {
        self.batch("/v1/check_transaction_by_md5_list", md5s).await
    }

    /// Asks about one payment by its full transaction hash.
    pub async fn check_transaction_by_hash(&self, hash: &str) -> Result<TxStatus, ApiError> {
        self.single("/v1/check_transaction_by_hash", json!({ "hash": hash }))
            .await
    }

    /// Asks about up to 50 payments by full hash, answered in the order given.
    pub async fn check_transaction_by_hash_list(
        &self,
        hashes: &[String],
    ) -> Result<Vec<TxStatus>, ApiError> {
        self.batch("/v1/check_transaction_by_hash_list", hashes)
            .await
    }

    /// Asks about one payment by short hash, which needs the amount to disambiguate.
    pub async fn check_transaction_by_short_hash(
        &self,
        hash: &str,
        amount: f64,
        currency: &str,
    ) -> Result<TxStatus, ApiError> {
        self.single(
            "/v1/check_transaction_by_short_hash",
            json!({ "hash": hash, "amount": amount, "currency": currency }),
        )
        .await
    }

    /// Asks about one payment by the instruction reference the sender used.
    pub async fn check_transaction_by_instruction_ref(
        &self,
        reference: &str,
    ) -> Result<TxStatus, ApiError> {
        self.single(
            "/v1/check_transaction_by_instruction_ref",
            json!({ "instructionRef": reference }),
        )
        .await
    }

    /// Asks about one payment by your own external reference.
    pub async fn check_transaction_by_external_ref(
        &self,
        reference: &str,
    ) -> Result<TxStatus, ApiError> {
        self.single(
            "/v1/check_transaction_by_external_ref",
            json!({ "externalRef": reference }),
        )
        .await
    }

    /// Turns a payload into a `bakong.page.link` the Bakong app can open.
    pub async fn generate_deeplink(
        &self,
        qr: &str,
        source: &SourceInfo,
    ) -> Result<String, ApiError> {
        let envelope: Envelope<Deeplink> = self
            .post(
                "/v1/generate_deeplink_by_qr",
                json!({ "qr": qr, "sourceInfo": source }),
            )
            .await?;

        if envelope.response_code != 0 {
            return Err(ApiError::Bakong {
                code: envelope.error_code,
                message: envelope.response_message.unwrap_or_default(),
            });
        }

        Ok(envelope
            .data
            .ok_or(ApiError::MissingData {
                endpoint: "generate_deeplink_by_qr",
            })?
            .short_link)
    }

    async fn single(&self, path: &'static str, body: Value) -> Result<TxStatus, ApiError> {
        let envelope: Envelope<Transaction> = self.post(path, body).await?;

        if envelope.response_code == 0 {
            return Ok(match envelope.data {
                Some(transaction) => TxStatus::Paid(Box::new(transaction)),
                None => TxStatus::NotFound,
            });
        }

        if envelope.error_code == Some(NOT_FOUND) {
            return Ok(TxStatus::NotFound);
        }

        Err(bakong(envelope.error_code, envelope.response_message))
    }

    async fn batch(&self, path: &'static str, items: &[String]) -> Result<Vec<TxStatus>, ApiError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }
        if items.len() > MAX_BATCH {
            return Err(ApiError::BatchTooLarge {
                count: items.len(),
                max: MAX_BATCH,
            });
        }

        let envelope: Envelope<Vec<Value>> = self.post(path, json!(items)).await?;

        if envelope.response_code != 0 && envelope.error_code != Some(NOT_FOUND) {
            return Err(bakong(envelope.error_code, envelope.response_message));
        }

        let returned = envelope.data.unwrap_or_default();
        if returned.len() != items.len() {
            return Err(ApiError::BatchMismatch {
                requested: items.len(),
                returned: returned.len(),
            });
        }

        returned.iter().map(status_of).collect()
    }

    async fn post<T: DeserializeOwned>(
        &self,
        path: &'static str,
        body: Value,
    ) -> Result<Envelope<T>, ApiError> {
        let mut response = self.send(path, &body).await?;

        if response.status() == StatusCode::UNAUTHORIZED && self.renewal_email.is_some() {
            self.renew_token().await?;
            response = self.send(path, &body).await?;
        }

        let status = response.status();
        let unauthorized = status == StatusCode::UNAUTHORIZED;

        let envelope: Envelope<T> = match response.json().await {
            Ok(envelope) => envelope,
            Err(error) if unauthorized => {
                return Err(ApiError::Unauthorized {
                    message: error.to_string(),
                })
            }
            Err(_) if !status.is_success() => {
                return Err(ApiError::Http {
                    status: status.as_u16(),
                })
            }
            Err(error) => return Err(ApiError::Transport(error)),
        };

        if unauthorized {
            return Err(ApiError::Unauthorized {
                message: envelope.response_message.unwrap_or_default(),
            });
        }

        Ok(envelope)
    }

    async fn send(&self, path: &str, body: &Value) -> Result<reqwest::Response, ApiError> {
        Ok(self
            .http
            .post(self.url(path))
            .bearer_auth(self.read_token())
            .json(body)
            .send()
            .await?)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn read_token(&self) -> String {
        self.token
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn write_token(&self, token: &str) {
        let mut stored = self
            .token
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        stored.clear();
        stored.push_str(token);
    }
}

impl fmt::Debug for BakongClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BakongClient")
            .field("base_url", &self.base_url)
            .field("token", &"<redacted>")
            .field("renewal_email", &self.renewal_email)
            .finish()
    }
}

fn bakong(code: Option<i64>, message: Option<String>) -> ApiError {
    ApiError::Bakong {
        code,
        message: message.unwrap_or_default(),
    }
}

/// Batch items carry their own status, and the transaction either inline or
/// under `data` depending on the endpoint. An item Bakong called SUCCESS is
/// never downgraded to NotFound, because that reports a paid order as unpaid.
fn status_of(item: &Value) -> Result<TxStatus, ApiError> {
    match item.get("status").and_then(Value::as_str) {
        Some("SUCCESS") => {
            let payload = item.get("data").unwrap_or(item);

            serde_json::from_value::<Transaction>(payload.clone())
                .map(|transaction| TxStatus::Paid(Box::new(transaction)))
                .map_err(|error| ApiError::Bakong {
                    code: None,
                    message: format!("a paid transaction could not be read: {error}"),
                })
        }
        Some("NOT_FOUND") => Ok(TxStatus::NotFound),
        Some("STATIC_QR") => Ok(TxStatus::StaticQr),
        other => Err(ApiError::Bakong {
            code: None,
            message: format!("unrecognised batch status {other:?}"),
        }),
    }
}
