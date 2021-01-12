#[test]
#[ignore]
fn test_f32_exhaustive_ryu() {
    let mut buf = ryu::Buffer::new();
    for i in 0..0xFFFF_FFFF_u32 {
        let a: f32 = unsafe { core::mem::transmute(i) };
        let s = buf.format(a);
        let b: f32 = fast_float::parse(s).unwrap();
        assert!(a == b || (a.is_nan() && b.is_nan()));
    }
}
