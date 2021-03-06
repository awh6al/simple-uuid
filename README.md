## UUID ![](https://github.com/awh6al/simple-uuid/workflows/simple-uuid/badge.svg)
A universally unique identifier (UUID) is a 128-bit number used to identify
information in computer systems. The term globally unique identifier (GUID)
is also used.

This crate generates and inspects UUIDs based on [RFC 4122](http://tools.ietf.org/html/rfc4122).

## Install
```TOML
[dependencies]
simple-uuid = { version = "*", features = ["rand_num"] }
```

## Usage
```Rust
use simple_uuid::v4;
println!("{}", v4!())
```

## Security

Do not assume that UUIDs are hard to guess; they should not be used as security capabilities.
