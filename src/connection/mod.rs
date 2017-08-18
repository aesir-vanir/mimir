// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Connection handles are used to represent connections to the database. These can be standalone
//! connections created by calling the function `create()` or acquired from a session pool by
//! calling the function `Pool::acquireConnection()`. They can be closed by calling the function
//! `close()` or releasing the last reference to the connection by calling the function `release()`.
//! Connection handles are used to create all handles other than session pools and context handles.
use common::{encoding, version};
use context::Context;
use context::params::{CommonCreate, ConnCreate, SubscrCreate};
use dequeue;
use enqueue;
use error::{ErrorKind, Result};
use lob::Lob;
use message::Properties;
use object::Object;
use objecttype::ObjectType;
use odpi::{enums, externs, flags};
use odpi::opaque::ODPIConn;
use odpi::structs::{ODPIEncodingInfo, ODPIVersionInfo};
use slog::Logger;
use statement::Statement;
use std::ffi::{CStr, CString};
use std::ptr;
use subscription::Subscription;
use util::ODPIStr;
use variable::Var;

/// Connection handles are used to represent connections to the database.
#[allow(dead_code)]
pub struct Connection {
    /// The ODPI-C connection.
    inner: *mut ODPIConn,
    /// Optional stdout logger.
    stdout: Option<Logger>,
    /// Optoinal stderr logger.
    stderr: Option<Logger>,
}

impl Connection {
    /// Adds a reference to the connection. This is intended for situations where a reference to the
    /// connection needs to be maintained independently of the reference returned when the
    /// connection was created.
    pub fn add_ref(&self) -> Result<()> {
        try_dpi!(
            externs::dpiConn_addRef(self.inner),
            Ok(()),
            ErrorKind::Connection("dpiConn_addRef".to_string())
        )
    }

    /// Begins a distributed transaction using the specified transaction id (XID) made up of the
    /// formatId, transactionId and branchId.
    ///
    /// * `format_id` - the identifier of the format of the XID. A value of -1 indicates that the
    /// entire XID is null.
    /// * `txn_id` - the global transaction id of the XID as a byte string. The maximum length
    /// permitted is 64 bytes.
    /// * `branch_id` - the branch id of the XID as a byte string. The maximum length permitted is
    /// 64 bytes.
    pub fn begin_distrib_trans(&self, format_id: i64, txn_id: &str, branch_id: &str) -> Result<()> {
        let txn_id_s = ODPIStr::from(txn_id);
        let branch_id_s = ODPIStr::from(branch_id);

        if txn_id_s.len() > 64 {
            Err(ErrorKind::TxnId.into())
        } else if branch_id_s.len() > 64 {
            Err(ErrorKind::BranchId.into())
        } else {
            try_dpi!(
                externs::dpiConn_beginDistribTrans(
                    self.inner,
                    format_id,
                    txn_id_s.ptr(),
                    txn_id_s.len(),
                    branch_id_s.ptr(),
                    branch_id_s.len()
                ),
                Ok(()),
                ErrorKind::Connection("dpiConn_beginDistribTrans".to_string())
            )
        }
    }

    /// Performs an immediate (asynchronous) termination of any currently executing function on the
    /// server associated with the connection.
    pub fn break_execution(&self) -> Result<()> {
        try_dpi!(
            externs::dpiConn_breakExecution(self.inner),
            Ok(()),
            ErrorKind::Connection("dpiConn_breakExecution".to_string())
        )
    }

    /// Changes the password of the specified user.
    ///
    /// * `username` - the name of the user whose password is to be changed, as a byte string in the
    /// encoding used for CHAR data.
    /// * `old_password` - the old password of the user whose password is to be changed, as a byte
    /// string in the encoding used for CHAR data.
    /// * `new_password` - the new password of the user whose password is to be changed, as a byte
    /// string in the encoding used for CHAR data.
    pub fn change_password(
        &self,
        username: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        let username_s = ODPIStr::from(username);
        let old_password_s = ODPIStr::from(old_password);
        let new_password_s = ODPIStr::from(new_password);

        try_dpi!(
            externs::dpiConn_changePassword(
                self.inner,
                username_s.ptr(),
                username_s.len(),
                old_password_s.ptr(),
                old_password_s.len(),
                new_password_s.ptr(),
                new_password_s.len()
            ),
            Ok(()),
            ErrorKind::Connection("dpiConn_changePassword".to_string())
        )
    }

