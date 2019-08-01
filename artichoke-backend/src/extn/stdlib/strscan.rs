use crate::load::LoadSources;
use crate::Artichoke;
use crate::ArtichokeError;

pub fn init(interp: &Artichoke) -> Result<(), ArtichokeError> {
    interp
        .borrow_mut()
        .def_module::<StringScanner>("StringScanner", None);
    interp.def_rb_source_file("strscan.rb", include_str!("strscan.rb"))?;
    Ok(())
}

pub struct StringScanner;
