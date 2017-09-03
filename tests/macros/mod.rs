use mimir::{Context, ContextBuilder};
use mimir::error::Result;
use slog::{Drain, Logger};
use slog_term;
use std::io;

pub fn within_context(f: &Fn(&Context) -> Result<()>) -> Result<()> {
    let stdout_plain = slog_term::PlainSyncDecorator::new(io::stdout());
    let stdout_logger = Logger::root(
        slog_term::FullFormat::new(stdout_plain).build().fuse(),
        o!(),
    );
    let stderr_plain = slog_term::PlainSyncDecorator::new(io::stderr());
    let stderr_logger = Logger::root(
        slog_term::FullFormat::new(stderr_plain).build().fuse(),
        o!(),
    );
    let ctxt: Context = ContextBuilder::default()
        .stdout(Some(stdout_logger))
        .stderr(Some(stderr_logger))
        .build()?;
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
