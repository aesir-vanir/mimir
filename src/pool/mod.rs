// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Pool handles are used to represent session pools. They are created using the function `create()`
//! and can be closed by calling the function `close()` or releasing the last reference to the pool
//! by calling the function `release()`. Pools can be used to create connections by calling the
//! function `acquireConnection()`.
use common::encoding;
use connection::Connection;
use context::Context;
use context::params::{CommonCreate, ConnCreate, PoolCreate};
use error::{Error, ErrorKind, Result};
use odpi::opaque::{ODPIConn, ODPIPool};
use odpi::structs::ODPIEncodingInfo;
use odpi::{enums, externs, flags};
use std::convert::TryFrom;
use std::ptr::{self, Unique};
use util::ODPIStr;

/// This structure represents session pools and is available by handle to a calling application or '
/// driver.
pub struct Pool {
    /// An ODPI-C dpiPool opaque struct pointer.
    inner: Unique<ODPIPool>,
}

impl Pool {
    /// Get the `inner` value.
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIPool {
        self.inner.as_ptr()
    }

    /// Acquires a connection from the pool and returns a reference to it. This reference should be
    /// released as soon as it is no longer needed.
    ///
    /// * `username` - the name of the user used for authenticating the user, as a string in the
    /// encoding used for CHAR data. None is also acceptable if external authentication is being
    /// requested or credentials were supplied when the pool was created.
    /// * `password` - the password to use for authenticating the user, as a string in the encoding
    /// used for CHAR data. None is also acceptable if external authentication is being requested or
    /// if credentials were supplied when the pool was created.
    /// * `conn_create_params` - An optional `ConnCreate` structure which is used to specify
    /// parameters for connection creation. None is acceptable in which case all default parameters
    /// will be used when creating the connection.
    pub fn acquire_connection(
        &self,
        username: Option<&str>,
        password: Option<&str>,
        conn_create_params: Option<ConnCreate>,
    ) -> Result<Connection> {
        let username_s: ODPIStr = TryFrom::try_from(username)?;
        let password_s: ODPIStr = TryFrom::try_from(password)?;
        let conn_cp = if let Some(conn_create_params) = conn_create_params {
            conn_create_params
        } else {
            Default::default()
        };
        let mut conn: *mut ODPIConn = ptr::null_mut();

        try_dpi!(
            externs::dpiPool_acquireConnection(
                self.inner.as_ptr(),
                username_s.ptr(),
                username_s.len(),
                password_s.ptr(),
                password_s.len(),
                &mut conn_cp.inner(),
                &mut conn
            ),
            Ok(conn.into()),
            ErrorKind::Pool("dpiPool_acquireConnection".to_string())
        )
    }

    /// Closes the pool and makes it unusable for further activity.
    ///
    /// * `close_mode` - one or more of the values from the enumeration `ODPIPoolCloseMode`, OR'ed
    /// together.
    pub fn close(&self, close_mode: flags::ODPIPoolCloseMode) -> Result<()> {
        try_dpi!(
            externs::dpiPool_close(self.inner.as_ptr(), close_mode),
            Ok(()),
            ErrorKind::Pool("dpiPool_close".to_string())
        )
    }

    /// Creates a session pool which creates and maintains a group of stateless sessions to the
    /// database. The main benefit of session pooling is performance since making a connection to
    /// the database is a time-consuming activity, especially when the database is remote.
    ///
    /// * `context` - the context handle created earlier using the function `Context::create()`.
    /// * `username` - the name of the user used for authenticating sessions, as a string in the
    /// encoding used for CHAR data. None is also acceptable if external authentication is being
    /// requested or if a heterogeneous pool is being created.
    /// * `password` - the password to use for authenticating sessions, as a string in the encoding
    /// used for CHAR data. None is also acceptable if external authentication is being requested or
    ///  if a heterogeneous pool is being created.
    /// * `connect_string` - the connect string identifying the database to which connections are to
    /// be established by the session pool, as a string in the encoding used for CHAR data. None is
    /// also acceptable for local connections (identified by the environment variable ORACLE_SID).
    /// * `common_create_params` - a `CommonCreate` structure which is used to specify context
    /// parameters for pool creation. None is also acceptable in which case all default parameters
    /// will be used when creating the pool.
    /// * `conn_create_params` - a `PoolCreate` structure which is used to specify parameters for
    /// pool creation. None is also acceptable in which case all default parameters will be used for
    /// pool creation.
    pub fn create(
        context: &Context,
        username: Option<&str>,
        password: Option<&str>,
        connect_string: Option<&str>,
        common_create_params: Option<CommonCreate>,
        pool_create_params: Option<PoolCreate>,
    ) -> Result<Self> {
        let username_s: ODPIStr = TryFrom::try_from(username)?;
        let password_s: ODPIStr = TryFrom::try_from(password)?;
        let connect_string_s: ODPIStr = TryFrom::try_from(connect_string)?;
        let mut inner: *mut ODPIPool = ptr::null_mut();

        let comm_cp = if let Some(common_create_params) = common_create_params {
            common_create_params
        } else {
            context.init_common_create_params()?
        };

        let pool_cp = if let Some(pool_create_params) = pool_create_params {
            pool_create_params
        } else {
            context.init_pool_create_params()?
        };

        try_dpi!(
            externs::dpiPool_create(
                context.inner(),
                username_s.ptr(),
                username_s.len(),
                password_s.ptr(),
                password_s.len(),
                connect_string_s.ptr(),
                connect_string_s.len(),
                &comm_cp.inner(),
                &mut pool_cp.inner(),
                &mut inner
            ),
            Ok(TryFrom::try_from(inner)?),
            ErrorKind::Pool("dpiPool_create".to_string())
        )
    }