    /// Closes the connection and makes it unusable for further activity.
    ///
    /// * `mode` - one or more of the values from the enumeration `ODPIConnCloseMode`, OR'ed
    /// together.
    /// * `tag` - a byte string in the encoding used for CHAR data, indicating what tag should be
    /// set on the connection when it is released back to the pool. None is also acceptable when
    /// indicating that the tag should be cleared. This value is ignored unless the close mode
    /// includes the value DPI_MODE_CONN_CLOSE_RETAG.
    pub fn close(&self, mode: flags::ODPIConnCloseMode, tag: Option<&str>) -> Result<()> {
        let tag_s = ODPIStr::from(tag);

        try_dpi!(
            externs::dpiConn_close(self.inner, mode, tag_s.ptr(), tag_s.len()),
            Ok(()),
            ErrorKind::Connection("dpiConn_close".to_string())
        )
    }

    /// Commits the current active transaction.
    pub fn commit(&self) -> Result<()> {
        try_dpi!(
            externs::dpiConn_commit(self.inner),
            Ok(()),
            ErrorKind::Connection("dpiConn_commit".to_string())
        )
    }

    /// Creates a standalone connection to a database or acquires a connection from a session pool
    /// and returns a reference to the connection.
    ///
    /// * `context` - the context handle created earlier using the function `Context::create()`. If
    /// the handle is NULL or invalid an error is returned.
    /// * `username` - the name of the user used for authenticating the user, as a byte string in
    /// the encoding used for CHAR data. None is also acceptable if external authentication is being
    /// requested or if credentials were specified when the pool was created.
    /// * `password` - the password to use for authenticating the user, as a byte string in the
    /// encoding used for CHAR data. None is also acceptable if external authentication is being
    /// requested or if credentials were specified when the pool was created.
    /// * `connect_string` - the connect string identifying the database to which a connection is to
    /// be established, as a byte string in the encoding used for CHAR data. None is also acceptable
    /// for local connections (identified by the environment variable $ORACLE_SID) or when a
    /// connection is being acquired from a session pool. This value is ignored when a connection is
    /// being acquired from a session pool.
    /// * `common_create_params` - An optional `CommonCreate` structure which is used to specify
    /// context parameters for connection creation. None is also acceptable in which case all
    /// default parameters will be used when creating the connection. This value is ignored when a
    /// cquiring a connection from a session pool.
    /// * `conn_create_params` - An optional `ConnCreate` structure which is used to specify
    /// parameters for connection creation. None is also acceptable in which case all default
    /// parameters will be used when creating the connection.
    pub fn create(
        context: &Context,
        username: Option<&str>,
        password: Option<&str>,
        connect_string: Option<&str>,
        common_create_params: Option<CommonCreate>,
        conn_create_params: Option<ConnCreate>,
    ) -> Result<Connection> {
        let username_s = ODPIStr::from(username);
        let password_s = ODPIStr::from(password);
        let connect_string_s = ODPIStr::from(connect_string);
        let mut inner: *mut ODPIConn = ptr::null_mut();

        let comm_cp = if let Some(common_create_params) = common_create_params {
            common_create_params
        } else {
            context.init_common_create_params()?
        };

        let conn_cp = if let Some(conn_create_params) = conn_create_params {
            conn_create_params
        } else {
            context.init_conn_create_params()?
        };

        try_dpi!(
            externs::dpiConn_create(
                context.inner(),
                username_s.ptr(),
                username_s.len(),
                password_s.ptr(),
                password_s.len(),
                connect_string_s.ptr(),
                connect_string_s.len(),
                &comm_cp.inner(),
                &mut conn_cp.inner(),
                &mut inner
            ),
            Ok(inner.into()),
            ErrorKind::Connection("dpiConn_create".to_string())
        )
    }

