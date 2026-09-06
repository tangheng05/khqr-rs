# Getting started

Pick the section for your language. Every one of them produces the same
payload string, because they all wrap the same Rust codec.

## Rust

```toml
[dependencies]
khqr-core = "0.1"
```

Add the `image` feature if you want PNG or SVG output:

```toml
khqr-core = { version = "0.1", features = ["image"] }
```

```rust
use khqr_core::{md5, Khqr};

let qr = Khqr::individual("shop@aclb")
    .merchant_name("Coffee Klaing")
    .merchant_city("Phnom Penh")
    .amount(5000.0)
    .build()?
    .to_qr_string()?;

let handle = md5(&qr);
```

`qr` is the string you turn into a QR image. `handle` is what you give to
Bakong later to ask whether it was paid.

## Command line

```sh
cargo install khqr-cli
```

```sh
khqr gen --account shop@aclb --name "Coffee Klaing" --city "Phnom Penh" \
    --amount 5000 --expires-in 300 --png qr.png
```

It prints the payload, then the MD5 handle, then any files it wrote:

```
00020101021229130009shop@aclb52045999530311654045000...63048AF3
md5 682f33ec80e311d909f91d70a70ab436
png qr.png
```

`khqr decode <payload>` prints every field. `khqr verify <payload>` exits
non zero if the checksum is wrong, so it drops into a shell script without
any parsing.

## Browser and Node

```sh
wasm-pack build khqr-wasm --target web
```

```js
import init, { Khqr, toDataUri, md5 } from "./pkg/khqr_wasm.js";

await init();

const builder = Khqr.individual("shop@aclb");
builder.merchantName("Coffee Klaing");
builder.merchantCity("Phnom Penh");
builder.amount(5000);

const qr = builder.build();
document.querySelector("img").src = toDataUri(qr, 512);
```

Setters mutate the builder rather than returning it. That is deliberate: a
chained call would move the object and free the handle you were holding.

Nothing here talks to the network, so a web front end can generate a QR with
no server round trip. Checking payment still needs a server, because that
call carries your Bakong token and you must not ship that to a browser.

## Kotlin, Swift and Python

Build the library, then generate bindings for the languages you want:

```sh
cargo build -p khqr-ffi --release

cargo run -p khqr-ffi --bin uniffi-bindgen -- generate \
    --library target/release/libkhqr_ffi.so \
    --language kotlin --language swift --language python \
    --out-dir bindings
```

On macOS the library is `libkhqr_ffi.dylib`, and on Windows `khqr_ffi.dll`.

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

Everything after the first four fields has a default, so you only name what
you actually set.

Dart is not one of the languages UniFFI generates. The third party
`uniffi-dart` bindgen reads the same compiled library, and the Rust side
needs no changes for it.

## Checking payments

Generating needs no token. Checking whether a QR was paid does, and that call
belongs on your server rather than in a browser or a phone app.

```rust
use khqr_api::{BakongClient, Environment};

let client = BakongClient::new(Environment::Production, token);
let status = client.check_transaction_by_md5(&handle).await?;
```

See [checking payments](payments.md) for polling, token renewal and the traps.

## Next

[Generating](generating.md) covers the rest of the fields and the rules the
builder enforces.
