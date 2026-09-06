//! Phase 6: the client, driven entirely against a mock server.

use khqr_api::{ApiError, BakongClient, SourceInfo, TxStatus};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn paid_body() -> serde_json::Value {
    json!({
        "responseCode": 0,
        "responseMessage": "Getting transaction successfully.",
        "errorCode": null,
        "data": {
            "hash": "e40a1cd9",
            "fromAccountId": "customer@bank",
            "toAccountId": "you@aclb",
            "currency": "USD",
            "amount": 0.1,
            "description": null,
            "createdDateMs": 1_772_125_349_000i64,
            "acknowledgedDateMs": 1_772_125_351_000i64,
            "externalRef": "100FT36931627892"
        }
    })
}

fn unpaid_body() -> serde_json::Value {
    json!({
        "responseCode": 1,
        "responseMessage": "Transaction could not be found.",
        "errorCode": 1,
        "data": null
    })
}

async fn mount(server: &MockServer, endpoint: &str, body: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path(endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(server)
        .await;
}

#[tokio::test]
async fn a_paid_transaction_is_parsed() {
    let server = MockServer::start().await;
    mount(&server, "/v1/check_transaction_by_md5", paid_body()).await;

    let client = BakongClient::with_base_url(server.uri(), "token");
    let status = client.check_transaction_by_md5("abc").await.unwrap();

    let transaction = status.transaction().expect("payment should be present");
    assert_eq!(transaction.hash, "e40a1cd9");
    assert_eq!(transaction.amount, Some(0.1));
    assert_eq!(transaction.currency.as_deref(), Some("USD"));
    assert_eq!(
        transaction.external_ref.as_deref(),
        Some("100FT36931627892")
    );
    assert!(!status.is_pending());
}

#[tokio::test]
async fn an_unpaid_transaction_stays_pending() {
    let server = MockServer::start().await;
    mount(&server, "/v1/check_transaction_by_md5", unpaid_body()).await;

    let client = BakongClient::with_base_url(server.uri(), "token");
    let status = client.check_transaction_by_md5("abc").await.unwrap();

    assert_eq!(status, TxStatus::NotFound);
    assert!(status.is_pending());
}

#[tokio::test]
async fn a_batch_keeps_the_per_item_status() {
    let server = MockServer::start().await;
    mount(
        &server,
        "/v1/check_transaction_by_md5_list",
        json!({
            "responseCode": 0,
            "data": [
                { "status": "SUCCESS", "hash": "aaa", "amount": 1.5, "currency": "KHR" },
                { "status": "NOT_FOUND" },
                { "status": "STATIC_QR" }
            ]
        }),
    )
    .await;

    let client = BakongClient::with_base_url(server.uri(), "token");
    let md5s = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let statuses = client.check_transaction_by_md5_list(&md5s).await.unwrap();

    assert_eq!(statuses.len(), 3);
    assert_eq!(statuses[0].transaction().unwrap().hash, "aaa");
    assert_eq!(statuses[1], TxStatus::NotFound);
    assert_eq!(statuses[2], TxStatus::StaticQr);
}

#[tokio::test]
async fn an_oversized_batch_never_leaves_the_process() {
    let client = BakongClient::with_base_url("http://127.0.0.1:1", "token");
    let md5s = vec!["a".to_string(); 51];

    match client.check_transaction_by_md5_list(&md5s).await {
        Err(ApiError::BatchTooLarge { count, max }) => {
            assert_eq!((count, max), (51, 50));
        }
        other => panic!("expected a batch limit error, got {other:?}"),
    }
}

#[tokio::test]
async fn a_known_account_is_reported() {
    let server = MockServer::start().await;
    mount(
        &server,
        "/v1/check_bakong_account",
        json!({ "responseCode": 0, "data": { "bakongAccountId": "shop@aclb" } }),
    )
    .await;

    let client = BakongClient::with_base_url(server.uri(), "token");

    assert!(client.check_bakong_account("shop@aclb").await.unwrap());
}

#[tokio::test]
async fn an_unknown_account_is_reported() {
    let server = MockServer::start().await;
    mount(
        &server,
        "/v1/check_bakong_account",
        json!({ "responseCode": 1, "errorCode": 4, "data": null }),
    )
    .await;

    let client = BakongClient::with_base_url(server.uri(), "token");

    assert!(!client.check_bakong_account("nobody@nowhere").await.unwrap());
}

#[tokio::test]
async fn a_deeplink_comes_back() {
    let server = MockServer::start().await;
    mount(
        &server,
        "/v1/generate_deeplink_by_qr",
        json!({
            "responseCode": 0,
            "data": { "shortLink": "https://bakong.page.link/ABCD" }
        }),
    )
    .await;

    let client = BakongClient::with_base_url(server.uri(), "token");
    let source = SourceInfo::new(
        "Shop",
        "https://shop.test/icon.png",
        "https://shop.test/paid",
    );
    let link = client.generate_deeplink("000201", &source).await.unwrap();

    assert_eq!(link, "https://bakong.page.link/ABCD");
}

#[tokio::test]
async fn a_rejected_token_is_renewed_and_the_call_retried() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/check_transaction_by_md5"))
        .and(header("authorization", "Bearer expired"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "responseCode": 1,
            "responseMessage": "Unauthorized, not yet requested for token or code invalid",
            "errorCode": 6,
            "data": null
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/renew_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "responseCode": 0,
            "data": { "token": "fresh" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/check_transaction_by_md5"))
        .and(header("authorization", "Bearer fresh"))
        .respond_with(ResponseTemplate::new(200).set_body_json(paid_body()))
        .mount(&server)
        .await;

    let client =
        BakongClient::with_base_url(server.uri(), "expired").with_renewal_email("me@shop.test");
    let status = client.check_transaction_by_md5("abc").await.unwrap();

    assert!(status.transaction().is_some());
    assert_eq!(client.token(), "fresh");
}

#[tokio::test]
async fn without_a_renewal_email_a_rejected_token_is_an_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/check_transaction_by_md5"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "responseCode": 1,
            "responseMessage": "Unauthorized",
            "errorCode": 6,
            "data": null
        })))
        .mount(&server)
        .await;

    let client = BakongClient::with_base_url(server.uri(), "expired");

    match client.check_transaction_by_md5("abc").await {
        Err(ApiError::Unauthorized { message }) => assert_eq!(message, "Unauthorized"),
        other => panic!("expected an unauthorized error, got {other:?}"),
    }
}

#[tokio::test]
async fn renewing_without_an_email_fails_before_any_request() {
    let client = BakongClient::with_base_url("http://127.0.0.1:1", "token");

    assert!(matches!(
        client.renew_token().await,
        Err(ApiError::NoRenewalEmail)
    ));
}

#[test]
fn the_token_is_kept_out_of_debug_output() {
    let client = BakongClient::with_base_url("http://127.0.0.1:1", "supersecret");

    assert!(!format!("{client:?}").contains("supersecret"));
}
