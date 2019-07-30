//! [`MatchData#named_captures`](https://ruby-doc.org/core-2.6.3/MatchData.html#method-i-named_captures)

use std::collections::HashMap;

use crate::convert::{Convert, RustBackedValue};
use crate::extn::core::matchdata::MatchData;
use crate::value::Value;
use crate::Artichoke;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Error {
    Fatal,
    NoMatch,
}

pub fn method(interp: &Artichoke, value: &Value) -> Result<Value, Error> {
    let data = unsafe { MatchData::try_from_ruby(interp, value) }.map_err(|_| Error::Fatal)?;
    let borrow = data.borrow();
    let regex = (*borrow.regexp.regex).as_ref().ok_or(Error::Fatal)?;
    let match_against = &borrow.string[borrow.region.start..borrow.region.end];
    let captures = regex.captures(match_against).ok_or(Error::NoMatch)?;
    // Use a Vec of key-value pairs because insertion order matters for spec
    // compliance.
    let mut map = HashMap::new();
    let mut captures_names = vec![];
    for (idx, name) in regex.capture_names().enumerate() {
        if let Some(name) = name {
            if !map.contains_key(name) {
                captures_names.push(name);
                map.insert(name.to_owned(), vec![]);
            }
            if let Some(matches) = map.get_mut(name) {
                let match_ = captures.get(idx).map(|m| m.as_str().to_owned());
                matches.push(match_);
            }
        }
    }
    let pairs = captures_names
        .into_iter()
        .filter_map(|name| {
            if let Some(matches) = map.remove(name) {
                Some((name.to_owned(), matches))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(Value::convert(interp, pairs))
}
