#![no_main]

use libfuzzer_sys::fuzz_target;

fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = core::ptr::read_volatile(&dummy);
        core::mem::forget(dummy);
        ret
    }
}

fuzz_target!(|data: &[u8]| {
    let _ = black_box(::fast_float::parse::<f32, _>(data));
    let _ = black_box(::fast_float::parse::<f64, _>(data));
});
