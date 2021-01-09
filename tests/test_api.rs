use fast_float::{parse, parse_partial, FastFloat};

macro_rules! check_ok {
    ($s:expr, $x:expr) => {
        let s = $s;
        check_ok!(s, $x, f32);
        check_ok!(s.as_bytes(), $x, f32);
        check_ok!(s, $x, f64);
        check_ok!(s.as_bytes(), $x, f64);
    };
    ($s:expr, $x:expr, $ty:ty) => {
        assert_eq!(<$ty>::parse_float($s).unwrap(), $x);
        assert_eq!(<$ty>::parse_float_partial($s).unwrap(), ($x, $s.len()));
        assert_eq!(parse::<$ty, _>($s).unwrap(), $x);
        assert_eq!(parse_partial::<$ty, _>($s).unwrap(), ($x, $s.len()));
    };
}

macro_rules! check_ok_partial {
    ($s:expr, $x:expr, $n:expr) => {
        let s = $s;
        check_ok_partial!(s, $x, $n, f32);
        check_ok_partial!(s.as_bytes(), $x, $n, f32);
        check_ok_partial!(s, $x, $n, f64);
        check_ok_partial!(s.as_bytes(), $x, $n, f64);
    };
    ($s:expr, $x:expr, $n:expr, $ty:ty) => {
        assert!(<$ty>::parse_float($s).is_err());
        assert_eq!(<$ty>::parse_float_partial($s).unwrap(), ($x, $n));
        assert!(parse::<$ty, _>($s).is_err());
        assert_eq!(parse_partial::<$ty, _>($s).unwrap(), ($x, $n));
    };
}

macro_rules! check_err {
    ($s:expr) => {
        let s = $s;
        check_err!(s, f32);
        check_err!(s.as_bytes(), f32);
        check_err!(s, f64);
        check_err!(s.as_bytes(), f64);
    };
    ($s:expr, $ty:ty) => {
        assert!(<$ty>::parse_float($s).is_err());
        assert!(<$ty>::parse_float_partial($s).is_err());
        assert!(parse::<$ty, _>($s).is_err());
        assert!(parse_partial::<$ty, _>($s).is_err());
    };
}

#[test]
fn test_api() {
    check_ok!("1.23", 1.23);
    check_ok!("0.", 0.);
    check_ok!("-0", 0.);
    check_ok!("+00", 0.);
    check_ok!("-0001e-02", -0.01);
    check_ok!("345", 345.);

    check_ok_partial!("1a", 1., 1);
    check_ok_partial!("-2e-1x", -0.2, 5);
    check_ok_partial!("2e2.", 200., 3);
    check_ok_partial!("2ea", 2., 1);

    check_err!("");
    check_err!(" ");
    check_err!(".");
    check_err!(".e1");
    check_err!("+");
    check_err!("-");
    check_err!("x");
    check_err!("a123");
}
