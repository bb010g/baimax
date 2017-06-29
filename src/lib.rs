#![feature(test)]
#![feature(try_from)]
#![cfg_attr(feature="lint", feature(plugin))]
#![cfg_attr(feature="lint", plugin(clippy))]

extern crate chrono;
extern crate itertools;
#[macro_use]
extern crate nom;
extern crate penny;
#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate test;
extern crate void;

use itertools::Itertools;

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

pub mod ast;
pub mod data;
pub mod parse;

use ast::convert::ConverterOutput;
use ast::parse::Parsed;

#[derive(Debug, Clone)]
pub enum ProcessFileError<'a> {
    Parse(nom::ErrorKind),
    FieldParse(ast::parse::ParseError<ast::Record<'a>>),
    UnfinishedConversion,
    Conversion(ast::convert::ConvertError),
}

pub fn process_file<'a>(
    file: &'a [u8],
    end_of_day: &chrono::NaiveTime,
) -> Result<data::File, ProcessFileError<'a>> {
    match parse::file(file).to_result().map(|raw_records| {
        let parsed_records = raw_records.iter().map(|r| ast::Record::parse(r));
        let mut converter = ast::convert::Converter::new(&end_of_day);
        parsed_records.into_iter().fold_results(
            ConverterOutput::Active,
            |acc, r| match converter.process(r) {
                ConverterOutput::Done => acc,
                o => o,
            },
        )
    }) {
        Ok(Ok(ConverterOutput::Done)) => unreachable!(),
        Ok(Ok(ConverterOutput::Err(e))) => Err(ProcessFileError::Conversion(e)),
        Ok(Ok(ConverterOutput::Ok(file))) => Ok(file),
        Ok(Ok(ConverterOutput::Active)) => Err(ProcessFileError::UnfinishedConversion),
        Ok(Err(e)) => Err(ProcessFileError::FieldParse(e)),
        Err(e) => Err(ProcessFileError::Parse(e)),
    }
}

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
                let bytes = bytes.as_slice();
                let end_of_day = $end_of_day;

                b.iter(|| {
                    let result = process_file(bytes, &end_of_day);
                    result.unwrap()
                })
            }

            #[bench]
            fn $parse(b: &mut Bencher) {
                let bytes = $file_name.bytes().collect::<Vec<_>>();

                b.iter(|| {
                    let result = parse::file(bytes.as_slice()).to_result();
                    result.unwrap()
                })
            }

            #[bench]
            fn $ast_parse(b: &mut Bencher) {
                let bytes = $file_name.bytes().collect::<Vec<_>>();

                let raw = parse::file(bytes.as_slice()).to_result();
                let raw = raw.unwrap();
                use ast::parse::Parsed;
                b.iter(|| {
                    raw.iter().map(|r| ast::Record::parse(r)).count()
                })
            }

            #[bench]
            fn $convert(b: &mut Bencher) {
                let bytes = $file_name.bytes().collect::<Vec<_>>();
                let end_of_day = $end_of_day;

                let raw: Result<_, _> = parse::file(bytes.as_slice()).to_result();
                use ast::parse::Parsed;
                let parsed =
                    raw.map(|r| r.iter().map(|r| ast::Record::parse(r)) .collect::<Vec<_>>());
                let parsed = parsed.unwrap();
                b.iter(|| {
                    let parsed = parsed.to_vec();
                    let mut converter = ast::convert::Converter::new(&end_of_day);
                    let result = parsed.into_iter().fold_results(None, |acc, r| {
                        converter.process(r).unwrap().or(acc)
                    });
                    result.unwrap().unwrap()
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
