# khqr-rs

A Rust implementation of KHQR, the payment QR standard used by Bakong in Cambodia.

KHQR is EMVCo merchant-presented QR: a flat ASCII string of nested tag-length-value
triples ending in a CRC-16 checksum. Generating one is pure string work, so there is
no good reason for a library that does it to require an async runtime.

That is the point of this crate. The codec lives in `khqr-core` with no heavy
dependencies, which keeps it usable from a sync CLI, a WASM bundle in the browser, or
a POS terminal. Network calls to the Bakong Open API are a separate crate you only pull
in if you need them.

## Status

Early. The workspace, the reference test vectors and CI are in place; the codec itself
is being written phase by phase. Not on crates.io yet, and the API will change.

| Crate | Purpose | State |
| --- | --- | --- |
| `khqr-core` | TLV codec, CRC16, builder, decoder, MD5, images | usable |
| `khqr-api` | Bakong Open API client | usable |
| `khqr-cli` | `khqr gen`, `decode`, `verify`, `watch` | planned |
| `khqr-wasm` | browser and Node bindings | planned |

## Layout

```
khqr-rs/
├── Cargo.toml          workspace manifest
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

## Building

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

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

MIT. See [LICENSE](LICENSE).
