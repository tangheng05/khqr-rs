/// The currency a payment is denominated in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    /// Cambodian riel, written to tag `53` as `116`.
    Khr,
    /// US dollar, written to tag `53` as `840`.
    Usd,
}

impl Currency {
    /// The EMV numeric code for tag `53`.
    pub fn code(self) -> &'static str {
        match self {
            Self::Khr => "116",
            Self::Usd => "840",
        }
    }

    /// Decimal places used when writing an amount. Riel carries none.
    pub fn decimals(self) -> usize {
        match self {
            Self::Khr => 0,
            Self::Usd => 2,
        }
    }
}

/// Whether the account behind the QR is a person or a business.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerchantType {
    /// Account details live in tag `29`.
    Individual,
    /// Account details live in tag `30`.
    Merchant,
}

impl MerchantType {
    /// The tag its account template is written to.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Individual => "29",
            Self::Merchant => "30",
        }
    }
}
