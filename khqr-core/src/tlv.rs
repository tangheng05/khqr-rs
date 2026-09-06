use crate::KhqrError;

/// One tag-length-value triple.
///
/// The value of a template tag such as `62` is itself a run of triples, so
/// feeding it back to [`parse_tlv`] descends one level.
///
/// # Examples
///
/// ```
/// use khqr_core::Tlv;
///
/// let field = Tlv { tag: "58".to_string(), value: "KH".to_string() };
/// assert_eq!(field.tag, "58");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tlv {
    /// Two ASCII digits identifying the field.
    pub tag: String,
    /// The field's payload, at most 99 characters.
    pub value: String,
}

/// Writes one triple: the tag, the value's length as two digits, then the value.
///
/// The length is a count of characters, matching the EMVCo specification and
/// the behaviour of the reference JavaScript SDK for Khmer text.
///
/// # Errors
///
/// Returns [`KhqrError::InvalidTag`] if the tag is not two ASCII digits, and
/// [`KhqrError::ValueTooLong`] if the value exceeds 99 characters.
///
/// # Examples
///
/// ```
/// use khqr_core::format_tlv;
///
/// assert_eq!(format_tlv("00", "01")?, "000201");
/// assert_eq!(format_tlv("59", "Jonh Smith")?, "5910Jonh Smith");
/// # Ok::<(), khqr_core::KhqrError>(())
/// ```
pub fn format_tlv(tag: &str, value: &str) -> Result<String, KhqrError> {
    if !is_tag(tag) {
        return Err(KhqrError::InvalidTag {
            tag: tag.to_string(),
        });
    }

    let length = value.chars().count();
    if length > 99 {
        return Err(KhqrError::ValueTooLong {
            tag: tag.to_string(),
            chars: length,
        });
    }

    Ok(format!("{tag}{length:02}{value}"))
}

/// Reads a run of triples until the input is exhausted.
///
/// An empty input yields an empty vector. Unknown tags are returned as they
/// are found; deciding what they mean is the decoder's job, not this one's.
///
/// # Errors
///
/// Returns [`KhqrError::InvalidTag`] or [`KhqrError::InvalidLength`] for a
/// malformed header, [`KhqrError::UnexpectedEnd`] if the input stops part way
/// through a header, and [`KhqrError::Truncated`] if a value is cut short.
///
/// # Examples
///
/// ```
/// use khqr_core::parse_tlv;
///
/// let fields = parse_tlv("0002015802KH")?;
/// assert_eq!(fields[0].value, "01");
/// assert_eq!(fields[1].tag, "58");
/// # Ok::<(), khqr_core::KhqrError>(())
/// ```
pub fn parse_tlv(input: &str) -> Result<Vec<Tlv>, KhqrError> {
    // Collected up front so the loop can slice by character. Indexing a &str
    // by byte would split a Khmer character in half and panic.
    let chars: Vec<char> = input.chars().collect();
    let mut fields = Vec::new();
    let mut pos = 0;

    while pos < chars.len() {
        let value_start = pos + 4;
        if value_start > chars.len() {
            return Err(KhqrError::UnexpectedEnd { offset: pos });
        }

        let tag: String = chars[pos..pos + 2].iter().collect();
        if !is_tag(&tag) {
            return Err(KhqrError::InvalidTag { tag });
        }

        let field: String = chars[pos + 2..value_start].iter().collect();
        let Some(declared) = two_digit_value(&field) else {
            return Err(KhqrError::InvalidLength { tag, length: field });
        };

        let available = chars.len() - value_start;
        if declared > available {
            return Err(KhqrError::Truncated {
                tag,
                declared,
                available,
            });
        }

        let value_end = value_start + declared;
        fields.push(Tlv {
            tag,
            value: chars[value_start..value_end].iter().collect(),
        });
        pos = value_end;
    }

    Ok(fields)
}

fn is_tag(tag: &str) -> bool {
    two_digit_value(tag).is_some()
}

