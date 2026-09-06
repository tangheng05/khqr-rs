# The KHQR format

You do not need this to use the library. It is here for when something does
not scan and you want to read the string yourself.

KHQR is EMVCo merchant presented QR. A payload is a flat ASCII string of tag
length value triples, with no separators.

```
00 02 01
│  │  └── the value
│  └───── how many characters the value is, always two digits, zero padded
└──────── the tag, always two digits
```

So `000201` is tag `00`, two characters long, holding `01`.

Read one triple, jump past it, read the next. Templates work the same way:
their value is another run of triples, so you recurse.

The length counts **characters**, not bytes. A Khmer merchant name is three
bytes per character, so a seven character name has a length of `07` while
occupying 21 bytes. Byte counting is the single most common bug in hand
written implementations, and in Rust it also panics, because slicing a string
in the middle of a character is not allowed.

## Top level tags

| Tag | Name | Notes |
| --- | --- | --- |
| `00` | Payload format indicator | always `01` |
| `01` | Point of initiation | `11` static, `12` dynamic |
| `15` | UnionPay merchant account | optional |
| `29` | Individual account | nested, excludes `30` |
| `30` | Merchant account | nested, excludes `29` |
| `52` | Merchant category code | four digits, `5999` by default |
| `53` | Transaction currency | `116` riel, `840` dollars |
| `54` | Transaction amount | absent entirely on a static QR |
| `58` | Country code | `KH` |
| `59` | Merchant name | max 25 |
| `60` | Merchant city | max 15 |
| `62` | Additional data | nested, optional |
| `64` | Alternate language | nested, optional |
| `99` | Timestamp | nested, optional |
| `63` | CRC | always last, four upper case hex |

Tags appear in ascending order, with `63` last no matter what.

## Nested tags

**Tag `29`, individual account.** `00` Bakong account ID, `01` account
information, `02` acquiring bank.

**Tag `30`, merchant account.** `00` Bakong account ID, `01` merchant ID,
`02` acquiring bank.

**Tag `62`, additional data.** `01` bill number, `02` mobile number, `03`
store label, `05` reference label, `07` terminal label, `08` purpose of
transaction. Note there is no `04` or `06` in KHQR.

**Tag `64`, alternate language.** `00` language preference, `01` merchant name
in that language, `02` merchant city in that language.

**Tag `99`, timestamp.** `00` creation time, `01` expiry, both epoch
milliseconds written as a string.

Banks also emit tags that are in no published table. The ABA production vector
carries a tag `68` inside its tag `62`. Skip what you do not recognise.

## The checksum

CRC-16/CCITT-FALSE. Polynomial `0x1021`, initial value `0xFFFF`, no input or
output reflection, no final XOR.

It is computed over the whole payload **including** the literal `6304` that
introduces the checksum field, and written after it as four upper case hex
characters.

That last part trips people up. You do not checksum the payload and then
append `6304XXXX`. You append `6304`, checksum everything including it, then
append the four hex characters.

```rust
let mut payload = everything_else;
payload.push_str("6304");
let checksum = crc16_ccitt_false(payload.as_bytes());
payload.push_str(&format!("{checksum:04X}"));
```

The published check value for this variant is `crc16_ccitt_false(b"123456789")
== 0x29B1`. If yours matches that, the variant is right.

## The MD5 handle

Plain `md5` of the whole payload string, written as 32 lower case hex
characters. Computed locally, no API call.

It only identifies a transaction for a dynamic QR. A static QR has no amount,
so there is nothing single to look up.

## Worked example

```
00020101021229180014jonhsmith@nbcq52045999530311654035005802KH5910Jonh Smith6010PHNOM PENH99170013173949577872263046894
```

Broken up:

| Tag | Length | Value |
| --- | --- | --- |
| `00` | `02` | `01` |
| `01` | `02` | `12` |
| `29` | `18` | `0014jonhsmith@nbcq` |
| `52` | `04` | `5999` |
| `53` | `03` | `116` |
| `54` | `03` | `500` |
| `58` | `02` | `KH` |
| `59` | `10` | `Jonh Smith` |
| `60` | `10` | `PHNOM PENH` |
| `99` | `17` | `00131739495778722` |
| `63` | `04` | `6894` |

Tag `29` unpacks to sub tag `00`, 14 characters, `jonhsmith@nbcq`. Tag `99`
unpacks to sub tag `00`, 13 characters, `1739495778722`, which is February
2025.

Point of initiation is `12` and there is an amount, so this is a dynamic QR
worth 500 riel. Its MD5 is `b1c250304b8594e4c6b53dd44791b57a`.

## Sources

The tag numbers and field limits here were taken from
[`fidele007/bakong-khqr-php`](https://github.com/fidele007/bakong-khqr-php),
which tracks the npm `bakong-khqr` package closely, and checked against the
four published test vectors in `khqr-core/tests/common/mod.rs`.

The official specification is the NBC KHQR SDK guide, downloadable from
<https://bakong.nbc.gov.kh>.
