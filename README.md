# dlib, making native libraries optional

[![](http://meritbadge.herokuapp.com/dlib)](https://crates.io/crates/dlib)
[![Docs.rs](https://docs.rs/dlib/badge.svg)](https://docs.rs/dlib)

dlib is a small crate providing macros to make easy the use of external system libraries
that can or cannot be optionally loaded at runtime, depending on whether the `dlopen` cargo
feature is enabled.

## Usage

dlib defines the `external_library!` macro, which can be invoked with way:

```rust
external_library!(Foo, "foo",
    statics:
        me: c_int,
        you: c_float,
    functions:
        fn foo() -> c_int,
        fn bar(c_int, c_float) -> (),
        fn baz(*const c_int) -> c_int,
    varargs:
        fn blah(c_int, c_int ...) -> *const c_void,
        fn bleh(c_int ...) -> (),
);
```

As you can see, it is required to separate static values from functions and from function
having variadic arguments. Each of these 3 categories is optional, but the ones used must appear
in this order. Return types of the functions must all be explicit (hence `-> ()` for void functions).

If the feature `dlopen` is absent, this macro will expand to an extern block defining each of the
items, using the second argument of the macro as a link name:

```rust
#[link(name = "foo")]
extern "C" {
    pub static me: c_int;
    pub static you: c_float;
    pub fn foo() -> c_int;
    pub fn bar(_: c_int, _: c_float) -> ();
    pub fn baz(_: *const c_int) -> c_int;
    pub fn blah(_: c_int, _: c_int, ...) -> *const c_void;
    pub fn bleh(_: c_int, ...) -> ();
}

```

If the feature `dlopen` is present, it will expand to a `struct` named by the first argument of the macro,
with one field for each of the symbols defined, and a method `open`, which tries to load the library
from the name or path given as argument

```rust
pub struct Foo {
    pub me: &'static c_int,
    pub you: &'static c_float,
    pub foo: unsafe extern "C" fn() -> c_int,
    pub bar: unsafe extern "C" fn(c_int, c_float) -> (),
    pub baz: unsafe extern "C" fn(*const c_int) -> c_int,
    pub blah: unsafe extern "C" fn(c_int, c_int, ...) -> *const c_void,
    pub bleh: unsafe extern "C" fn(c_int, ...) -> (),
}


impl Foo {
    pub fn open(name: &str) -> Result<Foo, DlError> { /* ... */ }
}
```

This method returns `Ok(..)` if the loading was successful. It contains an instance of the defined struct
with all of its fields pointing to the appropriate symbol.

If the library specified by `name` could not be found, it returns `Err(DlError::NotFount)`.

It will also fail on the first missing symbol, with `Err(DlError::MissingSymbol(symb))` where `symb` is a `&str`
containing the missing symbol name.

## Remaining generic in your crate

If you want your crate to remain generic over the `dlopen` cargo feature, simply add this to your `Cargo.toml`

```
[dependencies]
dlib = "0.2"

[features]
dlopen = ["dlib/dlopen"]
```

And the library also provides helper macros to dispatch the access to foreign symbols:

```rust
ffi_dispatch!(Foo, function, arg1, arg2);
ffi_dispatch_static!(Foo, static);
```

These will expand to the appropriate value or function call depending on the presence of the `dlopen` feature.

You must still ensure that the functions/statics or the wrapper struct `Foo` are in scope. A simple pattern would be
for example to use the `lazy_static!` crate to do the initialization and store the wrapper struct in a static, that you then
just need to import everywhere needed. Then, it can become as simple as putting this on top of all modules using the FFI:

```rust
#[cfg(features = "dlopen")]
use ffi::FOO_STATIC;
#[cfg(not(features = "dlopen"))]
use ffi::*;
```