    /// Dequeues a message from a queue.
    ///
    /// * `queue_name` - the name of the queue from which the message is to be dequeued, as a byte
    /// string in the encoding used for CHAR data.
    /// * `options` - a reference to the dequeue options that should be used when dequeuing the
    /// message from the queue.
    /// * `props` -- a reference to the message properties that will be populated with information
    /// from the message that is dequeued.
    pub fn deque_object(
        &self,
        queue_name: &str,
        options: &dequeue::Options,
        props: &Properties,
    ) -> Result<(String, Object)> {
        let queue_s = ODPIStr::from(queue_name);
        let payload = ptr::null_mut();
        let mut pdst = ptr::null();
        let mut dstlen = 0;

        try_dpi!(
            externs::dpiConn_deqObject(
                self.inner,
                queue_s.ptr(),
                queue_s.len(),
                options.inner(),
                props.inner(),
                payload,
                &mut pdst,
                &mut dstlen
            ),
            Ok((ODPIStr::new(pdst, dstlen).into(), payload.into())),
            ErrorKind::Connection("dpiConn_deqObject".to_string())
        )
    }

    /// Enqueues a message to a queue.
    ///
    /// * `queue_name` - the name of the queue to which the message is to be enqueued, as a byte
    /// string in the encoding used for CHAR data.
    /// * `options` - a reference to the enqueue options that should be used when enqueuing the
    /// message to the queue.
    /// * `props` - a reference to the message properties that will affect the message that is
    /// enqueued.
    pub fn enqueue_object(
        &self,
        queue_name: &str,
        options: &enqueue::Options,
        props: &Properties,
    ) -> Result<(String, Object)> {
        let payload = ptr::null_mut();
        let queue_s = ODPIStr::from(queue_name);
        let mut pdst = ptr::null();
        let mut dstlen = 0;

        try_dpi!(
            externs::dpiConn_enqObject(
                self.inner,
                queue_s.ptr(),
                queue_s.len(),
                options.inner(),
                props.inner(),
                payload,
                &mut pdst,
                &mut dstlen
            ),
            Ok((ODPIStr::new(pdst, dstlen).into(), payload.into())),
            ErrorKind::Connection("dpiConn_enqObject".to_string())
        )
    }

    /// Get the current schema.
    pub fn get_current_schema(&self) -> Result<String> {
        let mut pdst = ptr::null();
        let mut dstlen = 0;

        try_dpi!(
            externs::dpiConn_getCurrentSchema(self.inner, &mut pdst, &mut dstlen),
            Ok(ODPIStr::new(pdst, dstlen).into()),
            ErrorKind::Connection("dpiConn_getCurrentSchema".to_string())
        )
    }

    /// Returns the edition that is being used by the connection.
    pub fn get_edition(&self) -> Result<String> {
        let mut pdst = ptr::null();
        let mut dstlen = 0;

        try_dpi!(
            externs::dpiConn_getEdition(self.inner, &mut pdst, &mut dstlen),
            Ok(ODPIStr::new(pdst, dstlen).into()),
            ErrorKind::Connection("dpiConn_getEdition".to_string())
        )
    }

    /// Returns the encoding information used by the connection. This will be equivalent to the
    /// values passed when the standalone connection or session pool was created, or the values
    /// retrieved from the environment variables NLS_LANG and NLS_NCHAR.
    pub fn get_encoding_info(&self) -> Result<encoding::Info> {
        let mut encoding_info: ODPIEncodingInfo = Default::default();

        try_dpi!(
            externs::dpiConn_getEncodingInfo(self.inner, &mut encoding_info),
            Ok(encoding_info.into()),
            ErrorKind::Connection("dpiConn_getEncodingInfo".to_string())
        )
    }

    /// Returns the external name that is being used by the connection. This value is used when
    /// logging distributed transactions.
    pub fn get_external_name(&self) -> Result<String> {
        let mut pdst = ptr::null();
        let mut dstlen = 0;

        try_dpi!(
            externs::dpiConn_getExternalName(self.inner, &mut pdst, &mut dstlen),
            {
                if pdst.is_null() {
                    let err = "dpiConn_getExternalName: null pointer!".to_string();
                    Err(ErrorKind::Connection(err).into())
                } else {
                    let external_name_cstr = unsafe { CStr::from_ptr(pdst) };
                    let external_name = external_name_cstr.to_string_lossy().into_owned();
                    if external_name.len() == dstlen as usize {
                        Ok(external_name_cstr.to_string_lossy().into_owned())
                    } else {
                        let err = "dpiConn_getExternalName: invalid string length!".to_string();
                        Err(ErrorKind::Connection(err).into())
                    }
                }
            },
            ErrorKind::Connection("dpiConn_getExternalName".to_string())
        )
    }

