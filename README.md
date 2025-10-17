# fproxy #
## Introduction ##
Rust has an unstable ABI which makes rust to rust ffi unsafe (across dll boundaries). This crate aims to solve that issue by creating ffi-safe wrappers around existing types, so called proxies.

## The Goal ##
The goal of this crate is the allow safe rust to rust ffi for (ideally) the entirety of the rust language in the context of dynamically loaded libraries. The most common usecase are *plugin systems*, where some applications needs to load plugins at runtime.
An additional goal is *to minimise or even negate boilerplate* on the plugin side, that is, existing libraries should be able to become ffi-safe ("abi stable") *without altering existing code*.

To this end fproxy provides a set of macros that can be used to annotate the types that are required across a dll boundary.

## How it works ##
The crate generates an additional type (the proxy) for each type tagged with `#[fproxy::proxy]`.
A proxy looks as follows:
```rust
  struct FMyType<'l> {
    handle: *const (),
    lib: &'l Library,
  }
```

For each `impl` fproxy will generate an `extern "C" fn` and an `impl` on the generated proxy. The `impl` on the proxy merely propagates data to the `extern "C"` function, performing necesary conversions to `#[repr(C)]` types if needed. Finally, the C function will convert the C types back to the original rust types. 

This introduce some minor overhead, the C function cannot be inlined and for non `#[repr(C)]` types, conversions need to be performed in order to safely pass the dll boundary.
For proxies this is not a big problem, as it will simply pass its pointer. Since all types are known at compile time, this part of the crate is fast.

## Examples ##
For a detailed set of examples, please refer to the `./fproxy_examples` subdirectory.

## Roadmap ##
Primary goals are:
* traits (both user defined and std),

Possible problems are:
* generics,
* `impl SomeTrait` in parameters.


