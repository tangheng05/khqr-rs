use std::fmt;

/// Everything that can go wrong reading or writing a KHQR payload.
///
/// Offsets and lengths count characters, not bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KhqrError {
    /// A tag was not two ASCII digits.
    InvalidTag {
        /// The offending tag.
        tag: String,
    },
    /// A length field was not two ASCII digits.
    InvalidLength {
        /// The tag the length belonged to.
        tag: String,
        /// The offending length field.
        length: String,
    },
    /// The payload ran out before a full four character header could be read.
    UnexpectedEnd {
        /// Where the partial header starts.
        offset: usize,
    },
    /// A field declared more characters than the payload had left.
    Truncated {
        /// The tag whose value was cut short.
        tag: String,
        /// Characters the length field promised.
        declared: usize,
        /// Characters actually remaining.
        available: usize,
    },
    /// A required field was never set.
    MissingField {
        /// Name of the missing field.
        field: &'static str,
    },
    /// A field was longer than the specification allows.
    FieldTooLong {
        /// Name of the field.
        field: &'static str,
        /// Length in characters.
        chars: usize,
        /// Longest the field may be.
        max: usize,
    },
    /// A field was set to something the specification does not accept.
    InvalidField {
        /// Name of the field.
        field: &'static str,
        /// The rejected value.
        value: String,
    },
    /// A value was longer than a two digit length field can express.
    ValueTooLong {
        /// The tag being written.
        tag: String,
        /// Length of the value in characters.
        chars: usize,
    },
}

impl fmt::Display for KhqrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTag { tag } => {
                write!(f, "tag must be two ascii digits, found {tag:?}")
            }
            Self::InvalidLength { tag, length } => {
                write!(
                    f,
                    "length of tag {tag} must be two ascii digits, found {length:?}"
                )
            }
            Self::UnexpectedEnd { offset } => {
                write!(
                    f,
                    "payload ends at character {offset}, mid way through a tag and length"
                )
            }
            Self::Truncated {
                tag,
                declared,
                available,
            } => {
                write!(
                    f,
                    "tag {tag} declares {declared} characters but only {available} remain"
                )
            }
            Self::ValueTooLong { tag, chars } => {
                write!(f, "tag {tag} value is {chars} characters, the limit is 99")
            }
            Self::MissingField { field } => {
                write!(f, "{field} is required")
            }
            Self::FieldTooLong { field, chars, max } => {
                write!(f, "{field} is {chars} characters, the limit is {max}")
            }
            Self::InvalidField { field, value } => {
                write!(f, "{field} {value:?} is not valid")
            }
        }
    }
}

impl std::error::Error for KhqrError {}
