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
| `khqr-core` | TLV codec, CRC16, builder, decoder, MD5 | in progress |
| `khqr-api` | Bakong Open API client | planned |
| `khqr-cli` | `khqr gen`, `decode`, `verify`, `watch` | planned |
| `khqr-wasm` | browser and Node bindings | planned |

## Layout

```
khqr-rs/
├── Cargo.toml          workspace manifest
└── khqr-core/
    ├── src/
    │   ├── builder.rs  typed payload and its builder
    │   ├── crc.rs      crc-16/ccitt-false, checksum append and verify
    │   ├── error.rs    one error type for the whole crate
    │   ├── lib.rs
    │   ├── tlv.rs      tag-length-value encode and decode
    │   └── types.rs    currency and merchant type
    └── tests/
        ├── common/     published KHQR payloads used as reference vectors
        ├── builder.rs
        ├── crc.rs
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

## Building

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
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
