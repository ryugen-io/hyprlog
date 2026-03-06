#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // Must not panic on any input — invalid JSON simply returns Err.
    let _ = hyprs_log::server::WireRecord::from_line(data);
});
