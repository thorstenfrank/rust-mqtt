# rust-mqtt
An implementation of the [MQTT protocol](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html) in rust.

This project is solely a basis for developing a series of training exercises for learning the language itself, so 
there are no external dependencies (yet) that make life easier. Or that would seem sane to use.
Especially the encoding/decoding parts are hand-built on purpose, and *not* based on `nom` or even `serde`. Maybe later.

There are also obviously no efforts to go `no_std` or take embedded requirements into consideration. If you find
yourself in need of an MQTT implementation for embedded scenarios, there's probably options out there.

# Macros
A custom `derive` macro has been added to help with the repetitive nature of encoding and decoding `Properties`, which
may appear in any MQTT control packet's `Variable Header`, except for `PING...`, as well as the `Last Will`. 
This macro is internal to the library and cannot be used outside of it, as it generates code only accessible 
internally. Rust forces us to build this is an entirely separate crate, though.
For obvious reasons we deviate from the "no external crates" rule. Even if writing a `derive` macro without `syn` or `quote` is possible, the question is: why?