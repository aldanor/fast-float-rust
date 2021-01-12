#![no_main]
use libfuzzer_sys::fuzz_target;

// We only test the roundtrip of f64 with a fuzzer because f32 search space
// is small enough that we can test it exhaustively

fn check_roundtrip(float: f64, string: impl AsRef<str>) {
    let result = ::fast_float::parse::<f64, _>(string.as_ref()).unwrap();
    if float.is_nan() {
        assert!(result.is_nan());
    } else {
        assert_eq!(float, result);
    }
}

fuzz_target!(|float: f64| {
    // we use both ryu and stdlib since ryu prefers scientific notation while stdlib
    // never uses it at all; hence more code paths will be executed
    let mut buf = ryu::Buffer::new();
    check_roundtrip(float, buf.format(float));
    check_roundtrip(float, float.to_string());
});