fn two_digit_value(field: &str) -> Option<usize> {
    let mut digits = field.chars();
    let tens = digits.next()?.to_digit(10)?;
    let units = digits.next()?.to_digit(10)?;

    if digits.next().is_some() {
        return None;
    }

    Some((tens * 10 + units) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "Phnom Penh" in Khmer: seven characters, twenty one bytes.
    const KHMER_CITY: &str = "ភ្នំពេញ";

    #[test]
    fn format_pads_the_length_to_two_digits() {
        assert_eq!(format_tlv("00", "01").unwrap(), "000201");
        assert_eq!(format_tlv("53", "116").unwrap(), "5303116");
        assert_eq!(format_tlv("59", "Jonh Smith").unwrap(), "5910Jonh Smith");
    }

    #[test]
    fn format_accepts_an_empty_value() {
        assert_eq!(format_tlv("59", "").unwrap(), "5900");
    }

    #[test]
    fn format_rejects_a_value_over_99_characters() {
        let long = "x".repeat(100);

        assert_eq!(
            format_tlv("59", &long).unwrap_err(),
            KhqrError::ValueTooLong {
                tag: "59".to_string(),
                chars: 100
            }
        );
    }

    #[test]
    fn format_rejects_a_malformed_tag() {
        for tag in ["", "0", "000", "5a", "  ", "-1"] {
            assert!(format_tlv(tag, "x").is_err(), "tag {tag:?} should fail");
        }
    }

    #[test]
    fn parse_returns_nothing_for_empty_input() {
        assert_eq!(parse_tlv("").unwrap(), vec![]);
    }

    #[test]
    fn parse_reads_consecutive_fields() {
        let fields = parse_tlv("0002015802KH5910Jonh Smith").unwrap();

        assert_eq!(fields.len(), 3);
        assert_eq!(
            fields[2],
            Tlv {
                tag: "59".to_string(),
                value: "Jonh Smith".to_string()
            }
        );
    }

    #[test]
    fn parse_accepts_a_zero_length_value() {
        assert_eq!(
            parse_tlv("5900").unwrap(),
            vec![Tlv {
                tag: "59".to_string(),
                value: String::new()
            }]
        );
    }

    #[test]
    fn a_template_value_parses_again() {
        assert_eq!(
            parse_tlv("0014jonhsmith@nbcq").unwrap(),
            vec![Tlv {
                tag: "00".to_string(),
                value: "jonhsmith@nbcq".to_string()
            }]
        );
    }

    #[test]
    fn parse_rejects_a_header_cut_short() {
        assert_eq!(
            parse_tlv("590").unwrap_err(),
            KhqrError::UnexpectedEnd { offset: 0 }
        );
        assert_eq!(
            parse_tlv("00020159").unwrap_err(),
            KhqrError::UnexpectedEnd { offset: 6 }
        );
    }

    #[test]
    fn parse_rejects_a_truncated_value() {
        assert_eq!(
            parse_tlv("5905ab").unwrap_err(),
            KhqrError::Truncated {
                tag: "59".to_string(),
                declared: 5,
                available: 2
            }
        );
    }

    #[test]
    fn parse_rejects_a_non_numeric_header() {
        assert_eq!(
            parse_tlv("5x02KH").unwrap_err(),
            KhqrError::InvalidTag {
                tag: "5x".to_string()
            }
        );
        assert_eq!(
            parse_tlv("58x2KH").unwrap_err(),
            KhqrError::InvalidLength {
                tag: "58".to_string(),
                length: "x2".to_string()
            }
        );
    }

    #[test]
    fn length_counts_characters_not_bytes() {
        assert_eq!(KHMER_CITY.len(), 21);
        assert_eq!(KHMER_CITY.chars().count(), 7);

        let encoded = format_tlv("60", KHMER_CITY).unwrap();
        assert_eq!(encoded, format!("6007{KHMER_CITY}"));
        assert_eq!(
            parse_tlv(&encoded).unwrap(),
            vec![Tlv {
                tag: "60".to_string(),
                value: KHMER_CITY.to_string()
            }]
        );
    }

    #[test]
    fn a_truncated_multibyte_value_errors_instead_of_panicking() {
        assert_eq!(
            parse_tlv("6007ភ្នំ").unwrap_err(),
            KhqrError::Truncated {
                tag: "60".to_string(),
                declared: 7,
                available: 4
            }
        );
    }
}
