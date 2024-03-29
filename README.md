# rust-mqtt
An implementation of the [MQTT protocol](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html) in rust, 
including a basic MQTT client on top of the lib.

> This is still work in progress!

**Currently only supports version 5.**

This project is primarily a learning tool and the basis for a series of exercises for the language itself. As such,
external crates are used only where it is either a must ([macros](#Macros)), or where it would be overly burdensome 
not to ([CLI](#Running).

Especially the lib part purposefully ignores the usual set of obvious helpers like `thiserror`, maybe `nom` or `serde`.

There are also obviously no efforts to go `no_std` or take embedded requirements into consideration. For that, there's 
way better options out there.

## Contents
- `mqtt`: the basic protocol implementation, data types, packets, encoding and decoding
- `mqtt-derive`: (internal) macros to the lib. Mostly a teaching vehicle for derive macros. [See below](#Macros) for details.
- `mqtt-cli`: simple command-line mqtt client. [See below](#Running).

## Running

The command-line interface (CLI) is a very simple MQTT client, capable of publishing or listening to messages.

Using the client, for example: `cargo run pub -h test.mosquitto.org -t /some/topic -m "hello world"`.
For more options and features run `cargo run help`.

The CLI app is built using [`clap`](https://github.com/clap-rs/clap)) to generate the commands and options.

## Macros
A custom `derive` macro has been added to help with the repetitive nature of encoding and decoding 
[`Properties`](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901027), which
may appear in most of the MQTT control packet's `Variable Header` (except for `PING...` packets), as well as the 
`Last Will`. 
This macro is internal to the library and cannot be used outside of it, as it generates code only accessible 
internally. Rust forces us to build this is an entirely separate crate, though.
For obvious reasons we deviate from the "no external crates" rule. Even if writing a `derive` macro without 
[`syn`](https://github.com/dtolnay/syn)) or [`quote`](https://github.com/dtolnay/quote) is possible, we couldn't find
our soldering iron.