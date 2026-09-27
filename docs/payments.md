# Checking payments

Bakong tells you whether a payment arrived. You ask by MD5 handle, which you
compute locally from the payload with no API call.

```rust
use khqr_api::{BakongClient, Environment, TxStatus};
use khqr_core::md5;

let client = BakongClient::new(Environment::Production, token);
let status = client.check_transaction_by_md5(&md5(&qr)).await?;

match status {
    TxStatus::Paid(transaction) => println!("paid {}", transaction.hash),
    TxStatus::NotFound => println!("not yet"),
    TxStatus::StaticQr => println!("this QR carries no amount and cannot be tracked"),
}
```

A payment that has not arrived is not an error. Bakong answers HTTP 200 with
`responseCode: 1` and `errorCode: 1`, and that becomes `NotFound`. Any other
code becomes an `ApiError`, so an outage or a throttle can never be mistaken
for an unpaid order.

## Getting a token

Register at <https://api-bakong.nbc.gov.kh/register>. Tokens last about 90
days. Sandbox is a separate host with its own tokens.

Tell the client the email you registered with, and it renews on its own when
Bakong rejects the token:

```rust
let client = BakongClient::new(Environment::Production, token)
    .with_renewal_email("you@example.com");
```

The first request that comes back `401` triggers a renewal and one retry, and
the new token replaces the old one in the client. Read it back with
`client.token()` if you want to persist it.

Without a renewal email a rejected token is an `ApiError::Unauthorized` and
nothing is retried.

## Polling without burning your rate limit

The client never sleeps and never retries on a timer. You drive the loop, and
`Backoff` decides the gaps.

```rust
use khqr_api::Backoff;

let mut backoff = Backoff::new();

loop {
    let status = client.check_transaction_by_md5(&handle).await?;
    if !status.is_pending() {
        break status;
    }
    tokio::time::sleep(backoff.next_delay()).await;
}
```

Delays double from two seconds up to a minute. A customer who leaves a
checkout screen open for an hour costs you around sixty requests instead of
several thousand.

`Backoff::with_bounds` changes the start and ceiling.  `reset()` starts the
sequence over, which is what you want after showing a new QR.

## What a paid transaction tells you

Every field below came back from a real payment, and all but `hash` are
optional because Bakong leaves them null when they do not apply.

| Field | Example |
| --- | --- |
| `hash` | `296d729a1c8a7eee...` the full transaction hash |
| `from_account_id` | `abaakhppxxx@abaa`, the payer |
| `to_account_id` | `hengthegoat@aclb`, you |
| `currency`, `amount` | `USD`, `0.01` |
| `created_date_ms`, `acknowledged_date_ms` | epoch milliseconds |
| `external_ref` | `100FT38906478429`, the payer bank's reference |
| `description`, `instruction_ref` | usually null |
| `tracking_status`, `receiver_bank`, `receiver_bank_account` | usually null |

Unknown fields are ignored rather than rejected, so a new one Bakong adds will
not break decoding.

`amount` is a float in major units. `transaction.amount_minor()` gives it in
the units `KhqrBuilder::amount_minor` takes (cents for dollars, riel for riel),
or `None` if it is missing or has more precision than the currency allows, so
you can compare it with your order total without float arithmetic.

`created_date_ms` is when the payer sent the money and `acknowledged_date_ms`
when Bakong recorded it, which can be later. To decide whether a QR was paid
before it expired, compare `created_date_ms`.

## Asking about many at once

```rust
let handles = vec![first, second, third];
for (handle, status) in handles.iter().zip(client.check_transaction_by_md5_list(&handles).await?) {
    match status {
        Ok(TxStatus::Paid(transaction)) => println!("{handle} paid {}", transaction.hash),
        Ok(_) => println!("{handle} not yet"),
        Err(error) => println!("{handle} could not be read: {error}"),
    }
}
```

Answers come back in the order you asked. Fifty is the limit, and the client
checks that before making a request rather than letting Bakong reject it.

The outer `?` is for the batch as a whole: a network failure, a rejected token,
or an answer that does not line up with the request. Each item then has its own
result, so one answer the client cannot read does not hide the other 49.

## Every endpoint

