extern crate libloading;

pub use libloading::{Library, Symbol};

#[macro_export]
macro_rules! ffi_dispatch(
    ($feature: expr, $handle: ident, $func: ident, $($arg: expr),*) => (
        {
            #[cfg(feature = $feature)]
            let ret = ($handle.$func)($($arg),*);
            #[cfg(not(feature = $feature))]
            let ret = $func($($arg),*);

            ret
        }
    );
);

#[macro_export]
macro_rules! ffi_dispatch_static(
    ($feature: expr, $handle: ident, $name: ident) => (
        {
            #[cfg(feature = $feature)]
            let ret = $handle.$name;
            #[cfg(not(feature = $feature))]
            let ret = &$name;

            ret
        }
    );
);

#[macro_export]
macro_rules! link_external_library(
    ($link: expr,
        $(statics: $($sname: ident: $stype: ty),+,)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+,)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+) -> $vret: ty),+,)|*
    ) => (
        #[link(name = $link)]
        extern "C" {
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

#[macro_export]
macro_rules! dlopen_external_library(
    (__struct, $structname: ident,
        $(statics: $($sname: ident: $stype: ty),+,)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+,)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+) -> $vret: ty),+,)|*
    ) => (
        pub struct $structname {
            __lib: $crate::Library,
            $($(
                pub $sname: $crate::Symbol<'static, &'static $stype>,
            )+)*
            $($(
                pub $fname: $crate::Symbol<'static, unsafe extern "C" fn($($farg),*) -> $fret>,
            )+)*
            $($(
                pub $vname: $crate::Symbol<'static, unsafe extern "C" fn($($vargs),+ , ...) -> $vret>,
            )+)*
        }
    );
    (__impl, $structname: ident,
        $(statics: $($sname: ident: $stype: ty),+,)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+,)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+) -> $vret: ty),+,)|*
    ) => (
    impl $structname {
        pub fn open(name: &str) -> Result<$structname, $crate::DlError> {
            // we use it to ensure the 'static lifetime
            use std::mem::transmute;
            let lib = match $crate::Library::new(name) {
                Ok(l) => l,
                Err(_) => return Err($crate::DlError::NotFound)
            };
            unsafe {
                let s = $structname {
                    $($($sname: {
                        let s_name = concat!(stringify!($sname), "\0");
                        transmute(match lib.get::<&'static $stype>(s_name.as_bytes()) {
                            Ok(s) => s,
                            Err(_) => return Err($crate::DlError::MissingSymbol(s_name))
                        })
                    },
                    )+)*
                    $($($fname: {
                        let s_name = concat!(stringify!($fname), "\0");
                        transmute(match lib.get::<unsafe extern "C" fn($($farg),*) -> $fret>(s_name.as_bytes()) {
                            Ok(s) => s,
                            Err(_) => return Err($crate::DlError::MissingSymbol(s_name))
                        })
                    },
                    )+)*
                    $($($vname: {
                        let s_name = concat!(stringify!($vname), "\0");
                        transmute(match lib.get::<unsafe extern "C" fn($($vargs),+ , ...) -> $vret>(s_name.as_bytes()) {
                            Ok(s) => s,
                            Err(_) => return Err($crate::DlError::MissingSymbol(s_name))
                        })
                    },
                    )+)*
                    __lib: lib
                };
                Ok(s)
            }
        }
    }
    );
    ($structname: ident,
        $(statics: $($sname: ident: $stype: ty),+,)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+,)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+) -> $vret: ty),+,)|*
    ) => (
        $crate::dlopen_external_library!(__struct,
            $structname, $(statics: $($sname: $stype),+,)|*
            $(functions: $(fn $fname($($farg),*) -> $fret),+,)|*
            $(varargs: $(fn $vname($($vargs),+) -> $vret),+,)|*
        );
        $crate::dlopen_external_library!(__impl,
            $structname, $(statics: $($sname: $stype),+,)|*
            $(functions: $(fn $fname($($farg),*) -> $fret),+,)|*
            $(varargs: $(fn $vname($($vargs),+) -> $vret),+,)|*
        );
        unsafe impl Sync for $structname { }
    );
);

#[macro_export]
macro_rules! external_library(
    ($feature: expr, $structname: ident, $link: expr,
        $(statics: $($sname: ident: $stype: ty),+,)|*
        $(functions: $(fn $fname: ident($($farg: ty),*) -> $fret:ty),+,)|*
        $(varargs: $(fn $vname: ident($($vargs: ty),+) -> $vret: ty),+,)|*
    ) => (
        #[cfg(feature = $feature)]
        $crate::dlopen_external_library!(
            $structname, $(statics: $($sname: $stype),+,)|*
            $(functions: $(fn $fname($($farg),*) -> $fret),+,)|*
            $(varargs: $(fn $vname($($vargs),+) -> $vret),+,)|*
        );

        #[cfg(not(feature = $feature))]
        $crate::link_external_library!(
            $link, $(statics: $($sname: $stype),+,)|*
            $(functions: $(fn $fname($($farg),*) -> $fret),+,)|*
            $(varargs: $(fn $vname($($vargs),+) -> $vret),+,)|*
        );
    );
);
