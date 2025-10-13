# fproxy
Rust has an unstable ABI which makes rust to rust ffi unsafe (across dll boundaries). This crate aims to solve that issue by creating ffi-safe wrappers around existing types, so called proxies.