    #[doc(hidden)]
    /// Returns the OCI service context handle in use by the connection. This is a OCI_HTYPE_SVCCTX
    /// handle pointing to an OCISvcCtx struct from the OCI library.
    // TODO: Don't expose c_void.
    pub fn get_handle(&self) -> Result<()> {
        let mut pdst = ptr::null_mut();

        try_dpi!(
            externs::dpiConn_getHandle(self.inner, &mut pdst),
            {
                // TODO: cast pdst to a svcctx struct
                Ok(())
            },
            ErrorKind::Connection("dpiConn_getHandle".to_string())
        )
    }

    /// Returns the internal name that is being used by the connection. This value is used when
    /// logging distributed transactions.
    pub fn get_internal_name(&self) -> Result<String> {
        let mut pdst = ptr::null();
        let mut dstlen = 0;

        try_dpi!(
            externs::dpiConn_getInternalName(self.inner, &mut pdst, &mut dstlen),
            {
                if pdst.is_null() {
                    let err = "dpiConn_getInternalName: null pointer!".to_string();
                    Err(ErrorKind::Connection(err).into())
                } else {
                    let external_name_cstr = unsafe { CStr::from_ptr(pdst) };
                    let external_name = external_name_cstr.to_string_lossy().into_owned();
                    if external_name.len() == dstlen as usize {
                        Ok(external_name_cstr.to_string_lossy().into_owned())
                    } else {
                        let err = "dpiConn_getInternalName: invalid string length!".to_string();
                        Err(ErrorKind::Connection(err).into())
                    }
                }
            },
            ErrorKind::Connection("dpiConn_getInternalName".to_string())
        )
    }

    /// Returns the logical transaction id for the connection. This value is used in Transaction
    /// Guard to determine if the last failed call was completed and if the transaction was
    /// committed using the procedure call dbms_app_cont.get_ltxid_outcome().
    pub fn get_ltxid(&self) -> Result<String> {
        let mut pdst = ptr::null();
        let mut dstlen = 0;

        try_dpi!(
            externs::dpiConn_getLTXID(self.inner, &mut pdst, &mut dstlen),
            Ok(ODPIStr::new(pdst, dstlen).into()),
            ErrorKind::Connection("dpiConn_getLTXID".to_string())
        )
    }

    /// Looks up an object type by name in the database and returns a reference to it. The reference
    /// should be released as soon as it is no longer needed.
    ///
    /// * `name` - the name of the object type to lookup, as a string in the encoding used for
    /// CHAR data.
    pub fn get_object_type(&self, name: &str) -> Result<ObjectType> {
        let mut pobj = ptr::null_mut();
        let name_s = ODPIStr::from(name);

        try_dpi!(
            externs::dpiConn_getObjectType(self.inner, name_s.ptr(), name_s.len(), &mut pobj),
            Ok(pobj.into()),
            ErrorKind::Connection("dpiConn_getObjectType".to_string())
        )
    }

    /// Returns the version information of the Oracle Database to which the connection has been
    /// made.
    pub fn get_server_version(&self) -> Result<version::Info> {
        let mut pdst = ptr::null();
        let mut dstlen = 0;
        let mut version_info: ODPIVersionInfo = Default::default();

        try_dpi!(
            externs::dpiConn_getServerVersion(
                self.inner,
                &mut pdst,
                &mut dstlen,
                &mut version_info
            ),
            {
                let mut ver_info: version::Info = version_info.into();
                let release_s = ODPIStr::new(pdst, dstlen);
                ver_info.set_release(Some(release_s.into()));
                Ok(ver_info)
            },
            ErrorKind::Connection("dpiConn_getServerVersion".to_string())
        )
    }

