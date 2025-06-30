# Wasm2Casm

Transpiling your projects written in Rust, C and more into Cairo-M assembly.
Check out the [Cairo-M project](https://github.com/kkrt-labs/cairo-m).

## 1. Create and compile your project

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

## Expected result

```
=== WASM to CASM Transpilation ===

=== Function Types (2) ===
  Type 0: (I32, I32) -> (I32)
  Type 1: (I32, I32, I32) -> (I32)

=== Functions (3) ===
  Function 0: add (type: 0, params: 2)
  Function 1: mul (type: 0, params: 2)
  Function 2: main (type: 1, params: 3)

=== Exported Functions ===
  add: 0
  mul: 1
  main: 2

=== Generated CASM Instructions (17) ===
  [0]: [fp + 0] = [fp + -5]
  [1]: [fp + 1] = [fp + -4]
  [2]: [fp + 0] = [fp + 0] + [fp + 1]
  [3]: [fp + -3] = [fp + 0]
  [4]: ret
  [5]: [fp + 0] = [fp + -5]
  [6]: [fp + 1] = [fp + -4]
  [7]: [fp + 0] = [fp + 0] * [fp + 1]
  [8]: [fp + -3] = [fp + 0]
  [9]: ret
  [10]: [fp + 0] = [fp + -6]
  [11]: [fp + 1] = [fp + -5]
  [12]: [fp + 0] = [fp + 0] + [fp + 1]
  [13]: [fp + 1] = [fp + -4]
  [14]: [fp + 0] = [fp + 0] * [fp + 1]
  [15]: [fp + -3] = [fp + 0]
  [16]: ret

=== Function Labels ===
  main -> instruction [10]
  add -> instruction [0]
  mul -> instruction [5]
  ```
