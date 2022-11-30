# Building

1. Install stable toolchain from [rust-lang.org](https://www.rust-lang.org/tools/install)

2. Build
```
$ cargo build --release -p client # for client
$ cargo build --release -p server # for server
```

The output binaries are `target/release/neo` for client and `target/release/neogrokd` for server.
