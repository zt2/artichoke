// This module defines macros for working with interpreters and `Value`s. This
// source module is included first in `lib.rs`, which means the macros are
// available to all modules within the artichoke-backend crate in addition to
// being exported.

/// Extract an [`Artichoke`](interpreter::Artichoke) instance from the userdata on a
/// [`sys::mrb_state`].
///
/// If there is an error when extracting the Rust wrapper around the
/// interpreter, return `nil`.
///
/// This macro is `unsafe`.
#[macro_export]
macro_rules! unwrap_interpreter {
    ($mrb:expr) => {
        if let Ok(interp) = $crate::ffi::from_user_data($mrb) {
            interp
        } else {
            return $crate::sys::mrb_sys_nil_value();
        }
    };
}
