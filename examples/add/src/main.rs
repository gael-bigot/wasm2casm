#![no_std]
#![no_main]

extern crate panic_abort;

#[unsafe(no_mangle)]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[unsafe(no_mangle)]
fn mul(a: i32, b: i32) -> i32 {
    a * b
}

#[unsafe(no_mangle)]
fn foo(a: i32, b: i32, c: i32) -> i32 {
    let d = 3 * add(b, c);
    mul(a, d) - 12
}
