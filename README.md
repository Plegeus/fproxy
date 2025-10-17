# fproxy #
## Introduction ##
Rust has an unstable ABI which makes rust to rust ffi unsafe (across dll boundaries). This crate aims to solve that issue by creating ffi-safe wrappers around existing types, so called proxies.

## The Goal ##
The goal of this crate is the allow safe rust to rust ffi for (ideally) the entirety of the rust language in the context of dynamically loaded libraries. The most common usecase are **plugin systems**, where some applications needs to load plugins at runtime.
An additional goal is **to minimise or even negate boilerplate** on the plugin side, that is, existing libraries should be able to become ffi-safe ("abi stable") **without altering existing code**.

To this end fproxy provides a set of macros that can be used to annotate the types that are required across a dll boundary.

## How it works ##
### Proxies ###
The crate generates an additional type (the proxy) for each type tagged with `#[fproxy::proxy]`.

The following struct
```rust
struct MyType {
  /* snip */
}
```
translates to:
```rust
struct FMyType<'l> {
  handle: *const (),
  lib: &'l Library,
}
```
regardless of its fields.

### Implementations ###
For each `impl` fproxy will generate an `extern "C" fn` and an `impl` on the generated proxy. The `impl` on the proxy merely propagates data to the `extern "C"` function, performing necesary conversions to `#[repr(C)]` types if needed. Finally, the C function will convert the C types back to the original rust types. 
```rust
impl MyType
  fn do_something(&self) {
    /* snip */
  }
}
```
translates to:
```rust
impl FMyType<'_> {
  fn do_something(&self) {
     use fproxy::libloading::{Symbol};
     let func: Symbol<unsafe extern "C" fn(*const ()) -> ()> = 
       self.lib.get(b"_fproxy_MyType_do_something\0").unwrap();
     func(self.handle,)
  }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _fproxy_MyType_do_something(handle: *const ()) {
  MyType::do_something(&*(handle as *const MyPugin))
}
```

This introduce some minor overhead, the C function cannot be inlined and for non `#[repr(C)]` types, conversions need to be performed in order to safely pass the dll boundary.
For proxies this is not a big problem, as it will simply pass its pointer. Since all types are known at compile time, this part of the crate is fast.

### Conversion ###
The behaviour shown above can be customised for parameters, to that end the traits `FIntoProxy`, `FToC`, `FFromC` and `FReprC` can be implemented. Please consider the following toy example: 
```rust
/// Rust u128 is not guaranteed to be `#[repr(C]`, so a conversion type is needed.
#[derive_ReprC]
#[repr(C)]
pub struct U128 {
  l: u64,
  r: u64,
}

/// The `From` trait is implemented in both ways to convert the Rust type to a C type
/// and visa versa.
impl From<u128> for U128 {
  fn from(value: u128) -> Self {
    let r = value & 0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF;
    U128 { 
      l: ((value - r) >> 64) as u64, 
      r: r as u64,
    }
  }
}
impl From<U128> for u128 {
  fn from(value: U128) -> Self {
    ((value.l as u128) << 64) + value.r as u128
  }
}

/// The CType is used to "cast" types to their C variants.
impl FToC for u128 {
  type CType = U128;
  fn to_c(self) -> Self::CType {
    From::from(self)
  }
}

// FFromC is automatically implemented for types T that impl FToC + From<T::CType>.
```

These traits allow for full customisablity, notably not all types need to be a proxy.

## Examples ##
For a detailed set of examples, please refer to the `./fproxy_examples` subdirectory.

## Roadmap ##
Primary goals are:
* traits (both user defined and std),

Possible problems are:
* generics,
* `impl SomeTrait` in parameters.


