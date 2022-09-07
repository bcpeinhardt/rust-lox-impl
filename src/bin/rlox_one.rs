//! RLOX_One is the Rust version of the JLox impl from Part 2 of Crafting Interpreters (the original implementation). It is
//! a tree-walk interpreter and will be very object oriented (as we are porting Java). It tries to stay as true to the
//! Java implementation as possible, and only deviates when required by differences in the language. Therefore, it's not very
//! "rusty". WE will try much harder to rustify the port of the final C implementation.

use rust_lox_impl::lox::Lox;

fn main() {
    // We use a class here because the java impl uses static members of the class
    // main resides in for state. So we sort of needed a "Main Class".
    Lox::new().lox_main();
}
