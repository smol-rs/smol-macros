//! Use the `macro_rules_attribute` to use `main` as an attribute.

use macro_rules_attribute::apply;
use smol_macros::main;

#[apply(main!)]
async fn main() {
    println!("hello world!");
}