    /// Returns the size of the statement cache, in number of statements.
    pub fn get_statement_cache_size(&self) -> Result<u32> {
        let mut size = 0;

        try_dpi!(
            externs::dpiConn_getStmtCacheSize(self.inner, &mut size),
            Ok(size),
            ErrorKind::Connection("dpiConn_getStmtCacheSize".to_string())
        )
    }

    /// Returns a reference to a new set of dequeue options, used in dequeuing objects from a queue.
    /// The reference should be released as soon as it is no longer needed.
    pub fn new_deq_options(&self) -> Result<dequeue::Options> {
        let mut deq_ptr = ptr::null_mut();

        try_dpi!(
            externs::dpiConn_newDeqOptions(self.inner, &mut deq_ptr),
            Ok(deq_ptr.into()),
            ErrorKind::Connection("dpiConn_newDeqOptions".to_string())
        )
    }

    /// Returns a reference to a new set of enqueue options, used in enqueuing objects into a queue.
    /// The reference should be released as soon as it is no longer needed.
    pub fn new_enq_options(&self) -> Result<enqueue::Options> {
        let mut enq_ptr = ptr::null_mut();

        try_dpi!(
            externs::dpiConn_newEnqOptions(self.inner, &mut enq_ptr),
            Ok(enq_ptr.into()),
            ErrorKind::Connection("dpiConn_newEnqOptions".to_string())
        )
    }

    /// Returns a reference to a new set of message properties, used in enqueuing and dequeuing
    /// objects in a queue. The reference should be released as soon as it is no longer needed.
    pub fn new_msg_props(&self) -> Result<Properties> {
        let mut msg_props_ptr = ptr::null_mut();
        try_dpi!(
            externs::dpiConn_newMsgProps(self.inner, &mut msg_props_ptr),
            Ok(msg_props_ptr.into()),
            ErrorKind::Connection("dpiConn_newMsgProps".to_string())
        )
    }

    /// Returns a reference to a subscription which is used for requesting notifications of changes
    /// on tables or queries that are made in the database. The reference should be released as soon
    /// as it is no longer needed.
    pub fn new_subscription(
        &self,
        subscr_create_params: SubscrCreate,
    ) -> Result<(u64, Subscription)> {
        let mut subscr_ptr = ptr::null_mut();
        let mut subscr_id = 0;

        try_dpi!(
            externs::dpiConn_newSubscription(
                self.inner,
                &mut subscr_create_params.inner(),
                &mut subscr_ptr,
                &mut subscr_id
            ),
            {
                if subscr_ptr.is_null() {
                    Err(
                        ErrorKind::Connection("dpiConn_newSubscription".to_string()).into(),
                    )
                } else {
                    let sub: Subscription = subscr_ptr.into();
                    Ok((subscr_id, sub))
                }
            },
            ErrorKind::Connection("dpiConn_newSubscription".to_string())
        )
    }

    /// Returns a reference to a new temporary LOB which may subsequently be written and bound to a
    /// statement. The reference should be released as soon as it is no longer needed.
    ///
    /// * `lob_type` - the type of LOB which should be created. It should be one of these values
    /// from the enumeration `ODPIOracleTypeNum`: `Clob`, `NClob` or `Blob`.
    pub fn new_temp_lob(&self, lob_type: enums::ODPIOracleTypeNum) -> Result<Lob> {
        let mut lob_ptr = ptr::null_mut();

        match lob_type {
            enums::ODPIOracleTypeNum::Clob |
            enums::ODPIOracleTypeNum::NClob |
            enums::ODPIOracleTypeNum::Blob => {}
            _ => {
                return Err(
                    ErrorKind::Connection("invalid oracle type".to_string()).into(),
                )
            }
        }

        try_dpi!(
            externs::dpiConn_newTempLob(self.inner, lob_type, &mut lob_ptr),
            Ok(lob_ptr.into()),
            ErrorKind::Connection("dpiConn_newTempLob".to_string())
        )
    }

