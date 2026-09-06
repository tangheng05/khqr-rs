use std::fmt;

/// Everything that can go wrong talking to Bakong.
#[derive(Debug)]
#[non_exhaustive]
pub enum ApiError {
    /// The request never completed, or the response was not JSON.
    Transport(reqwest::Error),
    /// The token was rejected and could not be renewed.
    Unauthorized {
        /// What Bakong said.
        message: String,
    },
    /// Bakong answered, but with an error of its own.
    Bakong {
        /// The `errorCode` field, when present.
        code: Option<i64>,
        /// The `responseMessage` field.
        message: String,
    },
    /// A non JSON error response, such as the 403 Bakong returns outside Cambodia.
    Http {
        /// The HTTP status code.
        status: u16,
    },
    /// A batch answered with a different number of results than were asked for.
    BatchMismatch {
        /// How many were asked about.
        requested: usize,
        /// How many came back.
        returned: usize,
    },
    /// A successful response arrived without the data it should carry.
    MissingData {
        /// The endpoint that was called.
        endpoint: &'static str,
    },
    /// Renewing needs the email the token was registered with.
    NoRenewalEmail,
    /// A batch endpoint takes at most 50 items.
    BatchTooLarge {
        /// How many were passed.
        count: usize,
        /// The most the endpoint accepts.
        max: usize,
    },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => write!(f, "request to bakong failed: {error}"),
            Self::Unauthorized { message } => write!(f, "bakong rejected the token: {message}"),
            Self::Bakong { code, message } => match code {
                Some(code) => write!(f, "bakong returned error {code}: {message}"),
                None => write!(f, "bakong returned an error: {message}"),
            },
            Self::Http { status } => match status {
                403 => write!(
                    f,
                    "bakong refused with http 403, which usually means the request came from outside cambodia"
                ),
                other => write!(f, "bakong returned http {other}"),
            },
            Self::BatchMismatch {
                requested,
                returned,
            } => write!(
                f,
                "asked about {requested} transactions but {returned} came back, so the results cannot be lined up"
            ),
            Self::MissingData { endpoint } => {
                write!(f, "{endpoint} reported success but returned no data")
            }
            Self::NoRenewalEmail => {
                write!(
                    f,
                    "no renewal email was set, so the token cannot be renewed"
                )
            }
            Self::BatchTooLarge { count, max } => {
                write!(f, "batch of {count} exceeds the limit of {max}")
            }
        }
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transport(error) => Some(error),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        Self::Transport(error)
    }
}
