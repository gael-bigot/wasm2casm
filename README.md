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

int foo(int a, int b, int c){
    int d = 3 * add(b, c);
    return mul(a, d) - 12;
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
fn foo(a: i32, b: i32, c: i32) -> i32 {
    let d = 3 * add(b, c);
    mul(a, d) - 12
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
  Function 2: foo (type: 1, params: 3)

=== Exported Functions ===
  foo: 2
  mul: 1
  add: 0

=== Generated CASM Instructions (21) ===
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
  [10]: [fp + 0] = [fp + -4]
  [11]: [fp + 1] = [fp + -6]
  [12]: [fp + 2] = [fp + -5]
  [13]: [fp + 1] = [fp + 1] + [fp + 2]
  [14]: [fp + 0] = [fp + 0] * [fp + 1]
  [15]: [fp + 1] = 3
  [16]: [fp + 0] = [fp + 0] * [fp + 1]
  [17]: [fp + 1] = -12
  [18]: [fp + 0] = [fp + 0] + [fp + 1]
  [19]: [fp + -3] = [fp + 0]
  [20]: ret

=== Function Labels ===
  mul -> instruction [5]
  foo -> instruction [10]
  add -> instruction [0]
  ```
