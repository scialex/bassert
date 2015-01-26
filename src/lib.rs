#![crate_type="dylib"]
#![crate_name="bassert"]
#![allow(unstable)]
#![feature(plugin_registrar, quote)]
#![doc(html_logo_url = "https://avatars.io/gravatar/d0ad9c6f37bb5aceac2d7ac95ba82607?size=large",
       html_favicon_url="https://avatars.io/gravatar/d0ad9c6f37bb5aceac2d7ac95ba82607?size=small")]


extern crate syntax;
extern crate rustc;

use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ast::{self, TokenTree};
use syntax::ptr::P;
use syntax::ext::base::{DummyResult, ExtCtxt, MacResult, MacExpr};
use rustc::plugin::Registry;

#[macro_export]
macro_rules! bassert{
    ($e:expr) => ({
        bassert!($e,"");
    });

    ($e:expr, $fmt:expr) => ({
        bassert!($e,"{}",$e);
    });
    ($e:expr, $fmt:expr, $($args:tt)*) => ({
        let f = internal_bassert!($e); f(&format_args!(concat!("\n",$fmt), $($args)*));
    });
}

#[macro_export]
macro_rules! debug_bassert{
    ($e:expr) => ({
        if cfg!(not(ndebug)) { bassert!($e) }
    });
    ($e:expr, $fmt:expr) => ({
        if cfg!(not(ndebug)) { bassert!($e, $fmt) }
    });
    ($e:expr, $fmt:expr, $($args:tt)*) => ({
        if cfg!(not(ndebug)) { bassert!($e, $fmt, $($args)*) }
    });
}

fn expand_bassert(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    // Create the parser for the arguments.
    let mut parser = cx.new_parser_from_tts(args);

    // Parse the expression to be tested.
    let expr: P<ast::Expr> = parser.parse_expr();

    if parser.token != token::Eof {
        cx.span_err(sp, "Non terminated internal bassert macro!");
        return DummyResult::any(sp);
    }
    MacExpr::new(get_fmt_meth(cx, expr))
}

/// Get the P<Expr> that is a callable function that can be passed to format_args! for execution of
/// the actual check.
fn get_fmt_meth(cx: &mut ExtCtxt, expr: P<ast::Expr>) -> P<ast::Expr> {
    // NB `rest` is the function argument sent in by the closure we wrap this around. By the time
    // this code is executed it will be in the environment.
    let inner_check = match expr.node {
        ast::ExprBinary(cmp, ref given, ref expected) => {
            let chk_fun = get_check_func(cx, cmp);
            quote_expr!(cx,
                match (&($given), &($expected)) {
                    (given_val, expected_val) => {
                        let chk = $chk_fun;
                        if !chk(given_val, expected_val) {
                            panic!("assertion failed: {}:\n\
                                    left:  `{}` = `{:?}`\n\
                                    right: `{}` = `{:?}`{}",
                                stringify!($expr), stringify!($given), *given_val,
                                stringify!($expected), *expected_val, rest);
                        }
                    }
                }
            )
        },
        ast::ExprCall(ref fun, ref args) if args.len() > 0 && args.len() < 7 => {
            // Annoyingly the way that macro's are expanded means we cannot use them to define
            // these, we need to manually place all of them ourselves.
            match args.len() {
                1 => {
                    let a = &args[0];
                    quote_expr!(cx,
                        match (&($a),) {
                            (a,) => {
                                if !($fun(*a)) {
                                    panic!("assertion failed: {}:\n\
                                            {} = {:?}{}",
                                            stringify!($expr), stringify!($a), *a, rest);
                                };
                            }
                        };
                    )
                }
                2 => {
                    let a = &args[0]; let b = &args[1];
                    quote_expr!(cx,
                        match (&($a), &($b)) {
                            (a, b) => {
                                if !($fun(*a, *b)) {
                                    panic!("assertion failed: {}:\n\
                                            {} = {:?}\n\
                                            {} = {:?}{}",
                                            stringify!($expr), stringify!($a), *a, stringify!($b),
                                            *b, rest);
                                };
                            }
                        };
                    )
                }
                3 => {
                    let a = &args[0]; let b = &args[1]; let c = &args[2];
                    quote_expr!(cx,
                        match (&($a), &($b), &($c)) {
                            (a, b, c) => {
                                if !($fun(*a, *b, *c)) {
                                    panic!("assertion failed: {}:\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}{}",
                                            stringify!($expr), stringify!($a), *a, stringify!($b),
                                            *b, stringify!($c), *c, rest);
                                };
                            }
                        };
                    )
                }
                4 => {
                    let a = &args[0]; let b = &args[1]; let c = &args[2]; let d = &args[3];
                    quote_expr!(cx,
                        match (&($a), &($b), &($c), &($d)) {
                            (a, b, c, d) => {
                                if !($fun(*a, *b, *c, *d)) {
                                    panic!("assertion failed: {}:\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}{}",
                                            stringify!($expr), stringify!($a), *a, stringify!($b),
                                            *b, stringify!($c), *c, stringify!($d), *d, rest);
                                };
                            }
                        };
                    )
                }
                5 => {
                    let a = &args[0]; let b = &args[1]; let c = &args[2]; let d = &args[3]; let e = &args[4];
                    quote_expr!(cx,
                        match (&($a), &($b), &($c), &($d), &($e)) {
                            (a, b, c, d, e) => {
                                if !($fun(*a, *b, *c, *d, *e)) {
                                    panic!("assertion failed: {}:\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}{}",
                                            stringify!($expr), stringify!($a), *a, stringify!($b),
                                            *b, stringify!($c), *c, stringify!($d), *d,
                                            stringify!($e), *e, rest);
                                };
                            }
                        };
                    )
                }
                6 => {
                    let a = &args[0]; let b = &args[1]; let c = &args[2]; let d = &args[3]; let e = &args[4]; let f = &args[5];
                    quote_expr!(cx,
                        match (&($a), &($b), &($c), &($d), &($e), &($f)) {
                            (a, b, c, d, e, f) => {
                                if !($fun(*a, *b, *c, *d, *e, *f)) {
                                    panic!("assertion failed: {}:\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}\n\
                                            {} = {:?}{}",
                                            stringify!($expr),
                                            stringify!($a), *a, stringify!($b), *b, stringify!($c),
                                            *c, stringify!($d), *d, stringify!($e), *e,
                                            stringify!($f), *f, rest);
                                };
                            }
                        };
                    )
                }
                _ => { unreachable!() }
            }
        },
        _ => {
            quote_expr!(cx,
                if !($expr) {
                    panic!(concat!("assertion failed: ", stringify!($expr), "{}"), rest);
                }
            )
        }
    };
    // Wrap the actual check into a function capable of being passed to `format_args!` and make
    // sure that the variable `rest` is bound.
    quote_expr!(cx, |: rest: &::std::fmt::Arguments| { $inner_check } )
}

