use crate::convert::TryFromMrb;
use crate::interpreter::Mrb;
use crate::sys;
use crate::value::Value;
use crate::MrbError;
use std::io::Write;
use std::mem;

use super::*;

pub(super) struct RegexpNew {
    pub pattern: Value,
    pub options: Option<Options>,
    pub encoding: Option<Encoding>,
}

impl RegexpNew {
    pub unsafe fn extract(interp: &Mrb) -> Result<Self, MrbError> {
        let pattern = mem::uninitialized::<sys::mrb_value>();
        let opts = mem::uninitialized::<sys::mrb_value>();
        let has_opts = mem::uninitialized::<sys::mrb_bool>();
        let enc = mem::uninitialized::<sys::mrb_value>();
        let has_enc = mem::uninitialized::<sys::mrb_bool>();
        let mut argspec = vec![];
        argspec
            .write_all(
                format!(
                    "{}{}{}{}{}{}\0",
                    sys::specifiers::OBJECT,
                    sys::specifiers::FOLLOWING_ARGS_OPTIONAL,
                    sys::specifiers::OBJECT,
                    sys::specifiers::PREVIOUS_OPTIONAL_ARG_GIVEN,
                    sys::specifiers::OBJECT,
                    sys::specifiers::PREVIOUS_OPTIONAL_ARG_GIVEN
                )
                .as_bytes(),
            )
            .map_err(|_| MrbError::ArgSpec)?;
        sys::mrb_get_args(
            interp.borrow().mrb,
            argspec.as_ptr() as *const i8,
            &pattern,
            &opts,
            &has_opts,
            &enc,
            &has_enc,
        );
        let pattern = Value::new(&interp, pattern);
        let mut options = None;
        let mut encoding = None;
        // the C boolean as u8 comparisons are easier if we keep the
        // comparison inverted.
        if has_enc != 0 {
            encoding = Some(Encoding::from_value(&interp, enc, false)?);
        } else if has_opts != 0 {
            options = Some(Options::from_value(&interp, opts)?);
            encoding = Some(Encoding::from_value(&interp, opts, true)?);
        }
        Ok(Self {
            pattern,
            options,
            encoding,
        })
    }
}

pub struct Rest {
    pub rest: Vec<Value>,
}

impl Rest {
    pub unsafe fn extract(interp: &Mrb) -> Result<Self, MrbError> {
        let args = mem::uninitialized::<*const sys::mrb_value>();
        let count = mem::uninitialized::<usize>();
        let mut argspec = vec![];
        argspec
            .write_all(sys::specifiers::REST.as_bytes())
            .map_err(|_| MrbError::ArgSpec)?;
        argspec.write_all(b"\0").map_err(|_| MrbError::ArgSpec)?;
        sys::mrb_get_args(
            interp.borrow().mrb,
            argspec.as_ptr() as *const i8,
            &args,
            &count,
        );
        let args = std::slice::from_raw_parts(args, count);
        let args = args
            .iter()
            .map(|value| Value::new(&interp, *value))
            .collect::<Vec<_>>();
        Ok(Self { rest: args })
    }
}

#[derive(Debug, Clone)]
pub struct Match {
    pub string: String,
    pub pos: Option<usize>,
}

impl Match {
    pub unsafe fn extract(interp: &Mrb) -> Result<Self, MrbError> {
        let string = mem::uninitialized::<sys::mrb_value>();
        let pos = mem::uninitialized::<sys::mrb_value>();
        let has_pos = mem::uninitialized::<sys::mrb_bool>();
        let mut argspec = vec![];
        argspec
            .write_all(
                format!(
                    "{}{}{}{}\0",
                    sys::specifiers::OBJECT,
                    sys::specifiers::FOLLOWING_ARGS_OPTIONAL,
                    sys::specifiers::OBJECT,
                    sys::specifiers::PREVIOUS_OPTIONAL_ARG_GIVEN
                )
                .as_bytes(),
            )
            .map_err(|_| MrbError::ArgSpec)?;
        sys::mrb_get_args(
            interp.borrow().mrb,
            argspec.as_ptr() as *const i8,
            &string,
            &pos,
            &has_pos,
        );
        let string = String::try_from_mrb(&interp, Value::new(&interp, string))
            .map_err(MrbError::ConvertToRust)?;
        let pos = if has_pos == 0 {
            None
        } else {
            let pos = usize::try_from_mrb(&interp, Value::new(&interp, pos))
                .map_err(MrbError::ConvertToRust)?;
            Some(pos)
        };
        Ok(Self { string, pos })
    }
}

#[derive(Debug, Clone)]
pub enum MatchIndex {
    Index(i64),
    Name(String),
    StartLen(i64, usize),
}

impl MatchIndex {
    pub unsafe fn extract(interp: &Mrb) -> Result<Self, MrbError> {
        let first = mem::uninitialized::<sys::mrb_value>();
        let second = mem::uninitialized::<sys::mrb_value>();
        let has_second = mem::uninitialized::<sys::mrb_bool>();
        let mut argspec = vec![];
        argspec
            .write_all(
                format!(
                    "{}{}{}{}\0",
                    sys::specifiers::OBJECT,
                    sys::specifiers::FOLLOWING_ARGS_OPTIONAL,
                    sys::specifiers::OBJECT,
                    sys::specifiers::PREVIOUS_OPTIONAL_ARG_GIVEN
                )
                .as_bytes(),
            )
            .map_err(|_| MrbError::ArgSpec)?;
        sys::mrb_get_args(
            interp.borrow().mrb,
            argspec.as_ptr() as *const i8,
            &first,
            &second,
            &has_second,
        );
        if has_second == 0 {
            let mut start = mem::uninitialized::<sys::mrb_int>();
            let mut len = mem::uninitialized::<sys::mrb_int>();
            if sys::mrb_range_beg_len(interp.borrow().mrb, first, &mut start, &mut len, 0, 0_u8)
                == 1
            {
                let len = usize::try_from_mrb(&interp, Value::from_mrb(&interp, len))
                    .map_err(MrbError::ConvertToRust)?;
                Ok(MatchIndex::StartLen(start, len))
            } else {
                i64::try_from_mrb(&interp, Value::new(interp, first))
                    .map(MatchIndex::Index)
                    .or_else(|_| {
                        String::try_from_mrb(&interp, Value::new(interp, first))
                            .map(MatchIndex::Name)
                    })
                    .map_err(MrbError::ConvertToRust)
            }
        } else {
            let start = i64::try_from_mrb(&interp, Value::new(&interp, first))
                .map_err(MrbError::ConvertToRust)?;
            let len = usize::try_from_mrb(&interp, Value::new(&interp, second))
                .map_err(MrbError::ConvertToRust)?;
            Ok(MatchIndex::StartLen(start, len))
        }
    }
}
