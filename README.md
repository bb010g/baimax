# <img align="right" src="baymax.png" alt="Baymax" title="On a scale of one to ten, how would you rate your pain?"> baimax [![Build Status][img-buildstatus]][buildstatus] [![Cargo][img-cargo]][cargo]

[API documentation][api-docs] | [Changelog][changelog]

Baimax is a type-safe BAI deserialization library for Rust. It is currently in an
alpha state, and is probably not going to be restructured majorly soon. It
currently only works on variable-width records.

Baimax requires nightly Rust due to the [`try_from`][try-from] feature
([tracking issue][try-from-issue]).


[img-buildstatus]: https://img.shields.io/travis/bb010g/baimax.svg
[buildstatus]: http://travis-ci.org/bb010g/baimax
[img-cargo]: https://img.shields.io/crates/v/baimax.svg
[cargo]: https://crates.io/crates/baimax

[api-docs]: https://docs.rs/baimax/0.1.0/baimax
[changelog]: https://github.com/bb010g/baimax/blob/master/CHANGELOG.md

[try-from]: https://doc.rust-lang.org/nightly/std/convert/trait.TryFrom.html
[try-from-issue]: https://github.com/rust-lang/rust/issues/33417
