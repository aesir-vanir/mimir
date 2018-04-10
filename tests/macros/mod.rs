use mimir::error::Result;
use mimir::Context;
use slog::{Drain, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, TermDecorator};

pub fn within_context(f: &Fn(&Context) -> Result<()>) -> Result<()> {
    let stdout_decorator = TermDecorator::new().build();
    let stdout_drain = CompactFormat::new(stdout_decorator).build().fuse();
    let stdout_async_drain = Async::new(stdout_drain).build().fuse();
    let stdout_logger = Logger::root(stdout_async_drain, o!());

    let stderr_decorator = TermDecorator::new().stderr().build();
    let stderr_drain = CompactFormat::new(stderr_decorator).build().fuse();
    let stderr_async_drain = Async::new(stderr_drain).build().fuse();
    let stderr_logger = Logger::root(stderr_async_drain, o!());

    let mut ctxt = Context::create()?;
    ctxt.set_stdout(Some(stdout_logger));
    ctxt.set_stderr(Some(stderr_logger));

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
