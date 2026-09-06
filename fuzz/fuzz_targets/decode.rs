#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let _ = khqr_core::decode(data);
    let _ = khqr_core::verify_crc(data);
});
