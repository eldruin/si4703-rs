# Rust Si470x FM Radio Turner (Receiver) Driver

<!--TODO
[![crates.io](https://img.shields.io/crates/v/si470x.svg)](https://crates.io/crates/si470x)
[![Docs](https://docs.rs/si470x/badge.svg)](https://docs.rs/si470x)
-->
[![Build Status](https://travis-ci.org/eldruin/si470x-rs.svg?branch=master)](https://travis-ci.org/eldruin/si470x-rs)
[![Coverage Status](https://coveralls.io/repos/github/eldruin/si470x-rs/badge.svg?branch=master)](https://coveralls.io/github/eldruin/si470x-rs?branch=master)

This is a platform agnostic Rust driver for the Si470x FM radio turner
(receiver) family using the [`embedded-hal`] traits.
<!-- TODO
This driver allows you to:
-->
<!--TODO
[Introductory blog post](https://blog.eldruin.com/...)
-->

## The devices

The Si4702/03-C19 extends Silicon Laboratories Si4700/Si4701 FM tuner
family, and further increases the ease and attractiveness of adding FM
radio reception to mobile devices through small size and board area,
minimum component count, flexible programmability, and superior, proven
performance.

The device offers significant programmability, and caters to the
subjective nature of FM listeners and variable FM broadcast environments
world-wide through a simplified programming interface and
mature functionality.

The Si4703-C incorporates a digital processor for the European Radio Data
System (RDS) and the US Radio Broadcast Data System (RBDS) including all
required symbol decoding, block synchronization, error detection, and
error correction functions.

RDS enables data such as station identification and song name to be
displayed to the user. The Si4703-C offers a detailed RDS view and a
standard view, allowing adopters to selectively choose granularity of RDS
status, data, and block errors.

Datasheets:
- [Si4702/Si4703](https://www.silabs.com/documents/public/data-sheets/Si4702-03-C19.pdf)

Further documentation:
- [Si4700/01/02/03 Programmer's Guide](https://www.silabs.com/documents/public/application-notes/AN230.pdf)
- [Using RDS/RBDS with the Si4701/03](https://www.silabs.com/documents/public/application-notes/AN243.pdf)
- [Si47xx Programming Guide](https://www.silabs.com/documents/public/application-notes/AN332.pdf)

## Usage

To use this driver, import this crate and an `embedded_hal` implementation,
then instantiate the appropriate device.

<!--TODO
In the following example an instance of the device Si4703 will be created.
Other devices can be created with similar methods like:
`Si470x::new_si4702(...)`.
-->
Please find additional examples using hardware in this repository: [driver-examples]

[driver-examples]: https://github.com/eldruin/driver-examples

<!-- TODO
```rust
```
-->

## Support

For questions, issues, feature requests, and other changes, please file an
[issue in the github project](https://github.com/eldruin/si470x-rs/issues).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
