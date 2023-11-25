//! Example of using the `main` macro.

use smol_macros::main;

main! {
    async fn main() {
        println!("hello world!");
    }
}
