// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Context handles are the top level handles created by the library and are used for all error
//! handling as well as creating pools and standalone connections to the database. The first call to
//! ODPI-C by any application must be `create()` which will create the context as well asvalidate
//! the version used by the application.
use common::{error, version};
use error::{Error, ErrorKind, Result};
use odpi::constants::{DPI_MAJOR_VERSION, DPI_MINOR_VERSION};
use odpi::externs;
use odpi::opaque::ODPIContext;
use odpi::structs::{
    ODPICommonCreateParams, ODPIConnCreateParams, ODPIErrorInfo, ODPIPoolCreateParams,
    ODPISubscrCreateParams, ODPIVersionInfo,
};
use slog::Logger;
use std::convert::TryFrom;
use std::ptr::{self, Unique};
use util::ODPIStr;

pub mod params;

use self::params::{CommonCreate, ConnCreate, PoolCreate, SubscrCreate};

/// This structure represents the context in which all activity in the library takes place.
#[derive(Clone, Setters)]
pub struct Context {
    /// A pointer the the ODPI-C dpiContext struct.
    inner: Unique<ODPIContext>,
    /// Optional stdout logger.
    #[set = "pub"]
    stdout: Option<Logger>,
    /// Optional stderr logger.
    #[set = "pub"]
    stderr: Option<Logger>,
}

impl Context {
    /// Get the pointer to the inner ODPI struct.
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIContext {
        self.inner.as_ptr()
    }

    /// Create a `Context`
    pub fn create() -> Result<Self> {
        let mut ctxt = ptr::null_mut();
        let mut err: ODPIErrorInfo = Default::default();

        try_dpi!(
            externs::dpiContext_create(DPI_MAJOR_VERSION, DPI_MINOR_VERSION, &mut ctxt, &mut err),
            Ok(TryFrom::try_from(ctxt)?),
            "dpiContext_create".to_string()
        )
    }

    /// Return information about the version of the Oracle Client that is being used.
    #[cfg(feature = "trace")]
    pub fn get_client_version(&self) -> Result<version::Info> {
        logperf!(
            self.base_get_client_version(),
            self.stdout,
            "get_client_version"
        )
    }

    /// Return information about the version of the Oracle Client that is being used.
    #[cfg(not(feature = "trace"))]
    pub fn get_client_version(&self) -> Result<version::Info> {
        self.base_get_client_version()
    }

    /// The base (non-traced) version of `get_client_version`.
    fn base_get_client_version(&self) -> Result<version::Info> {
        let mut version_info: ODPIVersionInfo = Default::default();
        try_dpi!(
            externs::dpiContext_getClientVersion(self.inner.as_ptr(), &mut version_info),
            Ok(version_info.into()),
            ErrorKind::Context("dpiContext_getClientVersion".to_string())
        )
    }

    /// Returns error information for the last error that was raised by the library. This function
    /// must be called with the same thread that generated the error. It must also be called before
    /// any other ODPI-C library calls are made on the calling thread since the error information
    /// specific to that thread is cleared at the start of every ODPI-C function call.
    pub fn get_error(&self) -> error::Info {
        let mut error_info: ODPIErrorInfo = Default::default();
        unsafe {
            externs::dpiContext_getError(self.inner.as_ptr(), &mut error_info);
            error_info.into()
        }
    }

    /// Initializes the `CommonCreate` structure to default values.
    pub fn init_common_create_params(&self) -> Result<CommonCreate> {
        let mut ccp: ODPICommonCreateParams = Default::default();

        try_dpi!(
            externs::dpiContext_initCommonCreateParams(self.inner.as_ptr(), &mut ccp),
            {
                let mut driver_name = String::from(env!("CARGO_PKG_NAME"));
                driver_name.push(' ');
                driver_name.push_str(env!("CARGO_PKG_VERSION"));
                let driver_name_s: ODPIStr = TryFrom::try_from(driver_name)?;
                ccp.driver_name = driver_name_s.ptr();
                ccp.driver_name_length = driver_name_s.len();
                Ok(ccp.into())
            },
            ErrorKind::Context("dpiContext_initCommonCreateParams".to_string())
        )
    }

    /// Initializes the `ConnCreate` structure to default values.
    pub fn init_conn_create_params(&self) -> Result<ConnCreate> {
        let mut conn: ODPIConnCreateParams = Default::default();

        try_dpi!(
            externs::dpiContext_initConnCreateParams(self.inner.as_ptr(), &mut conn),
            Ok(ConnCreate::new(conn)),
            ErrorKind::Context("dpiContext_initConnCreateParams".to_string())
        )
    }

    /// Initializes the `PoolCreate` structure to default values.
    pub fn init_pool_create_params(&self) -> Result<PoolCreate> {
        let mut pool: ODPIPoolCreateParams = Default::default();
        try_dpi!(
            externs::dpiContext_initPoolCreateParams(self.inner.as_ptr(), &mut pool),
            Ok(PoolCreate::new(pool)),
            ErrorKind::Context("dpiContext_initPoolCreateParams".to_string())
        )
    }

    /// Initializes the `SubscrCreate` struct to default values.
    pub fn init_subscr_create_params(&self) -> Result<SubscrCreate> {
        let mut subscr: ODPISubscrCreateParams = Default::default();
        try_dpi!(
            externs::dpiContext_initSubscrCreateParams(self.inner.as_ptr(), &mut subscr),
            Ok(SubscrCreate::new(subscr)),
            ErrorKind::Context("dpiContext_initSubscrCreateParams".to_string())
        )
    }
}

impl TryFrom<*mut ODPIContext> for Context {
    type Error = Error;

    fn try_from(inner: *mut ODPIContext) -> Result<Self> {
        let ctxt = Unique::new(inner).ok_or_else(|| ErrorKind::Context("try_from".to_string()))?;
        Ok(Self {
            inner: ctxt,
            stdout: None,
            stderr: None,
        })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.inner.as_ptr().is_null() {
            unsafe {
                externs::dpiContext_destroy(self.inner.as_ptr());
            }
        }
    }
}
