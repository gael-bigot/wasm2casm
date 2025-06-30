#Wasm2Casm

Transpiling your projects written in Rust, C and more into Cairo-M assembly.
Check out the [Cairo-M project](https://github.com/kkrt-labs/cairo-m).

##  1. Create and compile your project

### In C

Write some non-libc code :

```C
int add(int x, int y){
    return x + y;
}


int mul(int x, int y){
    return x * y;
}

int main(int x, int y, int z){
    return mul(x, add(y, z));
}
```

Compile with
```
clang -c -nostdlib --target=wasm32 -o examples/add.wasm examples/add.c -O1
```

### In Rust

Write some no_std Rust

```Rust
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
fn main(a: i32, b: i32, c: i32) -> i32 {
    mul(a, add(b, c))
}

```

Compile with
```
cd examples/add
cargo build -r --target=wasm32-unknown-unknown
```

## 2. Convert to CASM

```
cargo run -- examples/add.wasm
```
or
```
cargo run -- examples/add/target/wasm32-unknown-unknown/release/add.wasm
```
