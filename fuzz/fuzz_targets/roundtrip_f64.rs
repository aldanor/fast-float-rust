#![no_main]
use libfuzzer_sys::fuzz_target;

// We only test the roundtrip of f64 with a fuzzer because f32 search space
// is small enough that we can test it exhaustively

fuzz_target!(|float: f64| {
    let roundtripped_float = ::fast_float::parse::<f64, _>(float.to_string()).unwrap();
    if float.is_nan() {
        assert!(roundtripped_float.is_nan());
    } else {
        assert_eq!(float, roundtripped_float)
    }
});