    /// Returns a reference to a new variable which can be used for binding data to a statement or
    /// providing a buffer for querying data from the database. The reference should be released as
    /// soon as it is no longer needed.
    ///
    /// * `oracle_type_num` - the type of Oracle data that is to be used. It should be one of the
    /// values from the enumeration `ODPIOracleTypeNum`
    /// * `native_type_num` - the type of native C data that is to be used. It should be one of the
    /// values from the enumeration `ODPINativeTypeNum`
    /// * `max_array_size` - the maximum number of rows that can be fetched or bound at one time
    /// from the database, or the maximum number of elements that can be stored in a PL/SQL array.
    /// * `size` - the maximum size of the buffer used for transferring data to/from Oracle. This
    /// value is only used for variables transferred as byte strings. Size is either in characters
    /// or bytes depending on the value of the `size_is_bytes` parameter. If the value is in
    /// characters, internally the value will be multipled by the maximum number of bytes for each
    /// character and that value used instead when determining the necessary buffer size.
    /// * `size_is_bytes` - boolean value indicating if the size parameter refers to characters or
    /// bytes. This flag is only used if the variable refers to character data.
    /// * `is_array` - boolean value indicating if the variable refers to a PL/SQL array or simply
    /// to buffers used for binding or fetching data.
    pub fn new_var(
        &self,
        oracle_type_num: enums::ODPIOracleTypeNum,
        native_type_num: enums::ODPINativeTypeNum,
        max_array_size: u32,
        size: u32,
        size_is_bytes: bool,
        is_array: bool,
    ) -> Result<Var> {
        let mut var_ptr = ptr::null_mut();
        let mut data_ptr = ptr::null_mut();
        let object_type = ptr::null_mut();

        let sib = if size_is_bytes { 1 } else { 0 };
        let ia = if is_array { 1 } else { 0 };

        /// TODO: Fix object_type when Object is implemented fully.
        try_dpi!(
            externs::dpiConn_newVar(
                self.inner,
                oracle_type_num,
                native_type_num,
                max_array_size,
                size,
                sib,
                ia,
                object_type,
                &mut var_ptr,
                &mut data_ptr
            ),
            Ok(var_ptr.into()),
            ErrorKind::Connection("dpiConn_newVar".to_string())
        )
    }

    /// Pings the database to verify that the connection is still alive.
    pub fn ping(&self) -> Result<()> {
        try_dpi!(
            externs::dpiConn_ping(self.inner),
            Ok(()),
            ErrorKind::Connection("dpiConn_ping".to_string())
        )
    }

    /// Prepares a distributed transaction for commit. This function should only be called after
    /// dpiConn_beginDistribTrans() is called and before dpiConn_commit() is called.
    pub fn prepare_distrib_trans(&self) -> Result<bool> {
        let mut commit_needed = 0;
        try_dpi!(
            externs::dpiConn_prepareDistribTrans(self.inner, &mut commit_needed),
            Ok(commit_needed != 0),
            ErrorKind::Connection("dpiConn_prepareDistribTrans".to_string())
        )
    }

    /// Returns a reference to a statement prepared for execution. The reference should be released
    /// as soon as it is no longer needed.
    ///
    /// * `sql` - the SQL that is to be prepared for execution, as a string in the encoding used for
    ///  CHAR data. The value can also be None if the tag parameter is specified.
    /// * `tag` - the key to be used for searching for the statement in the statement cache, as a
    /// string in the encoding used for CHAR data. The value can also be None if the sql parameter
    /// is specified.
    /// * `scrollable` - a boolean indicating if the statement is scrollable or not. If it is
    /// scrollable, `Statement::scroll()` can be used to reposition the cursor; otherwise, rows are
    /// retrieved in order from the statement until the rows are exhausted. This value is ignored
    /// for statements that do not refer to a query.
    pub fn prepare_stmt(
        &self,
        sql: Option<&str>,
        tag: Option<&str>,
        scrollable: bool,
    ) -> Result<Statement> {
        let sql_s = ODPIStr::from(sql);
        let tag_s = ODPIStr::from(tag);
        let scroll_i = if scrollable { 0 } else { 1 };
        let mut stmt_ptr = ptr::null_mut();

        try_dpi!(
            externs::dpiConn_prepareStmt(
                self.inner,
                scroll_i,
                sql_s.ptr(),
                sql_s.len(),
                tag_s.ptr(),
                tag_s.len(),
                &mut stmt_ptr
            ),
            Ok(Statement::new(stmt_ptr)),
            ErrorKind::Connection("dpiConn_prepareStmt".to_string())
        )
    }

