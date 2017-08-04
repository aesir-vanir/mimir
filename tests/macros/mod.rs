use mimir::Context;
use mimir::error::Result;

pub fn within_context(f: &Fn(&Context) -> Result<()>) -> Result<()> {
    let ctxt = Context::create()?;
    match f(&ctxt) {
        Ok(_) => Ok(()),
        Err(e) => {
            use std::io::{self, Write};
            writeln!(io::stderr(), "{}", ctxt.get_error())?;
            Err(e)
        }
    }
}

macro_rules! check_with_ctxt {
    ($f:ident) => {{
        match $crate::macros::within_context(&$f) {
            Ok(_) => assert!(true),
            Err(e) => {
                use std::io::{self, Write};
                writeln!(io::stderr(), "{}", e).expect("badness");
                assert!(false);
            }
        }
    }};
}
