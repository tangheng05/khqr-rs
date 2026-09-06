# khqr-rs

A Rust implementation of KHQR, the payment QR standard used by Bakong in Cambodia.

KHQR is EMVCo merchant-presented QR: a flat ASCII string of nested tag-length-value
triples ending in a CRC-16 checksum. Generating one is pure string work, so there is
no good reason for a library that does it to require an async runtime.

That is the point of this crate. The codec lives in `khqr-core` with no heavy
dependencies, which keeps it usable from a sync CLI, a WASM bundle in the browser, or
a POS terminal. Network calls to the Bakong Open API are a separate crate you only pull
in if you need them.

## Documentation

[Getting started](https://github.com/tangheng05/khqr-rs/blob/main/docs/getting-started.md) covers Rust, the CLI, the browser
and the mobile bindings. Beyond that: [generating](https://github.com/tangheng05/khqr-rs/blob/main/docs/generating.md),
[decoding](https://github.com/tangheng05/khqr-rs/blob/main/docs/decoding.md), [checking payments](https://github.com/tangheng05/khqr-rs/blob/main/docs/payments.md), and
[the format itself](https://github.com/tangheng05/khqr-rs/blob/main/docs/format.md) if you need to read a payload by hand.

## Status

The codec, the API client, the CLI and all three binding targets are written and
tested against the four published KHQR vectors. Not on crates.io yet, and the API
will change before 1.0, so pin an exact version.

| Crate | Purpose | State |
| --- | --- | --- |
| `khqr-core` | TLV codec, CRC16, builder, decoder, MD5, images | usable |
| `khqr-api` | Bakong Open API client | usable |
| `khqr-cli` | `khqr gen`, `decode`, `verify`, `watch` | usable |
| `khqr-wasm` | browser and Node bindings | usable |
| `khqr-ffi` | Kotlin, Swift and Python bindings | usable |

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
cargo install --path khqr-cli

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
const qr = new Khqr.individual("shop@aclb");
qr.merchantName("Coffee Klaing");
qr.merchantCity("Phnom Penh");
qr.amount(5000);
const payload = qr.build();

document.querySelector("img").src = toDataUri(payload, 512);
```

## Kotlin, Swift and Python

`khqr-ffi` exposes the codec through UniFFI, so the same compiled library backs
an Android app, an iOS app and a Python service.

```sh
cargo build -p khqr-ffi --release
cargo run -p khqr-ffi --bin uniffi-bindgen -- generate     --library target/release/libkhqr_ffi.so     --language kotlin --language swift --language python     --out-dir bindings
```

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

Minimum supported Rust version is 1.75, checked in CI. The API will change
before 1.0, so pin an exact version. See [CHANGELOG.md](https://github.com/tangheng05/khqr-rs/blob/main/CHANGELOG.md).

## Roadmap

1. TLV codec, encode and decode, with correct handling of multi-byte values
2. CRC-16/CCITT-FALSE, hand rolled
3. Typed payloads and a builder with the spec's validation rules
4. Full decoder, lenient about unknown tags and strict about the checksum
5. PNG and SVG rendering behind a feature flag, plus the MD5 payment handle
6. `khqr-api`, the async Bakong client
7. WASM, UniFFI and CLI front ends

## Notes on the spec

Two things are easy to get wrong and both break the checksum:

- Lengths count characters of the value and are always two digits, so `04` and not `4`.
- KHR amounts carry no decimals, USD amounts carry two.

Any QR with an amount is dynamic, which means tag `01` must be `12` and tag `99` must
carry an expiry timestamp. A QR without an amount is static and cannot be tracked by MD5.

The KHQR logo and card assets belong to the National Bank of Cambodia and must be used
unmodified. They are deliberately not bundled here.

## License

MIT. See [LICENSE](https://github.com/tangheng05/khqr-rs/blob/main/LICENSE).