    /// Releases a reference to the connection. A count of the references to the connection is
    /// maintained and when this count reaches zero, the memory associated with the connection is
    /// freed and the connection is closed or released back to the session pool if that has not
    /// already taken place using the function `close()`.
    pub fn release(&self) -> Result<()> {
        try_dpi!(
            externs::dpiConn_release(self.inner),
            Ok(()),
            ErrorKind::Connection("dpiConn_release".to_string())
        )
    }

    /// Rolls back the current active transaction.
    pub fn rollback(&self) -> Result<()> {
        try_dpi!(
            externs::dpiConn_rollback(self.inner),
            Ok(()),
            ErrorKind::Connection("dpiConn_rollback".to_string())
        )
    }

    /// Sets the action attribute on the connection. This is one of the end-to-end tracing
    /// attributes that can be tracked in database views, shown in audit trails and seen in tools
    /// such as Enterprise Manager.
    ///
    /// * `action` - a string in the encoding used for CHAR data which will be used to set the
    /// action attribute.
    pub fn set_action(&self, action: &str) -> Result<()> {
        let action_s = ODPIStr::from(action);

        try_dpi!(
            externs::dpiConn_setAction(self.inner, action_s.ptr(), action_s.len()),
            Ok(()),
            ErrorKind::Connection("dpiConn_setAction".to_string())
        )
    }

    /// Sets the client identifier attribute on the connection. This is one of the end-to-end
    /// tracing attributes that can be tracked in database views, shown in audit trails and seen in
    /// tools such as Enterprise Manager.
    ///
    /// * `id` - a string in the encoding used for CHAR data which will be used to set the client
    /// identifier attribute.
    pub fn set_client_identifier(&self, id: &str) -> Result<()> {
        let id_s = ODPIStr::from(id);

        try_dpi!(
            externs::dpiConn_setClientIdentifier(self.inner, id_s.ptr(), id_s.len()),
            Ok(()),
            ErrorKind::Connection("dpiConn_setClientIdentifier".to_string())
        )
    }

    /// Sets the client info attribute on the connection. This is one of the end-to-end tracing
    /// attributes that can be tracked in database views, shown in audit trails and seen in tools
    /// such as Enterprise Manager.
    ///
    /// * `info` - a string in the encoding used for CHAR data which will be used to set the client
    /// info attribute.
    pub fn set_client_info(&self, info: &str) -> Result<()> {
        let info_s = ODPIStr::from(info);

        try_dpi!(
            externs::dpiConn_setClientInfo(self.inner, info_s.ptr(), info_s.len()),
            Ok(()),
            ErrorKind::Connection("dpiConn_setClientInfo".to_string())
        )
    }

    /// Sets the current schema to be used on the connection. This has the same effect as the SQL
    /// statement ALTER SESSION SET CURRENT_SCHEMA. The value be changed when the next call
    /// requiring a round trip to the server is performed. If the new schema name does not exist,
    /// the same error is returned as when the alter session statement is executed. The new schema
    /// name is placed before database objects in statement that you execute that do not already
    /// have a schema.
    ///
    /// * `schema` - A string in the encoding used for CHAR data which will be used to set the
    /// current schema.
    pub fn set_current_schema(&self, schema: &str) -> Result<()> {
        let curr_schema_s = ODPIStr::from(schema);
        try_dpi!(
            externs::dpiConn_setCurrentSchema(self.inner, curr_schema_s.ptr(), curr_schema_s.len()),
            Ok(()),
            ErrorKind::Connection("dpiConn_setCurrentSchema".to_string())
        )
    }

    /// Sets the database operation attribute on the connection. This is one of the end-to-end
    /// tracing attributes that can be tracked in database views, shown in audit trails and seen in
    /// tools such as Enterprise Manager.
    ///
    /// * `op` - a string in the encoding used for CHAR data which will be used to set the database
    /// operation attribute.
    pub fn set_db_op(&self, op: &str) -> Result<()> {
        let db_op_s = ODPIStr::from(op);

        try_dpi!(
            externs::dpiConn_setDbOp(self.inner, db_op_s.ptr(), db_op_s.len()),
            Ok(()),
            ErrorKind::Connection("dpiConn_setDbOp".to_string())
        )
    }


