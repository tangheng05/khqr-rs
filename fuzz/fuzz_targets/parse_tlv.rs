#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    if let Ok(fields) = khqr_core::parse_tlv(data) {
        for field in fields {
            let _ = khqr_core::format_tlv(&field.tag, &field.value);
        }
    }
});
