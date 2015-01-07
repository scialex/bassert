#![feature(plugin)]
#[macro_use] #[no_link] #[plugin] extern crate bassert;

#[test]
fn assert_with_message() {
    bassert!(0i == 0, "hello world");
    bassert!(0i == 0, "hello {}", "world");
    let bar = "world";
    bassert!(0i == 0, "{} {}", "hello", bar);
}

#[test]
fn test_complex_expression() {
    bassert!(1i + 1i == 2i);
    bassert!(1i + 1i != 1i);
    bassert!(1i + 1i >  1i);
    bassert!(1i + 1i >= 2i);
    bassert!(1i + 1i <  3i);
    bassert!(1i + 1i <= 3i);
    bassert!(true && true);
    if cfg!(fail) {
        bassert!(1i + 1i > 3i);
    }
}


#[test]
fn test_methods() {
    fn no_args() -> bool { true }
    fn one_arg(i: u8) -> bool { i == 3 }
    fn two_args(i: u8, _: &'static str) -> bool { i == 3}
    fn six_args(i: u8, _: &'static str, _: i8, _: u32, _: i32, _: i8) -> bool { i == 3}
    bassert!(no_args());
    bassert!(one_arg(3u8));
    bassert!(two_args(3, "hello"));
    let x = "hi";
    let y = 3;
    bassert!(two_args(y, x));
    bassert!(six_args(y, x, 9, 32, 44, 12), "WHY => {}", y);
    bassert!(!six_args(3 + y, x, 9, 32, 44, 12), "{} {}", "hello", "world");
    if cfg!(fail) {
        let double = |&: x:i32| -> i32 {x * 2};
        bassert!(six_args(y + 4, x, 3+2 * 2, 9|12, double(4), 12), "WHY => {}", y);
    }
}
