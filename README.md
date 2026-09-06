# khqr-rs

[![CI](https://github.com/tangheng05/khqr-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/tangheng05/khqr-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/khqr-core.svg)](https://crates.io/crates/khqr-core)
[![docs.rs](https://img.shields.io/docsrs/khqr-core)](https://docs.rs/khqr-core)
[![license](https://img.shields.io/crates/l/khqr-core.svg)](https://github.com/tangheng05/khqr-rs/blob/main/LICENSE)
[![msrv](https://img.shields.io/badge/msrv-1.88-blue.svg)](https://releases.rs/docs/1.88.0/)

A Rust implementation of KHQR, the payment QR standard used by Bakong in Cambodia.

The codec lives in `khqr-core`. It has no async runtime and no HTTP, so it runs in a
CLI, in a browser through WASM, or on a POS terminal. Add `khqr-api` when you need to
check whether a payment arrived.

## Documentation

[Getting started](https://github.com/tangheng05/khqr-rs/blob/main/docs/getting-started.md) covers Rust, the CLI, the browser
and the mobile bindings. Beyond that: [generating](https://github.com/tangheng05/khqr-rs/blob/main/docs/generating.md),
[decoding](https://github.com/tangheng05/khqr-rs/blob/main/docs/decoding.md), [checking payments](https://github.com/tangheng05/khqr-rs/blob/main/docs/payments.md), and
[the format itself](https://github.com/tangheng05/khqr-rs/blob/main/docs/format.md) if you need to read a payload by hand.

## Status

All five crates are published. Verified against production Bakong: a QR from
this library was scanned by a banking app, paid, and found again through every
lookup the API offers. The API will change before 1.0, so pin an exact version.

| Crate | Purpose |
| --- | --- |
| [`khqr-core`](https://crates.io/crates/khqr-core) | Codec, checksum, builder, decoder, MD5, images |
| [`khqr-api`](https://crates.io/crates/khqr-api) | Bakong Open API client |
| [`khqr-cli`](https://crates.io/crates/khqr-cli) | `khqr gen`, `decode`, `verify`, `watch` |
| [`khqr-wasm`](https://crates.io/crates/khqr-wasm) | Browser and Node |
| [`khqr-ffi`](https://crates.io/crates/khqr-ffi) | Kotlin, Swift and Python |

## Layout

```
khqr-rs/
├── Cargo.toml          workspace manifest
├── khqr-cli/           the khqr binary
├── khqr-wasm/          wasm-bindgen over khqr-core
├── khqr-ffi/           uniffi bindings for Kotlin, Swift and Python
├── khqr-api/
│   ├── src/
│   │   ├── backoff.rs  how long to wait before polling again
│   │   ├── client.rs   the endpoints
│   │   ├── error.rs
│   │   ├── lib.rs
│   │   └── model.rs    request and response shapes
│   └── tests/          driven against a mock server, no token needed
└── khqr-core/
    ├── src/
    │   ├── builder.rs  typed payload and its builder
    │   ├── crc.rs      crc-16/ccitt-false, checksum append and verify
    │   ├── decoder.rs  payload in, flattened fields out
    │   ├── hash.rs     the md5 payment handle
    │   ├── error.rs    one error type for the whole crate
    │   ├── lib.rs
    │   ├── qr.rs       png and svg rendering, behind a feature
    │   ├── tlv.rs      tag-length-value encode and decode
    │   └── types.rs    currency and merchant type
    └── tests/
        ├── common/     published KHQR payloads used as reference vectors
        ├── builder.rs
        ├── crc.rs
        ├── decoder.rs
        ├── qr.rs
        ├── tlv.rs
        └── vectors.rs
```

## Usage

Building a payload validates it, then writes the TLV string and its checksum.

```rust
use khqr_core::Khqr;

let qr = Khqr::individual("jonhsmith@nbcq")
    .merchant_name("Jonh Smith")
    .merchant_city("Phnom Penh")
    .amount(500.0)
    .build()?
    .to_qr_string()?;
```

Riel is the default currency and gets no decimal places; dollars get two. An
amount makes the QR single use, so tag `01` becomes `12` on its own.

Field limits come from the reference SDK: 32 characters for account identifiers, 25 for
labels, 15 for the city, 13 for the amount. Tag `15`, the UnionPay merchant account, is
supported on both sides.

Reading one back flattens every nested template into a single struct.

```rust
let decoded = khqr_core::decode(&qr)?;
assert_eq!(decoded.merchant_name, "Jonh Smith");
```

Tags the crate does not know are kept in `unknown` rather than rejected, since
production QRs carry vendor extensions. A checksum that does not match is
always an error.

`md5(&qr)` gives the handle Bakong uses to poll for payment. It works for
dynamic QRs only, since a static one has no amount to track.

Images live behind the `image` feature, off by default:

```toml
khqr-core = { version = "0.1", features = ["image"] }
```

```rust
let png = khqr_core::to_png(&qr, 512)?;
let svg = khqr_core::to_svg(&qr)?;
```

Polling for payment is driven by you, never by a timer inside the client.

```rust
use khqr_api::{BakongClient, Backoff, Environment};

let client = BakongClient::new(Environment::Production, token)
    .with_renewal_email("you@example.com");
let mut backoff = Backoff::new();

let status = client.check_transaction_by_md5(&khqr_core::md5(&qr)).await?;
if status.is_pending() {
    tokio::time::sleep(backoff.next_delay()).await;
}
```

A rejected token is renewed once and the call retried, so the 90 day expiry
does not surface as a failure. Bakong geo-restricts, so expect HTTP 403 if you
run this outside Cambodia.

## Command line

```sh
cargo install khqr-cli

khqr gen --account shop@aclb --name "Coffee Klaing" --city "Phnom Penh"     --amount 5000 --expires-in 300 --png qr.png
khqr decode "0002010102..."
khqr verify "0002010102..."          # exits non zero if the checksum is wrong
BAKONG_TOKEN=... khqr watch --md5 682f33ec80e311d909f91d70a70ab436
```

`gen` prints the payload and its MD5 handle, then writes the images you asked
for. `watch` polls with the same backoff the library uses and gives up after
five minutes by default.

## Browser and Node

`khqr-wasm` wraps the codec only, so a payload can be built and rendered client
side with no server round trip.

```sh
wasm-pack build khqr-wasm --target web
```

```js
import init, { Khqr, toDataUri } from "./pkg/khqr_wasm.js";

await init();

const qr = Khqr.individual("shop@aclb");
qr.merchantName("Coffee Klaing");
qr.merchantCity("Phnom Penh");
qr.amount(5000);

document.querySelector("img").src = toDataUri(qr.build(), 512);
```

## Kotlin, Swift and Python

`khqr-ffi` exposes the codec through UniFFI, so the same compiled library backs
an Android app, an iOS app and a Python service.

```sh
cargo build -p khqr-ffi --release
cargo run -p khqr-ffi --bin uniffi-bindgen -- generate \
    --library target/release/libkhqr_ffi.so \
    --language kotlin --language swift --language python \
    --out-dir bindings
```

On macOS that library is `libkhqr_ffi.dylib`, and on Windows `khqr_ffi.dll`.

```python
import khqr_ffi as khqr

qr = khqr.generate(khqr.KhqrOptions(
    merchant_type=khqr.MerchantType.INDIVIDUAL,
    account_id="shop@aclb",
    merchant_name="Coffee Klaing",
    merchant_city="Phnom Penh",
    amount=5000.0,
))
handle = khqr.md5(qr)
```

Everything past the first four fields defaults to unset, so each language gets
named arguments rather than a twenty parameter call. Dart is not generated by
UniFFI itself; the third party `uniffi-dart` bindgen reads the same library.

## Building

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy -p khqr-wasm --target wasm32-unknown-unknown -- -D warnings
cargo build -p khqr-core --no-default-features --target thumbv7em-none-eabihf
cargo fmt --all --check
```

Fuzzing needs nightly:

```sh
cargo +nightly fuzz run decode
```

The same ground is covered on stable by `khqr-core/tests/hostile.rs`, which
throws forty thousand mutated and random payloads at the decoder on a fixed
seed.

## Versioning

Minimum supported Rust version is 1.88, checked in CI against that exact
toolchain. The API will change before 1.0, so pin an exact version. See [CHANGELOG.md](https://github.com/tangheng05/khqr-rs/blob/main/CHANGELOG.md).

## License

MIT. See [LICENSE](https://github.com/tangheng05/khqr-rs/blob/main/LICENSE).
