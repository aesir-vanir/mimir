// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `oci` macros

#[doc(hidden)]
macro_rules! try_dpi {
    ($code:expr, $ret:expr, $err:expr) => {{
        if unsafe { $code } == ::odpi::constants::DPI_SUCCESS {
            $ret
        } else {
            Err($err.into())
        }
    }};
}

#[doc(hidden)]
#[cfg(feature = "trace")]
macro_rules! logperf {
    ($func:expr, $logger:expr, $msg:expr) => {{
        let timer = ::std::time::Instant::now();
        let res = $func;
        let elapsed = timer.elapsed();
        let elapsed_f64 = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;
        try_trace!($logger, "{}: {:.9}s", $msg, elapsed_f64);
        res
    }};
}