    /// Sets the external name that is being used by the connection. This value is used when logging
    /// distributed transactions.
    ///
    /// * `external_name` - a string in the encoding used for CHAR data which will be used to set
    /// the external name.
    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    pub fn set_external_name(&self, external_name: &str) -> Result<()> {
        let external_name_cstr = CString::new(external_name)?;
        let external_name_len = external_name.len() + 1;

        if external_name_len <= u32::max_value() as usize {
            try_dpi!(
                externs::dpiConn_setExternalName(
                    self.inner,
                    external_name_cstr.as_ptr(),
                    external_name_len as u32
                ),
                Ok(()),
                ErrorKind::Connection("dpiConn_setExternalName".to_string())
            )
        } else {
            let err = "dpiConn_setExternalName: length out of bounds".to_string();
            Err(ErrorKind::Connection(err).into())
        }
    }

    /// Sets the internal name that is being used by the connection. This value is used when logging
    /// distributed transactions.
    ///
    /// * `internal_name` - a string in the encoding used for CHAR data which will be used to set
    /// the internal name.
    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    pub fn set_internal_name(&self, internal_name: &str) -> Result<()> {
        let internal_name_cstr = CString::new(internal_name)?;
        let internal_name_len = internal_name.len() + 1;

        if internal_name_len <= u32::max_value() as usize {
            try_dpi!(
                externs::dpiConn_setInternalName(
                    self.inner,
                    internal_name_cstr.as_ptr(),
                    internal_name_len as u32
                ),
                Ok(()),
                ErrorKind::Connection("dpiConn_setInternalName".to_string())
            )
        } else {
            let err = "dpiConn_setInternalName: length out of bounds".to_string();
            Err(ErrorKind::Connection(err).into())
        }
    }

    /// Sets the module attribute on the connection. This is one of the end-to-end tracing
    /// attributes that can be tracked in database views, shown in audit trails and seen in tools
    /// such as Enterprise Manager.
    ///
    /// * `module` - a string in the encoding used for CHAR data which will be used to set the
    /// module attribute.
    pub fn set_module(&self, module: &str) -> Result<()> {
        let module_s = ODPIStr::from(module);

        try_dpi!(
            externs::dpiConn_setModule(self.inner, module_s.ptr(), module_s.len()),
            Ok(()),
            ErrorKind::Connection("dpiConn_setModule".to_string())
        )
    }

    /// Sets the size of the statement cache.
    ///
    /// * `size` - the new size of the statement cache, in number of statements.
    pub fn set_statement_cache_size(&self, size: u32) -> Result<()> {
        try_dpi!(
            externs::dpiConn_setStmtCacheSize(self.inner, size),
            Ok(()),
            ErrorKind::Connection("dpiConn_setStmtCacheSize".to_string())
        )
    }

    /// Shuts down the database. This function must be called twice for the database to be shut down
    /// successfully. After calling this function the first time, the SQL statements "alter database
    /// close normal" and "alter database dismount" must be executed. Once that is complete this
    /// function should be called again with the mode DPI_MODE_SHUTDOWN_FINAL in order to complete
    /// the orderly shutdown of the database.
    ///
    /// * `mode` - one of the values from the enumeration `ODPIShutdownMode`.
    pub fn shutdown_database(self, mode: enums::ODPIShutdownMode) -> Result<()> {
        try_dpi!(
            externs::dpiConn_shutdownDatabase(self.inner, mode),
            Ok(()),
            ErrorKind::Connection("dpiConn_shutdownDatabase".to_string())
        )
    }

    /// Starts up a database
    ///
    /// * `mode` - one of the values from the enumeration `ODPIStartupMode`.
    pub fn start_database(self, mode: enums::ODPIStartupMode) -> Result<()> {
        try_dpi!(
            externs::dpiConn_startupDatabase(self.inner, mode),
            Ok(()),
            ErrorKind::Connection("dpiConn_startupDatabase".to_string())
        )
    }
}

impl From<*mut ODPIConn> for Connection {
    fn from(inner: *mut ODPIConn) -> Connection {
        Connection {
            inner: inner,
            stdout: None,
            stderr: None,
        }
    }
}
