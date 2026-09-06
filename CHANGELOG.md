# Changelog

This project follows [semantic versioning](https://semver.org). Until 1.0 the
minor version is where breaking changes land, so pin an exact version.

## Unreleased

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
  it since npm `bakong-khqr` v1.0.18. Three of the four published test vectors
  carry an amount with no expiry, so enforcing it would reject the official
  test data. This will be revisited before 1.0.
- Amounts decode as strings rather than numbers, so a decode then encode round
  trip reproduces the input byte for byte.

## Compatibility

Minimum supported Rust version is 1.75, checked in CI. Raising it is a minor
version change before 1.0 and a major one after.
