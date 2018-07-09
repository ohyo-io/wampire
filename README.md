# wampire
Rust implementation of a WAMP [Web Application Messaging Protcol](http://wamp-proto.org/). client and router

At present the entire Basic Profile is supported, as well as pattern based subscriptions and registrations from the Advanced Profile.

There is currently no support for secure connections.

For instructions on how to use, please see the [examples](examples) directory.

To include in your project, place the following in your `Cargo.toml`

```toml
[dependencies]
wampire = "0.1"
```

wampire uses [serde-rs](https://github.com/serde-rs/serde), which requires Rust 1.15 or greater.

Initial forked from https://github.com/dyule/wamp-rs

## router

```bash
RUST_LOG=info cargo run --example router
```