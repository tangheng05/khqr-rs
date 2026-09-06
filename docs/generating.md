# Generating a payload

`Khqr::individual` is for a personal Bakong account, `Khqr::merchant` for a
business. The difference is which tag the account details go in, `29` or `30`,
and that a merchant can carry a merchant ID.

```rust
use khqr_core::{Currency, Khqr};

let qr = Khqr::merchant("shop@aclb")
    .merchant_name("Coffee Klaing")
    .merchant_city("Phnom Penh")
    .merchant_id("125021214532846")
    .acquiring_bank("ACLEDA Bank")
    .currency(Currency::Usd)
    .amount(1.5)
    .bill_number("INV-2003")
    .terminal_label("POS-1")
    .created_at_ms(now_ms)
    .expires_at_ms(now_ms + 300_000)
    .build()?
    .to_qr_string()?;
```

Nothing is written until `build()`, which is where the checking happens. If a
field is wrong you get a `KhqrError` naming it, not a broken QR that fails
quietly at the till.

## Static or dynamic

You do not set this. It follows from whether you gave an amount.

| | Amount | Tag `01` | Reusable | Trackable by MD5 |
| --- | --- | --- | --- | --- |
| Static | absent | `11` | yes | no |
| Dynamic | present | `12` | no | yes |

A shop sticker on a counter is static. A checkout screen is dynamic. If you
want to know when a specific customer paid, you need dynamic, because the MD5
handle has to identify one transaction.

Deriving the tag from the amount means the two can never disagree. The
official SDK asks you for both and trusts you to keep them consistent.

## Amounts

Riel carries no decimal places, dollars carry two. The library formats the
number for you based on the currency:

| Currency | You pass | Value in the payload |
| --- | --- | --- |
| `Currency::Khr` | `5000.0` | `5000` |
| `Currency::Khr` | `500.7` | `501` |
| `Currency::Usd` | `1.5` | `1.50` |
| `Currency::Usd` | `0.1` | `0.10` |

Getting this wrong changes the string, which changes the checksum, which means
the QR fails to scan. It is the most common way to break a hand rolled
implementation, so the formatting is not optional here.

Riel is the default. Negative amounts, infinities and NaN are rejected.

## Fields and their limits

These come from the reference SDK and are enforced in `build()`.

| Field | Method | Max characters |
| --- | --- | --- |
| Bakong account ID | first argument | 32 |
| Account information | `account_information` | 32 |
| Merchant ID | `merchant_id` | 32 |
| Acquiring bank | `acquiring_bank` | 32 |
| Merchant name | `merchant_name` | 25 |
| Merchant city | `merchant_city` | 15 |
| Bill number | `bill_number` | 25 |
| Mobile number | `mobile_number` | 25 |
| Store label | `store_label` | 25 |
| Reference label | `reference_label` | 25 |
| Terminal label | `terminal_label` | 25 |
| Purpose of transaction | `purpose_of_transaction` | 25 |
| UnionPay merchant | `union_pay_merchant` | 99 |
| Amount | `amount` | 13, after formatting |
| Creation and expiry | `created_at_ms`, `expires_at_ms` | 13 digits |
| Merchant category code | `merchant_category_code` | exactly 4 digits |
| Country code | `country_code` | exactly 2 upper case letters |
| Language preference | `alternate_language` | exactly 2 letters |

Required text fields may not be empty, and no field may carry a control
character such as a newline or a null.

Name and city are required. The category code defaults to `5999`, which means
a general merchant, and the country code defaults to `KH`.

The account ID has to look like `name@bank`. One `@`, something on each side.

Counting is by character, not by byte. A Khmer name is three bytes per
character, and the length field in the payload counts characters, so the two
disagree if you are careless. This library counts characters everywhere.

## Khmer names

Tag `64` carries a second language version of the name and city:

```rust
.alternate_language("KM", "កាហ្វេ ក្លាំង", "ភ្នំពេញ")
```

The preference is two characters, and the name and city obey the same 25 and
15 limits as their Latin counterparts.

## Timestamps

`created_at_ms` and `expires_at_ms` are epoch milliseconds. Both go in tag
`99`, and both are optional here.

The official SDK requires an expiry on any QR with an amount, from npm
`bakong-khqr` v1.0.18 onward. We do not enforce that, because two of the
published test vectors carry an amount with no expiry, and rejecting the
official test data would be worse than being lenient. Set an expiry anyway if
you are generating for a real checkout. Confirm the exact behaviour against a
freshly generated QR from the official SDK before you rely on it.

## Images

With the `image` feature:

```rust
use khqr_core::{to_base64_uri, to_png, to_svg};

let png = to_png(&qr, 512)?;          // bytes, at least 512 pixels wide
let svg = to_svg(&qr)?;               // scales on its own
let uri = to_base64_uri(&qr, 512)?;   // data:image/png;base64,...
```

The PNG side is rounded up to a whole number of pixels per module, so the
image stays sharp. A fractional scale is what makes generated QR codes blurry
and hard for a phone to read.

### Drawing your own design around it

The output is a plain black and white code on purpose, so you can lay it out
however you like. Most web front ends put the merchant name, amount and the
KHQR mark around it in HTML and absolutely position a logo over the middle,
which stays crisp at any size and needs no image compositing.

Two things matter if you do that.

```rust
use khqr_core::{to_png_with, ErrorCorrection, ImageOptions};

let png = to_png_with(&qr, &ImageOptions {
    size: 512,
    error_correction: ErrorCorrection::High,
    quiet_zone: 0,
})?;
```

**Error correction.** Anything covering the middle of a code eats into what the
reader can recover. The default, `Medium`, tolerates about 15%. A typical
centred roundel covers close to 18%, which is over budget: the code then scans
on one phone and fails on the next. Use `High`, which tolerates about 30%.
`ImageOptions::with_overlay()` is that setting under a shorter name.

**Quiet zone.** The specification asks for four blank modules around the code,
and that is the default. If your own card already pads the image, set it to
zero rather than paying for the margin twice.

The KHQR logo and card artwork belong to the National Bank of Cambodia and
must be used unmodified. They are deliberately not bundled here. Overlay them
yourself from the official source if your design calls for it.

## Errors

`build()` and `to_qr_string()` both return `Result`. The variants you will see:

| Variant | Means |
| --- | --- |
| `MissingField` | Name or city was not set. |
| `FieldTooLong` | A field is over its limit. Carries the name, the length and the limit. |
| `InvalidField` | Account ID, amount, category code, country code or language preference was rejected. |
| `ValueTooLong` | A whole template went past 99 characters. |

The last one is the reason `to_qr_string()` returns a `Result` at all. Each
label can be valid on its own while the additional data template they share
overflows. It is rare, and it is not a panic.
