extern crate libc;

#[cfg(feature = "dlopen")]
#[macro_export]
macro_rules! ffi_dispatch(
    ($handle: ident, $func: ident, $($arg: expr),*) => (
        ($handle.$func)($($arg),*)
    )
);

#[cfg(not(feature = "dlopen"))]
#[macro_export]
macro_rules! ffi_dispatch(
    ($handle: ident, $func: ident, $($arg: expr),*) => (
        $func($($arg),*)
    )
);

#[cfg(feature = "dlopen")]
#[macro_export]
macro_rules! ffi_dispatch_static(
    ($handle: ident, $name: ident) => (
        $handle.$name
    )
);

#[cfg(not(feature = "dlopen"))]
#[macro_export]
macro_rules! ffi_dispatch_static(
    ($handle: ident, $name: ident) => (
        &$name
    )
);

#[cfg(not(feature = "dlopen"))]
#[macro_export]
macro_rules! external_library(
    ($structname: ident, $link: expr,
        $(statics: $($sname: ident: $stype: ty),+)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+ ...) -> $vret: ty),+)|*
    ) => (
        #[link(name = $link)]
        extern {
            $($(
                pub static $sname: $stype;
            )+)*
            $($(
                pub fn $fname($(_: $farg),*) -> $fret;
            )+)*
            $($(
                pub fn $vname($(_: $vargs),+ , ...) -> $vret;
            )+)*
        }
    );
);

pub enum DlError {
    NotFound,
    MissingSymbol(&'static str)
}

#[cfg(feature = "dlopen")]
#[macro_export]
macro_rules! external_library(
    (__struct, $structname: ident, $link: expr,
        $(statics: $($sname: ident: $stype: ty),+)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+ ...) -> $vret: ty),+)|*
    ) => (
        pub struct $structname {
            $($(
                pub $sname: &'static $stype,
            )+)*
            $($(
                pub $fname: unsafe extern fn($($farg),*) -> $fret,
            )+)*
            $($(
                pub $vname: unsafe extern "C" fn($($vargs),+ , ...) -> $vret,
            )+)*
        }
    );
    (__impl, $structname: ident,
        $(statics: $($sname: ident: $stype: ty),+)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+ ...) -> $vret: ty),+)|*
    ) => (
    impl $structname {
        pub fn open(name: &str) -> Result<$structname, DlError> {
            let cname = match ::std::ffi::CString::new(name) {
                Ok(cs) => cs,
                Err(_) => Err(DlError::NotFound)
            };
            unsafe {
                let dl = $crate::ffi::dlopen(cname.as_bytes_with_nul().as_ptr() as *const _, 1);
                if dl.is_null() {
                    return Err(DlError::NotFound);
                }
                $crate::ffi::dlerror();
                let s = $structname {
                    $($($sname: {
                        let s_name = concat!(stringify!($sname), "\0");
                        let symbol = $crate::ffi::dlsym(dl, s_name.as_ptr() as *const _);
                        if !$crate::ffi::dlerror().is_null() {
                            return Err(DlError::MissingSymbol(s_name))
                        }
                        ::std::mem::transmute(symbol)
                    },
                    )+)*
                    $($($fname: {
                        let s_name = concat!(stringify!($fname), "\0");
                        let symbol = $crate::ffi::dlsym(dl, s_name.as_ptr() as *const _);
                        if !$crate::ffi::dlerror().is_null() {
                            return Err(DlError::MissingSymbol(s_name))
                        }
                        ::std::mem::transmute(symbol)
                    },
                    )+)*
                    $($($vname: {
                        let s_name = concat!(stringify!($vname), "\0");
                        let symbol = $crate::ffi::dlsym(dl, s_name.as_ptr() as *const _);
                        if !$crate::ffi::dlerror().is_null() {
                            return Err(DlError::MissingSymbol(s_name))
                        }
                        ::std::mem::transmute(symbol)
                    },
                    )+)*
                };
                Ok(s)
            }
        }
    }
    );
    ($structname: ident,
        $(statics: $($sname: ident: $stype: ty),+)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+ ...) -> $vret: ty),+)|*
    ) => (
        external_library!(__struct,
            $structname, $(statics: $($sname: $stype),+)|*
            $(functions: $(fn $fname($($farg),*) -> $fret),+)|*
           $(varargs: $(fn $vname($($vargs),+ ...) -> $vret),+)|*
        );
        external_library!(__impl,
            $structname, $(statics: $($sname: $stype),+)|*
            $(functions: $(fn $fname($($farg),*) -> $fret),+)|*
           $(varargs: $(fn $vname($($vargs),+ ...) -> $vret),+)|*
        );
        unsafe impl Sync for $structname { }
    );
);

pub mod ffi {
    use libc::{c_char, c_int, c_void};
    extern {
        pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        pub fn dlerror() -> *mut c_char;
        pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        pub fn dlclose(handle: *mut c_void) -> c_int;
    }
}