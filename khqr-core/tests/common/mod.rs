//! Published KHQR payloads used as the reference vectors for every phase.
//!
//! Nothing here depends on `khqr_core`, so the vectors stay trustworthy while
//! the implementation moves.

// Each test binary pulls in the whole module but uses only part of it.
#![allow(dead_code)]

/// Individual account, KHR 500, no additional data.
pub const INDIVIDUAL_KHR_500: &str = "00020101021229180014jonhsmith@nbcq52045999530311654035005802KH5910Jonh Smith6010PHNOM PENH99170013173949577872263046894";

/// MD5 handle of [`INDIVIDUAL_KHR_500`].
pub const INDIVIDUAL_KHR_500_MD5: &str = "b1c250304b8594e4c6b53dd44791b57a";

/// Merchant account with merchant ID, acquiring bank and a mobile number.
pub const MERCHANT_KHR: &str = "00020101021130400014jonhsmith@nbcq01061234560208Dev Bank5204599953031165802KH5910Jonh Smith6009Siem Reap6215021185512345678991700131739495778722630433E1";

/// MD5 handle of [`MERCHANT_KHR`].
pub const MERCHANT_KHR_MD5: &str = "c0d2d74726f8e887f37a585cda3b3a79";

/// Individual account carrying bill number, store label and terminal label.
///
/// Note the amount is `5000.0` in riel, which the spec says takes no decimals.
/// It is kept exactly as published, which is why decoding keeps amounts as text.
pub const INDIVIDUAL_WITH_LABELS: &str = "00020101021229190015john_smith@devb52045999530311654065000.05802KH5910jonh smith6010Phnom Penh62360109#INV-20030313Coffee Klaing0702#299170013161302797275763049ACF";

/// Production ABA/PayWay merchant QR. Deeply nested tag 62, MCC 5987, 25 char name.
pub const ABA_MERCHANT: &str = "00020101021230510016abaakhppxxx@abaa01151250212145328460208ABA Bank52045987530311654031005802KH5925OLD ME 25 CHAR WINNER IP26010Phnom Penh62570115MC-REF-KH-1500068340010PAYWAY@ABA0208104514230604A2279934001317598053453370113175980552533763049FBD";

/// Every vector, in the order the roadmap introduces them.
pub const ALL: [&str; 4] = [
    INDIVIDUAL_KHR_500,
    MERCHANT_KHR,
    INDIVIDUAL_WITH_LABELS,
    ABA_MERCHANT,
];
