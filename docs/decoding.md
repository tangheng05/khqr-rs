# Decoding and validating

```rust
use khqr_core::decode;

let decoded = decode(payload)?;

println!("{}", decoded.merchant_name);
println!("{}", decoded.bakong_account_id);

if let Some(amount) = &decoded.transaction_amount {
    println!("{amount} {}", decoded.transaction_currency);
}
```

`decode` returns one flat struct. Every nested template is unpacked into it,
so you never walk a tree to find a bill number.

## Two rules

**Strict about the checksum.** A payload whose CRC does not match its contents
is an error, always. You get `ChecksumMismatch` with both the checksum the
payload carried and the one its contents produce.

**Lenient about unknown tags.** Real bank QRs carry vendor extensions that are
in no published table. The ABA production vector in our test suite has a tag
`68` nested inside tag `62` holding PayWay routing data. A parser that rejects
what it does not recognise rejects real money.

This leniency covers tags the crate does not recognise. A *repeat* of a tag it
does know is refused, since the checksum is not a signature and a duplicate
would let someone change the currency or the destination account.

Unknown tags are kept rather than dropped:

```rust
for field in decoded.unknown.iter().chain(&decoded.additional_unknown) {
    println!("tag {} = {}", field.tag, field.value);
}
```

`unknown` holds top level tags, `additional_unknown` holds sub tags found
inside tag `62`. Sub tags of the account template and of tags `64` and `99` are
not kept, so do not rely on this to audit a payload byte for byte.

## Amounts stay as strings

`transaction_amount` is an `Option<String>`, not a number. This is deliberate.

One of the published vectors carries `5000.0` in riel, which the specification
says should have no decimal places at all. Parsing that into a float and
writing it back would silently rewrite what the bank actually sent. Keeping
the original string means a decode then encode round trip reproduces the input
byte for byte.

Use `amount()` when you need arithmetic:

```rust
match decoded.amount() {
    Some(amount) => charge(amount),
    None => reject("no amount, or one that is not a number"),
}
```

It returns `None` for anything that is not a finite, non negative number.
Parsing the raw string yourself is a trap: `"NaN"` parses successfully, and
every comparison against it is false, so a guard like `amount < expected`
would pass.

`decoded.currency()` gives you a typed `Currency` when tag `53` held a code
this library knows, and `None` otherwise.

## Just checking the checksum

`verify_crc` is cheaper than a full decode and does not allocate a struct:

```rust
use khqr_core::verify_crc;

if !verify_crc(payload) {
    return Err("that QR is damaged");
}
```

It accepts hex in either case, and returns `false` rather than panicking on
anything malformed, including a payload that ends part way through a multi
byte character.

## What you get back

| Always present | |
| --- | --- |
| `merchant_type` | `Individual` or `Merchant` |
| `bakong_account_id` | `name@bank` |
| `payload_format_indicator` | `01` |
| `point_of_initiation_method` | `11` static, `12` dynamic |
| `merchant_category_code` | four digits |
| `transaction_currency` | `116` riel, `840` dollars |
| `country_code`, `merchant_name`, `merchant_city` | |
| `crc` | four hex characters |

| Present when the payload has them | |
| --- | --- |
| `transaction_amount` | dynamic QRs only |
| `account_information`, `merchant_id`, `acquiring_bank` | tag `29` or `30` |
| `union_pay_merchant` | tag `15` |
| `bill_number`, `mobile_number`, `store_label`, `reference_label`, `terminal_label`, `purpose_of_transaction` | tag `62` |
| `language_preference`, `merchant_name_alternate`, `merchant_city_alternate` | tag `64` |
| `created_at_ms`, `expires_at_ms` | tag `99` |

## Errors

| Variant | Means |
| --- | --- |
| `ChecksumMismatch` | The CRC does not match. Carries what was found and what was expected. |
| `MissingField` | A required tag is absent, named in plain words rather than by number. |
| `UnexpectedEnd` | The payload stops part way through a tag and length. |
| `Truncated` | A field declares more characters than remain. |
| `InvalidTag`, `InvalidLength` | A tag or length was not two digits. |
| `DuplicateTag` | A tag appeared twice. Leniency covers tags we do not know, not repeats of ones we do, because a repeat lets someone append a field and recompute the checksum. |
| `InvalidField` | A timestamp that was not digits. |

`KhqrError` is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.

None of these panic. `decode` takes hostile input by design, so every path out
of it is a `Result`.

## Working at the TLV level

If you need the raw triples, for a non KHQR EMV payload or for debugging:

```rust
use khqr_core::{format_tlv, parse_tlv};

let fields = parse_tlv(payload)?;
let rebuilt: String = fields
    .iter()
    .map(|f| format_tlv(&f.tag, &f.value).unwrap())
    .collect();

assert_eq!(rebuilt, payload);
```

`parse_tlv` does not care whether a payload is KHQR. It reads tag length value
triples until the input runs out, and a template's value fed back into it
descends one level.