    /// Returns the number of sessions in the pool that are busy.
    pub fn get_busy_count(&self) -> Result<u32> {
        let mut busy_count = 0;

        try_dpi!(
            externs::dpiPool_getBusyCount(self.inner.as_ptr(), &mut busy_count),
            Ok(busy_count),
            ErrorKind::Pool("dpiPool_getBusyCount".to_string())
        )
    }

    /// Returns the encoding information used by the pool. This will be equivalent to the values
    /// passed when the pool was created, or the values retrieved from the environment variables
    /// NLS_LANG and NLS_NCHAR.
    pub fn get_encoding_info(&self) -> Result<encoding::Info> {
        let mut enc_info: ODPIEncodingInfo = Default::default();

        try_dpi!(
            externs::dpiPool_getEncodingInfo(self.inner.as_ptr(), &mut enc_info),
            Ok(enc_info.into()),
            ErrorKind::Pool("dpiPool_getEncodingInfo".to_string())
        )
    }

    /// Returns the mode used for acquiring or getting connections from the pool.
    pub fn get_get_mode(&self) -> Result<enums::ODPIPoolGetMode> {
        let mut get_mode = enums::ODPIPoolGetMode::NoWait;

        try_dpi!(
            externs::dpiPool_getGetMode(self.inner.as_ptr(), &mut get_mode),
            Ok(get_mode),
            ErrorKind::Pool("dpiPool_getGetMode".to_string())
        )
    }

    /// Returns the maximum lifetime of all sessions in the pool, in seconds. Sessions in the pool
    /// are terminated when this value has been reached, but only when another session is released
    /// back to the pool.
    pub fn get_max_lifetime_session(&self) -> Result<u32> {
        let mut max_lifetime_session = 0;

        try_dpi!(
            externs::dpiPool_getMaxLifetimeSession(self.inner.as_ptr(), &mut max_lifetime_session),
            Ok(max_lifetime_session),
            ErrorKind::Pool("dpiPool_getMaxLifetimeSession".to_string())
        )
    }

    /// Returns the number of sessions in the pool that are open.
    pub fn get_open_count(&self) -> Result<u32> {
        let mut open_count = 0;

        try_dpi!(
            externs::dpiPool_getOpenCount(self.inner.as_ptr(), &mut open_count),
            Ok(open_count),
            ErrorKind::Pool("dpiPool_getOpenCount".to_string())
        )
    }

    /// Returns the default size of the statement cache for sessions in the pool, in number of
    /// statements.
    pub fn get_stmt_cache_size(&self) -> Result<u32> {
        let mut stmt_cache_size = 0;

        try_dpi!(
            externs::dpiPool_getStmtCacheSize(self.inner.as_ptr(), &mut stmt_cache_size),
            Ok(stmt_cache_size),
            ErrorKind::Pool("dpiPool_getStmtCacheSize".to_string())
        )
    }

    /// Returns the amount of time, in seconds, after which idle sessions in the pool are
    /// terminated, but only when another session is released back to the pool.
    pub fn get_timeout(&self) -> Result<u32> {
        let mut timeout = 0;

        try_dpi!(
            externs::dpiPool_getTimeout(self.inner.as_ptr(), &mut timeout),
            Ok(timeout),
            ErrorKind::Pool("dpiPool_getTimeout".to_string())
        )
    }

    /// Sets the mode used for acquiring or getting connections from the pool.
    ///
    /// * `get_mode` - A value from the `ODPIGetMode` enumeration.
    pub fn set_get_mode(&self, get_mode: enums::ODPIPoolGetMode) -> Result<()> {
        try_dpi!(
            externs::dpiPool_setGetMode(self.inner.as_ptr(), get_mode),
            Ok(()),
            ErrorKind::Pool("dpiPool_setGetMode".to_string())
        )
    }

    /// Sets the maximum lifetime of all sessions in the pool, in seconds. Sessions in the pool are
    /// terminated when this value has been reached, but only when another session is released back
    /// to the pool.
    ///
    /// * `max_lifetime` - the maximum lifetime of all sessions in the pool, in seconds.
    pub fn set_max_lifetime_session(&self, max_lifetime: u32) -> Result<()> {
        try_dpi!(
            externs::dpiPool_setMaxLifetimeSession(self.inner.as_ptr(), max_lifetime),
            Ok(()),
            ErrorKind::Pool("dpiPool_setMaxLifetimeSession".to_string())
        )
    }

    /// Sets the default size of the statement cache for sessions in the pool.
    ///
    /// * `stmt_cache_size` - the new size of the statement cache, in number of statements.
    pub fn set_stmt_cache_size(&self, stmt_cache_size: u32) -> Result<()> {
        try_dpi!(
            externs::dpiPool_setStmtCacheSize(self.inner.as_ptr(), stmt_cache_size),
            Ok(()),
            ErrorKind::Pool("dpiPool_setStmtCacheSize".to_string())
        )
    }

    /// Sets the amount of time, in seconds, after which idle sessions in the pool are terminated,
    /// but only when another session is released back to the pool.
    pub fn set_timeout(&self, timeout: u32) -> Result<()> {
        try_dpi!(
            externs::dpiPool_setTimeout(self.inner.as_ptr(), timeout),
            Ok(()),
            ErrorKind::Pool("dpiPool_setTimeout".to_string())
        )
    }
}

impl TryFrom<*mut ODPIPool> for Pool {
    type Error = Error;

    fn try_from(inner: *mut ODPIPool) -> Result<Self> {
        let pool = Unique::new(inner).ok_or_else(|| ErrorKind::Pool("try_from".to_string()))?;
        Ok(Self { inner: pool })
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        if !self.inner.as_ptr().is_null() {
            unsafe {
                externs::dpiPool_release(self.inner.as_ptr());
            }
        }
    }
}
