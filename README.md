# <img align="right" src="baymax.png" alt="Baymax" title="On a scale of one to ten, how would you rate your pain?"> baimax [![Build Status][img-buildstatus]][buildstatus] [![Cargo][img-cargo]][cargo]

[API documentation][api-docs] | [Changelog][changelog]

Baimax is a type-safe BAIv2 deserialization library for Rust. It is currently in
an alpha state, and is probably not going to be restructured majorly soon.

Baimax requires nightly Rust due to the [`try_from`][try-from] feature
([tracking issue][try-from-issue]).

## Compliance

The physical record length header isn't checked while parsing.

Record number checks in trailers aren't supported because I haven't figured out
an efficient encoding for them in the AST yet. You'd think some bitflags would
do, but any 88 Continuation record can be followed by another continuation,
turning your nice set of bits to cover everything into a `u8` tagging the code
and every field of every record type, which is less nice.

Pull requests are welcome to increase spec compliance.


[img-buildstatus]: https://img.shields.io/travis/bb010g/baimax.svg
[buildstatus]: http://travis-ci.org/bb010g/baimax
[img-cargo]: https://img.shields.io/crates/v/baimax.svg
[cargo]: https://crates.io/crates/baimax

[api-docs]: https://docs.rs/baimax/0.1.0/baimax
[changelog]: https://github.com/bb010g/baimax/blob/master/CHANGELOG.md

[try-from]: https://doc.rust-lang.org/nightly/std/convert/trait.TryFrom.html
[try-from-issue]: https://github.com/rust-lang/rust/issues/33417
