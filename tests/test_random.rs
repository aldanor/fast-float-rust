#[test]
#[ignore]
fn test_f64_random_from_u64() {
    const N_ITER: u64 = 1 << 32;

    let rng = fastrand::Rng::with_seed(0);
    let mut buf = ryu::Buffer::new();
    for _ in 0..N_ITER {
        let i: u64 = rng.u64(0..0xFFFF_FFFF_FFFF_FFFF);
        let a: f64 = unsafe { core::mem::transmute(i) };
        let s = buf.format(a);
        let b: f64 = fast_float::parse(s).unwrap();
        assert!(a == b || (a.is_nan() && b.is_nan()));
    }
}
