#![feature(try_from)]
#![feature(plugin)]
#![plugin(clippy)]

extern crate chrono;
#[macro_use]
extern crate if_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nom;
extern crate penny;
#[macro_use]
extern crate regex;
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
            type Err = $ty;
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
            type Err = $ty;
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
pub mod type_codes;
pub mod ast;
pub mod parse;

#[cfg(test)]
mod tests {
    mod parser {
        use ::parse;

        #[test]
        fn record() {}
    }
}
