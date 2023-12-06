# smol-macros

[![Build](https://github.com/smol-rs/smol-macros/actions/workflows/ci.yml/badge.svg)](
https://github.com/smol-rs/smol-macros/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](
https://github.com/smol-rs/smol-macros)
[![Cargo](https://img.shields.io/crates/v/smol-macros.svg)](
https://crates.io/crates/smol-macros)
[![Documentation](https://docs.rs/smol-macros/badge.svg)](
https://docs.rs/smol-macros)

Macros for using `smol-rs`.

One of the advantages of [`smol`] is that it lets you set up your own executor, optimized for
your own use cases. However, quick scaffolding is important for many organizational use cases.
Especially when sane defaults are appreciated, setting up your own executor is a waste of
time.

This crate provides macros for setting up an efficient [`smol`] runtime quickly and
effectively. It provides sane defaults that are useful for most applications.

## Simple Executor

Just have an `async` main function, using the [`main`] macro.

```rust
use smol_macros::main;

main! {
    async fn main() {
        println!("Hello, world!");
    }
}
```

This crate uses declarative macros rather than procedural macros, in order to avoid needing
to use heavy macro dependencies. If you want to use the proc macro syntax, you can use the
[`macro_rules_attribute::apply`] function to emulate it.

The following is equivalent to the previous example.

```rust
use macro_rules_attribute::apply;
use smol_macros::main;

#[apply(main!)]
async fn main() {
    println!("Hello, world!");
}
```

## Task-Based Executor

This crate re-exports [`smol::Executor`]. If that is used as the first parameter in a
function in [`main`], it will automatically create the executor.

```rust
use macro_rules_attribute::apply;
use smol_macros::{main, Executor};

#[apply(main!)]
async fn main(ex: &Executor<'_>) {
    ex.spawn(async { println!("Hello world!"); }).await;
}
```

If the thread-safe [`smol::Executor`] is used here, a thread pool will be spawned to run
the executor on multiple threads. For the thread-unsafe [`smol::LocalExecutor`], no threads
will be spawned.

See documentation for the [`main`] function for more details.

## Tests

Use the [`test`] macro to set up test cases that run self-contained executors.

```rust
use macro_rules_attribute::apply;
use smol_macros::{test, Executor};

#[apply(test!)]
async fn do_test(ex: &Executor<'_>) {
    ex.spawn(async {
        assert_eq!(1 + 1, 2);
    }).await;
}
```

[`smol`]: https://crates.io/crates/smol
[`smol::Executor`]: https://docs.rs/smol/latest/smol/struct.Executor.html
[`smol::LocalExecutor`]: https://docs.rs/smol/latest/smol/struct.LocalExecutor.html
[`macro_rules_attribute::apply`]: https://docs.rs/macro_rules_attribute/latest/macro_rules_attribute/attr.apply.html

## MSRV Policy

The Minimum Supported Rust Version (MSRV) of this crate is **1.63**. As a **tentative** policy, the MSRV will not advance past the [current Rust version provided by Debian Stable](https://packages.debian.org/stable/rust/rustc). At the time of writing, this version of Rust is *1.63*. However, the MSRV may be advanced further in the event of a major ecosystem shift or a security vulnerability.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
