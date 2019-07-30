use log::trace;

use crate::convert::TryConvert;
use crate::def::{ClassLike, Define};
use crate::eval::Eval;
use crate::extn::core::error::{ArgumentError, RubyException, RuntimeError, TypeError};
use crate::sys;
use crate::value::Value;
use crate::{Artichoke, ArtichokeError};

mod scan;

pub fn patch(interp: &Artichoke) -> Result<(), ArtichokeError> {
    if interp.borrow().class_spec::<RString>().is_some() {
        return Ok(());
    }
    let string = interp
        .borrow_mut()
        .def_class::<RString>("String", None, None);
    interp.eval(include_str!("string.rb"))?;
    string
        .borrow_mut()
        .add_method("ord", RString::ord, sys::mrb_args_none());
    string
        .borrow_mut()
        .add_method("scan", RString::scan, sys::mrb_args_req(1));
    string
        .borrow()
        .define(interp)
        .map_err(|_| ArtichokeError::New)?;
    trace!("Patched String onto interpreter");
    Ok(())
}

#[allow(clippy::module_name_repetitions)]
pub struct RString;

impl RString {
    unsafe extern "C" fn ord(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        let interp = unwrap_interpreter!(mrb);
        if let Ok(s) = String::try_convert(&interp, Value::new(&interp, slf)) {
            if let Some(first) = s.chars().next() {
                // One UTF-8 character, which are at most 32 bits.
                if let Ok(value) = Value::try_convert(&interp, first as u32) {
                    value.inner()
                } else {
                    drop(s);
                    ArgumentError::raise(interp, "Unicode out of range")
                }
            } else {
                drop(s);
                ArgumentError::raise(interp, "empty string")
            }
        } else {
            sys::mrb_sys_nil_value()
        }
    }

    unsafe extern "C" fn scan(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(&interp, slf);
        let result =
            scan::Args::extract(&interp).and_then(|args| scan::method(&interp, args, value));

        match result {
            Ok(result) => result.inner(),
            Err(scan::Error::WrongType) => {
                TypeError::raise(interp, "wrong argument type (expected Regexp)")
            }
            Err(scan::Error::Fatal) => RuntimeError::raise(interp, "fatal String#scan error"),
        }
    }
}
