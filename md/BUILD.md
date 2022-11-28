# Building NeoGrok

WARNING: Wine is not supported due to https://github.com/tokio-rs/mio/issues/1444 (broken by mio's tokio backend because mio internally uses non-documented windows-specific APIs to perform O(1) polling)

# Installing Rust compiler

Follow the [official instructions](https://www.rust-lang.org/tools/install)

# MSRV

Currently NeoGrok builds under `rustc 1.66` because internally it uses let else statements, for example.

NeoGrok is written using only stable features, so you don not need to install nightly toolchain.

# Building

## Client

```
$ cargo build --release -p neogrok
```

Output binary: `target/release/neo`. For CLI help type 
```
$ neo --help
```

## Server

```
$ cargo build --release -p neogrokd
```

Output binary: `target/release/neogrokd`.
