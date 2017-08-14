// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! ODPI-C public bitflags.
bitflags! {
    #[repr(C)]
    /// This enumeration identifies the mode to use when authorizing connections to the database.
    pub struct ODPIAuthMode: u32 {
        /// Default value used when creating connections.
        const DPI_MODE_AUTH_DEFAULT = 0b0000_0000;
        /// Authenticates with SYSDBA access.
        const DPI_MODE_AUTH_SYSDBA  = 0b0000_0010;
        /// Authenticates with SYSOPER access.
        const DPI_MODE_AUTH_SYSOPER = 0b0000_0100;
        /// Used together with DPI_MODE_AUTH_SYSDBA or DPI_MODE_AUTH_SYSOPER to authenticate for
        /// certain administrative tasks (such as starting up or shutting down the database).
        const DPI_MODE_AUTH_PRELIM  = 0b0000_1000;
        /// Authenticates with SYSASM access.
        const DPI_MODE_AUTH_SYSASM  = 0b1000_0000_0000_0000;
    }
}

bitflags! {
    #[repr(C)]
    /// This enumeration identifies the mode to use when closing connections to the database.
    pub struct ODPIConnCloseMode: u32 {
        /// Default value used when closing connections.
        const DPI_MODE_CONN_CLOSE_DEFAULT = 0b00;
        /// Causes the session to be dropped from the session pool instead of simply returned to the
        /// pool for future use.
        const DPI_MODE_CONN_CLOSE_DROP    = 0b01;
        /// Causes the session to be tagged with the tag information given when the connection is
        /// closed. A value of NULL for the tag will cause the tag to be cleared.
        const DPI_MODE_CONN_CLOSE_RETAG   = 0b10;
    }
}

bitflags! {
    #[repr(C)]
    /// This enumeration identifies the mode to use when creating connections to the database. Note
    /// that the OCI objects mode is always enabled.
    pub struct ODPICreateMode: u32 {
        /// Default value used when creating connections.
        const DPI_MODE_CREATE_DEFAULT  = 0b0000_0000;
        /// Enables threaded mode. Internal OCI structures not exposed to the user are protected
        /// from concurrent access by multiple threads. Error information is also managed in thread
        /// local storage.
        const DPI_MODE_CREATE_THREADED = 0b0000_0001;
        /// Enables events mode which is required for the use of advanced queuing (AQ) and
        /// continuous query notification (CQN).
        const DPI_MODE_CREATE_EVENTS   = 0b0000_0100;
    }
}

bitflags! {
    #[repr(C)]
    /// This enumeration identifies the mode to use when creating connections to the database. Note
    /// that the OCI objects mode is always enabled.
    pub struct ODPIExecMode: u32 {
        /// Default mode for execution. Metadata is made available after queries are executed.
        const DPI_MODE_EXEC_DEFAULT        = 0x0;
        /// Do not execute the statement but simply acquire the metadata for the query.
        const DPI_MODE_EXEC_DESCRIBE_ONLY       = 0x10;
        /// If execution completes successfully, the current active transaction is committed.
        const DPI_MODE_EXEC_COMMIT_ON_SUCCESS   = 0x20;
        /// Enable batch error mode. This permits an an array DML operation to succeed even if some
        /// of the individual operations fail. The errors can be retrieved using the function
        /// `dpiStmt_getBatchErrors()`.
        const DPI_MODE_EXEC_BATCH_ERRORS        = 0x80;
        /// Do not execute the statement but only parse it and return any parse errors.
        const DPI_MODE_EXEC_PARSE_ONLY          = 0x100;
        /// Enable getting row counts for each DML operation when performing an array DML execution.
        /// The actual row counts can be retrieved using the function `dpiStmt_getRowCounts()`.
        const DPI_MODE_EXEC_ARRAY_DML_ROWCOUNTS = 0x10_0000;
    }
}

bitflags! {
    #[repr(C)]
    /// This enumeration identifies the types of operations that can take place during object change
    /// and query change notification. It is used both as a filter when determining which operations
    /// to consider when sending notifications as well as identifying the operation that took place
    /// on a particular table or row when a notification is sent. Multiple values can be OR'ed
    /// together to specify multiple types of operations at the same time.
    pub struct ODPIOpCode: u32 {
        /// Indicates that notifications should be sent for all operations on the table or query.
        const DPI_OPCODE_ALL_OPS  = 0b0000_0000;
        /// Indicates that all rows have been changed in the table or query (or too many rows were
        /// changed or row information was not requested).
        const DPI_OPCODE_ALL_ROWS = 0b0000_0001;
        /// Indicates that an insert operation has taken place in the table or query.
        const DPI_OPCODE_INSERT   = 0b0000_0010;
        /// Indicates that an update operation has taken place in the table or query.
        const DPI_OPCODE_UPDATE   = 0b0000_0100;
        /// Indicates that a delete operation has taken place in the table or query.
        const DPI_OPCODE_DELETE   = 0b0000_1000;
        /// Indicates that the registered table or query has been altered.
        const DPI_OPCODE_ALTER    = 0b0001_0000;
        /// Indicates that the registered table or query has been dropped.
        const DPI_OPCODE_DROP     = 0b0010_0000;
        /// An unknown operation has taken place.
        const DPI_OPCODE_UNKNOWN  = 0b0100_0000;
    }
}

bitflags! {
    #[repr(C)]
    /// This enumeration identifies the mode to use when closing pools.
    pub struct ODPIPoolCloseMode: u32 {
        /// Default value used when closing pools. If there are any active sessions in the pool an
        /// error will be raised.
        const DPI_MODE_POOL_CLOSE_DEFAULT = 0b0;
        /// Causes all of the active connections in the pool to be closed before closing the pool
        /// itself.
        const DPI_MODE_POOL_CLOSE_FORCE   = 0b1;
    }
}

bitflags! {
    #[repr(C)]
    /// This enumeration identifies the quality of service flags for sending notifications to
    /// subscriptions.
    pub struct ODPISubscrQOS: u32 {
        /// No QOS
        const DPI_SUBSCR_QOS_NONE        = 0b0000_0000;
        /// Notifications are sent reliably. If the database fails, the notifications are not lost.
        /// This is not supported for nonpersistent queues or buffered messaging.
        const DPI_SUBSCR_QOS_RELIABLE    = 0b0000_0001;
        /// When the notification has been received, the subscription is removed.
        const DPI_SUBSCR_QOS_DEREG_NFY   = 0b0000_0010;
        /// Information on the rows affected by the database or query change is sent along with the
        /// notification.
        const DPI_SUBSCR_QOS_ROWIDS      = 0b0000_0100;
        /// Perform query notification instead of database change notification. Notification is done
        /// in guaranteed mode which guarantees that the query has in fact changed.
        const DPI_SUBSCR_QOS_QUERY       = 0b0000_1000;
        /// Perform query notification in best effort mode which may result in notifications being
        /// sent when the query has not in fact changed. This is needed for complex queries that
        /// cannot be registered in guaranteed mode.
        const DPI_SUBSCR_QOS_BEST_EFFORT = 0b0001_0000;
    }
}
