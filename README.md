# Demo library for my SambaXP 2018 talk

## Parts

The repository is split into multiple parts:

### Protocol

The basic protocol implementation in Rust

### Client

A command-line client in Rust

### Rust-Server

A pure Rust implementation of the server, as proof of concept


### C-Server

A C-based UDP server handing off the parsing to the Rust library, managing allocated memory in Rust.


## Talk

Slides of the talk I gave at SambaXP 2018 can be found at https://kblin.org/talks/sambaxp/2018/rust_in_samba.html#1


## License

Like Samba, this code is under a GNU GPL v3+ license,
see [`LICENSE`](LICENSE) for details.
