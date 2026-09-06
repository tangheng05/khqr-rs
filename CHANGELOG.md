# Changelog

This project follows [semantic versioning](https://semver.org). Until 1.0,
behaviour can change in any release, including a patch: 0.1.1 both raised the
minimum Rust version and made the decoder reject payloads it used to accept.
Pin `"=0.2.0"` if you would rather review each change.

## 0.2.0

### Added

- `DecodedKhqr::amount()` returns the amount as a number, or `None` when it is
  not a finite non negative one. `"NaN"` parses, and every comparison against it
  is false, so the raw string was a trap for a guard like `amount < expected`.

### Changed

- `khqr-ffi` no longer pulls the binding generator into every consumer. The
  `cli` feature carries it and the `uniffi-bindgen` binary requires it, so a
  mobile app linking the library compiles 69 crates rather than 112, without a
  template engine, an executable parser or an argument parser.

### Fixed

- Lengths count UTF-16 code units, as `String.length` does in the JavaScript and
  Dart SDKs. Counting Unicode scalars agreed on Khmer and disagreed on anything
  astral, so a payload holding an emoji parsed one way here and another way
  everywhere else, both readings carrying the same valid checksum. An attacker
  choosing the emoji chose which amount or account each reader saw.
- A repeated sub-tag inside a template is refused. The duplicate-tag defence in
  0.1.1 covered only top-level tags, so a second account ID inside tag `29` was
  accepted: this crate reported the first, a reader that assigns into a map
  reported the last, and the checksum satisfied both.
- A second tag `63` is refused, as is a checksum field that is not four
  characters. Either let one payload carry two valid checksums.
- Amounts round on the value the double actually holds. Scaling by a power of
  ten first turned near ties into exact ties, so `2.675` was written as `2.68`
  where the reference SDK writes `2.67`. It affected about 4% of two-decimal
  amounts and always rounded up.
- An amount that rounds to nothing is refused rather than making a dynamic QR
  nobody can pay.
- Control characters are refused in the account, merchant ID, acquiring bank and
  UnionPay fields, which had only a length check.
- `decode` refuses a payload larger than any QR symbol can hold. Reading
  allocates several times the input, so a 40MB string cost over a gigabyte.
- Batch answers are matched by the identifier each item echoes back, not only by
  count. A reordered answer used to attribute a payment to the wrong order.
- `khqr decode` replaces control characters before printing. A merchant name
  carrying an escape sequence could rewrite lines already on screen, including
  the amount.
- A renewal refused with a non JSON body is `Unauthorized` rather than a
  transport error.
- `to_png` accepts the documented maximum of 4096.

- The browser sample followed a build that leaves the renderers out, then
  imported `toDataUri`. A named import of a missing export is a link error, so
  the pasted sample killed the whole module.
- The decoding guide left `DuplicateTag` and `InvalidField` out of its error
  table, and did not say `KhqrError` is non exhaustive.
- Two documents claimed a 403 always means the request came from outside
  Cambodia. One endpoint answers 403 from inside reach.
- `StaticQr` was described as something the single lookup returns. Only the
  batch endpoints report it.
- The expiry rationale counted three vectors carrying an amount with no expiry.
  It is two.
- The versioning policy said breaking changes wait for a minor release, while
  0.1.1 raised the minimum Rust version and tightened the decoder.

## 0.1.5

### Added

- `to_png_with`, `to_svg_with` and `to_base64_uri_with` take an `ImageOptions`
  with the error correction level and quiet zone. Overlaying a logo on the old
  output could exceed what the default level recovers, giving codes that scanned
  unreliably. `khqr gen` gains `--ecc` and `--quiet-zone`.
- `khqr-wasm` has an `image` feature, on by default. Turning it off drops the
  PNG and SVG renderers and takes the bundle from 260KB to 113KB, which suits a
  page that already draws QR codes in JavaScript.
- A browser test for `khqr-wasm` under `khqr-wasm/tests/browser`. The Rust
  unit tests run on the host, where `JsValue` cannot exist, so the bindings had
  never been exercised as a browser sees them. The page runs the real wasm in a
  real browser: 20 checks covering the published vector, the decode getters, the
  currency guard, Khmer text and all three renderers.

## 0.1.4

### Changed

- Trimmed the README. The roadmap, the spec notes and the intro essay all said
  things the docs already say better, and duplicated guidance drifts.

## 0.1.3

### Changed

- Documented what a paid transaction carries, the confirmed not-found response,
  and the payment check in the getting started guide.

## 0.1.2

### Added

- `Transaction` carries `instruction_ref`, `tracking_status`, `receiver_bank`
  and `receiver_bank_account`, which the live service returns and this crate
  was dropping.

### Changed

- Verified end to end against production with a real payment. A QR built by
  this library was scanned by a banking app, paid, and then found again by
  MD5 handle, full hash, short hash, external reference and hash batch. The
  MD5 computed locally matches the one Bakong indexes the transaction under.
- `check_transaction_by_md5_list` returns a bare nginx 403 on production with a
  valid token, whatever the request body. `check_transaction_by_hash_list`
  works on the same token. Documented rather than worked around.
- Checked against the live Bakong service for the first time. Confirmed:
  `check_bakong_account` needs no token and returns `errorCode: 11` for an
  unknown account; an expired token gives HTTP 401 with an envelope on most
  endpoints but a bare nginx HTML 403 on `check_transaction_by_md5_list`;
  and `generate_deeplink_by_qr` reports its own failures with HTTP 200 and a
  non zero `responseCode`. The client handles all four correctly.
- The 403 message no longer asserts geography, since the live service also
  returns 403 for an endpoint a token cannot reach.

## 0.1.1

### Fixed

- `khqr-core`: a repeated top-level tag is now refused. A checksum is not a
  signature, so anyone able to reprint a QR could append a second tag `53` or a
  second account template and recompute the CRC. The decoder took the last one,
  which changed the currency or the destination account on a payload that still
  verified.
- `khqr-core`: `-0.0` passed the amount check and wrote an amount of `-0`.
- `khqr-core`: amounts round ties away from zero, matching the reference SDK's
  `toFixed`. Rust's float formatting rounds them to even, so `500.5` riel wrote
  `500` where the official generator writes `501`, giving a different checksum
  and a different MD5 handle.
- `khqr-core`: `to_png` and `to_base64_uri` overflowed and panicked above about
  65,000 pixels. Sizes are now capped at 4096.
- `khqr-core`: empty and control-character values are refused in required
  fields, and account IDs may no longer contain spaces or newlines.
- `khqr-core`: timestamps must be digits. `+1739495778722` used to parse.
- `khqr-api`: a Bakong side error on a transaction check was reported as
  "not paid yet". During an outage a merchant was told the customer had not
  paid, and the documented poll loop kept polling, with no error ever surfacing.
- `khqr-api`: batch results are checked against the request. A short response
  used to shift every later result onto the wrong transaction.
- `khqr-api`: an item Bakong marked `SUCCESS` that failed to deserialise was
  reported as unpaid. It is now an error.
- `khqr-api`: requests carry a 30 second timeout. There was none, so a stalled
  connection could hang a poll indefinitely.
- `khqr-api`: a 401 whose body is not JSON stays `Unauthorized`, and a 403 says
  it usually means the request came from outside Cambodia.
- `khqr-api`: renewal checks that Bakong accepted it before storing the token.
- `khqr-api`: building with no TLS feature is now a compile error rather than a
  client that fails every request at runtime.
- `khqr-cli`: `watch` no longer gives up to a minute early, so a payment inside
  the window you asked for is seen.
- `khqr-cli`: `--expires-in` and `--timeout` no longer overflow. In release
  builds `--expires-in` wrapped, producing a QR that had already expired.
- `khqr-cli`: `BAKONG_EMAIL` no longer prints its value in `--help`.
- `khqr-wasm`: an unrecognised currency string threw instead of falling back to
  riel, which billed 2 riel for a 1.50 dollar charge.
- `khqr-wasm`: a rejected `build()` no longer destroys the builder.
- `khqr-ffi`: setting both `account_information` and `merchant_id` is an error
  rather than silently dropping one, and a partly filled alternate language
  template is an error rather than vanishing.

### Changed

- Minimum supported Rust version is 1.88, and CI now checks it. The previously
  declared 1.75 was never achievable: `image` requires 1.88, `reqwest` and
  `clap` require 1.85.
- `INDIVIDUAL_USD_FULL` in the test vectors is renamed `INDIVIDUAL_WITH_LABELS`.
  It was never a dollar payload; it carries riel with a fractional amount.

## 0.1.0

### Added

- `khqr-core`: TLV codec, CRC-16/CCITT-FALSE, typed builder, decoder, MD5
  handle, and PNG, SVG and data URI rendering behind the `image` feature.
- `khqr-core`: UnionPay merchant account, tag `15`, on both the build and
  decode sides.
- `khqr-core`: `no_std` support. Turn off the `std` feature and the crate
  builds for bare metal, checked in CI against `thumbv7em-none-eabihf`.
- `khqr-api`: client for all ten Bakong Open API endpoints, with token renewal
  on rejection and a `Backoff` the caller drives.
- `khqr-cli`: `khqr gen`, `decode`, `verify` and `watch`.
- `khqr-wasm`: browser and Node bindings over the codec.
- `khqr-ffi`: Kotlin, Swift and Python bindings through UniFFI.
- Fuzz targets for `decode` and `parse_tlv`, plus a deterministic hostile
  input test that runs on stable.

### Notes

- Expiry is not required on dynamic QRs, though the official SDK has required
  it since npm `bakong-khqr` v1.0.18. Two of the published test vectors carry an
  amount with no expiry, so enforcing it would reject the official test data. This will be revisited before 1.0.
- Amounts decode as strings rather than numbers, so a decode then encode round
  trip reproduces the input byte for byte.

## Compatibility

Minimum supported Rust version is 1.88, checked in CI. Before 1.0 it can rise
in any release, as it did in 0.1.1; after 1.0 that becomes a major change.
