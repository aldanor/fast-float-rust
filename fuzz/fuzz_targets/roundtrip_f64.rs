#![no_main]
use libfuzzer_sys::fuzz_target;

// We only test the roundtrip of f64 with a fuzzer because f32 search space
// is small enough that we can test it exhaustively

fuzz_target!(|float: f64| {
    // we use ryu instead of stdlib formatter because it exercises more paths:
    // it will format long floats in scientific notation, if appropriate,
    // while the std formatter will never do so.
    let mut buf = ryu::Buffer::new();
    let stringified_float = buf.format(float);
    let roundtripped_float = ::fast_float::parse::<f64, _>(stringified_float).unwrap();
    if float.is_nan() {
        assert!(roundtripped_float.is_nan());
    } else {
        assert_eq!(float, roundtripped_float)
    }
});
