#![cfg_attr(cfg="test", feature(test))]
#![feature(try_from)]
#![cfg_attr(feature="lint", feature(plugin))]
#![cfg_attr(feature="lint", plugin(clippy))]

extern crate chrono;
#[macro_use]
extern crate nom;
extern crate penny;
#[cfg(featuer = "serde")]
extern crate serde;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate test;
extern crate void;

macro_rules! enum_mapping {
    ($(#[$attr:meta])* pub $name:ident($ty:ty) {
        $($key:ident($val:expr)),+;
        $custom:ident {
            from: $from_pat:pat => $from_expr:expr;
            to: $to_pat:pat => $to_expr:expr;
        }
    }) => {
        $(#[$attr])*
        pub enum $name {
            $($key,)+
            $custom($ty)
        }
        impl ::std::convert::TryFrom<$ty> for $name {
            type Error = $ty;
            fn try_from(x: $ty) -> Result<$name, $ty> {
                match x {
                    $($val => Ok($name::$key),)+
                    $from_pat => $from_expr,
                    _ => Err(x),
                }
            }
        }
        impl From<$name> for $ty {
            fn from(x: $name) -> $ty {
                match x {
                    $($name::$key => $val,)+
                    $to_pat => $to_expr,
                }
            }
        }
    };
    ($(#[$attr:meta])* pub $name:ident($ty:ty) { $($key:ident($val:expr),)+ }) => {
        $(#[$attr])*
        pub enum $name {
            $($key,)+
        }
        impl ::std::convert::TryFrom<$ty> for $name {
            type Error = $ty;
            fn try_from(x: $ty) -> Result<$name, $ty> {
                match x {
                    $($val => Ok($name::$key),)+
                    _ => Err(x),
                }
            }
        }
        impl From<$name> for $ty {
            fn from(x: $name) -> $ty {
                match x {
                    $($name::$key => $val,)+
                }
            }
        }
    };
}

pub mod data;
pub mod ast;
pub mod parse;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    macro_rules! benchmark_file {
        ($file_name:ident, $file_path:expr,
         $process:ident, $parse:ident, $ast_parse:ident, $convert:ident,
         $end_of_day:expr) => {
            static $file_name: &'static str = include_str!($file_path);

            #[bench]
            fn $process(b: &mut Bencher) {
                let bytes = $file_name.bytes().collect::<Vec<_>>();
                let end_of_day = $end_of_day;

                b.iter(|| {
                    let raw = parse::file(bytes.as_slice()).to_result().unwrap();
                    use ast::parse::Parsed;
                    let mut parsed = raw.iter().map(|r| ast::Record::parse(r).unwrap());
                    ast::convert::convert(&mut parsed, &end_of_day).unwrap()
                })
            }

            #[bench]
            fn $parse(b: &mut Bencher) {
                let bytes = $file_name.bytes().collect::<Vec<_>>();

                b.iter(|| {
                    parse::file(bytes.as_slice()).to_result().unwrap()
                })
            }

            #[bench]
            fn $ast_parse(b: &mut Bencher) {
                let bytes = $file_name.bytes().collect::<Vec<_>>();

                let raw = parse::file(bytes.as_slice()).to_result().unwrap();
                use ast::parse::Parsed;
                b.iter(|| {
                    raw.iter().map(|r| ast::Record::parse(r).unwrap()).count()
                })
            }

            #[bench]
            fn $convert(b: &mut Bencher) {
                let bytes = $file_name.bytes().collect::<Vec<_>>();
                let end_of_day = $end_of_day;

                let raw = parse::file(bytes.as_slice()).to_result().unwrap();
                use ast::parse::Parsed;
                let parsed = raw.iter().map(|r| ast::Record::parse(r).unwrap()).collect::<Vec<_>>();
                b.iter(|| {
                    let parsed = parsed.to_vec();
                    ast::convert::convert(&mut parsed.into_iter(), &end_of_day).unwrap()
                })
            }
        };
    }

    benchmark_file!(
        SPEC_EXAMPLE,
        "../spec-example.bai",
        process_spec_example,
        parse_spec_example,
        ast_parse_spec_example,
        convert_spec_example,
        chrono::NaiveTime::from_hms(17, 23, 00)
    );
}