| Method | Asks by |
| --- | --- |
| `check_transaction_by_md5` | MD5 handle |
| `check_transaction_by_md5_list` | up to 50 handles. See the warning below. |
| `check_transaction_by_hash` | full transaction hash |
| `check_transaction_by_hash_list` | up to 50 hashes |
| `check_transaction_by_short_hash` | short hash, plus amount and currency |
| `check_transaction_by_instruction_ref` | the sender's instruction reference |
| `check_transaction_by_external_ref` | your own reference |
| `check_bakong_account` | whether `name@bank` exists, and needs no token |
| `generate_deeplink` | a `bakong.page.link` the Bakong app can open |
| `renew_token` | your registered email |

## One endpoint that does not work

`check_transaction_by_md5_list` answers with a bare nginx `403 Forbidden` on
production, with a valid token and whatever request body you send. Its sibling
`check_transaction_by_hash_list` works normally on the same token, so this is
the edge refusing the path rather than an auth or payload problem.

The client reports it as `ApiError::Http { status: 403 }`. Until it comes back,
poll single handles with `check_transaction_by_md5`, or batch by hash once you
have hashes from a previous lookup.

## Deep links

On a phone, a link that opens the Bakong app beats a QR the user has to
photograph with another device.

```rust
use khqr_api::SourceInfo;

let source = SourceInfo::new(
    "Coffee Klaing",
    "https://shop.example/icon.png",
    "https://shop.example/paid",
);

let link = client.generate_deeplink(&qr, &source).await?;
```

All three fields are required by the endpoint. The callback is where Bakong
sends the user back to once they are done.

## Two things that will catch you out

**Bakong geo restricts.** Requests from outside Cambodia can come back `403`.
A German or Singaporean VPS is exactly the case that fails. Test with a plain
`curl` from the machine you intend to deploy on before you write anything
against it:

```sh
curl -i -X POST https://api-bakong.nbc.gov.kh/v1/check_transaction_by_md5 \
    -H "Content-Type: application/json" -d '{"md5":"test"}'
```

`401` means you are through and only need a token. `403` means either a
Cambodian egress is needed, or that endpoint is closed to you: see the section
below, where one endpoint answers 403 from inside reach.

**Static QRs cannot be tracked.** There is no single transaction to look up, so
`check_transaction_by_md5` answers `NotFound` forever. Only the batch endpoints
report `StaticQr`, and only they can tell you why. Your receiving bank has to check
its own records. If you need to know who paid, generate a dynamic QR.

## Errors

| Variant | Means |
| --- | --- |
| `Transport` | The request never completed, or the response was not JSON. |
| `Unauthorized` | The token was rejected and could not be renewed. |
| `Bakong` | Bakong answered with an error of its own. Carries its code. |
| `MissingData` | A successful response arrived without the data it should carry. |
| `NoRenewalEmail` | Renewal was attempted with no email set. |
| `BatchTooLarge` | More than 50 items. Caught before any request goes out. |
| `BatchMismatch` | A batch answered with a different number of results than were asked about, so they cannot be lined up. |
| `Http` | A non JSON error response. `Http { status: 403 }` is either the geo restriction or an endpoint closed to your token. |

`ApiError` is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.

A rejected token comes back with the same JSON envelope as a success, so the
message from Bakong is preserved rather than replaced with a status code.

## TLS

`rustls` by default, which needs no system libraries. Switch if you would
rather use the platform's:

```toml
khqr-api = { version = "0.4", default-features = false, features = ["native-tls"] }
```

The default `rustls-tls` brings rustls's `aws-lc-rs` backend. If your
application already enables the `ring` backend (through sqlx or hyper-rustls,
say), both end up in the binary and rustls refuses to pick one, panicking on
the first connection. Use `rustls-tls-no-provider` instead and install the
backend you want once at startup, before creating the client:

```toml
khqr-api = { version = "0.4", default-features = false, features = ["rustls-tls-no-provider"] }
```

```rust
rustls::crypto::ring::default_provider().install_default().ok();
```

## Keep the token on your server

The token identifies you to Bakong. Generating a QR needs no token at all, so
a browser or a phone app can do that locally, and only the payment check goes
through your backend. Shipping a Bakong token to a client is handing it out.