/// Defines all the binary operations we might do, so we can handle them all.
fn get_check_func(cx: &mut ExtCtxt, cmp: ast::BinOp) -> P<ast::Expr> {
    match cmp.node {
        ast::BiEq  => {
            quote_expr!(cx, { fn chk<T: Eq>(x: &T, y: &T) -> bool { *x == *y }; chk })
        },
        ast::BiNe  => {
            quote_expr!(cx, { fn chk<T: Eq>(x: &T, y: &T) -> bool { *x != *y }; chk })
        },
        ast::BiLe  => {
            quote_expr!(cx, { fn chk<T: Ord>(x: &T, y: &T) -> bool { *x <= *y }; chk })
        },
        ast::BiGe  => {
            quote_expr!(cx, { fn chk<T: Ord>(x: &T, y: &T) -> bool { *x >= *y }; chk })
        },
        ast::BiLt  => {
            quote_expr!(cx, { fn chk<T: Ord>(x: &T, y: &T) -> bool { *x <  *y }; chk })
        },
        ast::BiGt  => {
            quote_expr!(cx, { fn chk<T: Ord>(x: &T, y: &T) -> bool { *x >  *y }; chk })
        },
        ast::BiOr  => {
            quote_expr!(cx, { fn chk(x: &bool, y: &bool)   -> bool { *x || *y }; chk })
        },
        ast::BiAnd => {
            quote_expr!(cx, { fn chk(x: &bool, y: &bool)   -> bool { *x && *y }; chk })
        },
        ast::BiAdd => {
            quote_expr!(cx, { fn chk<R, L: Add<R, bool>>(x: &L, y: &R) -> bool { *x + *y }; chk })
        },
        ast::BiSub => {
            quote_expr!(cx, { fn chk<R, L: Sub<R, bool>>(x: &L, y: &R) -> bool { *x - *y }; chk })
        },
        ast::BiMul => {
            quote_expr!(cx, { fn chk<R, L: Mul<R, bool>>(x: &L, y: &R) -> bool { *x * *y }; chk })
        },
        ast::BiDiv => {
            quote_expr!(cx, { fn chk<R, L: Div<R, bool>>(x: &L, y: &R) -> bool { *x / *y }; chk })
        },
        ast::BiRem => {
            quote_expr!(cx, { fn chk<R, L: Rem<R, bool>>(x: &L, y: &R) -> bool { *x % *y }; chk })
        },
        ast::BiShr => {
            quote_expr!(cx, { fn chk<R, L: Shr<R, bool>>(x: &L, y: &R) -> bool { *x >> *y }; chk })
        },
        ast::BiShl => {
            quote_expr!(cx, { fn chk<R, L: Shl<R, bool>>(x: &L, y: &R) -> bool { *x << *y }; chk })
        },
        ast::BiBitOr  => {
            quote_expr!(cx, { fn chk<R, L: BitOr<R, bool>>(x: &L, y: &R) -> bool { *x | *y }; chk })
        },
        ast::BiBitAnd => {
            quote_expr!(cx, { fn chk<R, L: BitAnd<R, bool>>(x: &L, y: &R)-> bool { *x & *y }; chk })
        },
        ast::BiBitXor => {
            quote_expr!(cx, { fn chk<R, L: BitXor<R, bool>>(x: &L, y: &R)-> bool { *x ^ *y }; chk })
        },
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) { reg.register_macro("internal_bassert", expand_bassert); }
