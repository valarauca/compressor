feature_macros
---

This crate allows for controling how safe/unsafe other crates
within the overarching project are.

As these crates are built to emulate C code in performance
some consumers may wish to ensure "closer" compatibility with
the default unsafe C execution model. Therefore these options
are given.

## Developer Controllable Options

* `ub_unreachable`: This controls the semantics of the `inconceivable!` macro. When this options is not supplied (or when this options is supplied, and the crate is compiled with `rustc --version < 1.27`) `inconceivable!` will simply alias `unreachable!`. When this option is supplied (and the crate is compiled with `rustc --version >= 1.27`) this will instead emit `unreachable_uncheck()` which is UB.
* `unbounded`: This controls the semantics of the internal `Number<T>` trait. When provided, bounds checks will **not** be performed. 
* `branch_hints`: The internal functions `hint_likely` and `hint_unlikely` are always exported, but whe this is used (on `nightly` channel) they will emit the LLVM's branch hinting intrinsics.
* `prefetch_hints`: The internal function `prefetch` will emit either the platform correct prefetch hint, or nothing. When not enabled this function is a `nop`.

## Developer Uncontrollable Options

These options are set by the internal `build.rs` script. It is best to not worry about them.

* `RUSTC_DEV`: States if this crate is being compiled by a `developer` channel compiler.
* `RUSTC_NIGHTLY`: States if this crate is being compiled by a `nightly` channel compiler.
* `RUSTC_STABLE`: States if this crate is being compiled by a `stable` channel compiler.
* `RUSTC_VERSION_GE_1_27`: States if `rustc --version >= 1.27` this is used as a feature check.
* `RUSTC_VERSION_GE_1_26`: States if `rustc --version >= 1.26` this is used as a feature check.

