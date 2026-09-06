# khqr-rs documentation

KHQR is the payment QR standard used by Bakong in Cambodia. It is EMVCo
merchant presented QR: a flat ASCII string of nested tag length value triples
ending in a CRC16 checksum.

This library splits that work into pieces you can take separately. The codec
has no async runtime and no HTTP, so it runs in a browser, on a POS terminal
and inside a phone app. Network calls live in their own crate that you only
pull in if you need them.

## Where to start

| If you want to | Read |
| --- | --- |
| Install it and make your first QR | [Getting started](getting-started.md) |
| Build a payload with the right fields | [Generating](generating.md) |
| Read a payload back, or validate one | [Decoding](decoding.md) |
| Know when a customer has paid | [Checking payments](payments.md) |
| Understand the format itself | [The KHQR format](format.md) |

## What each crate is for

| Crate | Purpose | Pull it in when |
| --- | --- | --- |
| `khqr-core` | Codec, checksum, builder, decoder, MD5, images | Always. Everything else builds on it. |
| `khqr-api` | Bakong Open API client | You need to know whether a payment arrived. |
| `khqr-cli` | The `khqr` command | You want to try things, or script them. |
| `khqr-wasm` | Browser and Node | You generate QR codes in a web front end. |
| `khqr-ffi` | Kotlin, Swift, Python | You are writing an Android, iOS or Python app. |

`khqr-core` is the only one that is required. It has three dependencies, and
two of those are optional and only used for rendering images.

## The one thing to get right

A QR with an amount is single use. Its MD5 handle identifies one payment, and
that is what you poll for. A QR without an amount is reusable, and Bakong
cannot tell you anything about it, because there is no single transaction to
look up. Your bank has to check its own records instead.

If you are building a checkout, you want an amount.

## Is it proven

Yes, on the path that matters. A QR built by this library was scanned by a
banking app, paid, and then found again through the API by MD5 handle, full
hash, short hash, external reference and hash batch, against production.

The codec is checked against the four published KHQR vectors on every commit,
and the decoder is fuzzed.

## Versioning

The API will change before 1.0. Pin an exact version until then.

Minimum supported Rust version is 1.88, checked in CI against that toolchain.
