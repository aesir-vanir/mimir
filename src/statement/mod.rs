// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Statement handles are used to represent statements of all types (queries, DML, DDL and PL/SQL).
//! They are created by calling the function `Connection::prepareStmt()` or the function
//! `Subscription::prepareStmt()`. They are also created implicitly when a variable of type
//! `DPI_ORACLE_TYPE_STMT` is created. Statement handles can be closed by calling the function
//! `close()` or by releasing the last reference to the statement by calling the function
//! `release()`.
use common::error;
use data::Data;
use error::{ErrorKind, Result};
use odpi::{enums, externs};
use odpi::flags::ODPIExecMode;
use odpi::opaque::ODPIStmt;
use odpi::structs::{ODPIQueryInfo, ODPIStmtInfo};
use query;
use std::{ptr, slice};
use std::convert::TryFrom;
use util::ODPIStr;
use variable::Var;

/// This structure represents statements of all types (queries, DML, DLL and PL/SQL) and is
/// available by handle to a calling application or driver.
pub struct Statement {
    /// The ODPI-C statement
    inner: *mut ODPIStmt,
}

impl Statement {
    /// Create a new statement from an `ODPIStmt` pointer
    #[doc(hidden)]
    pub fn new(inner: *mut ODPIStmt) -> Self {
        Self { inner: inner }
    }

    /// Get the `inner` value.
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIStmt {
        self.inner
    }

    /// Binds a variable to a named placeholder in the statement. A reference to the variable is
    /// retained by the library and is released when the statement itself is released or a new
    /// variable is bound to the same name.
    ///
    /// * `name` - a string in the encoding used for CHAR data giving the name of the placeholder
    /// which is to be bound.
    /// * `var` - a variable which is to be bound.
    pub fn bind_by_name(&self, name: &str, var: &Var) -> Result<()> {
        let name_s: ODPIStr = TryFrom::try_from(name)?;

        /// TODO: Test this when Var is complete.
        try_dpi!(
            externs::dpiStmt_bindByName(self.inner, name_s.ptr(), name_s.len(), var.inner()),
            Ok(()),
            ErrorKind::Statement("dpiStmt_bindByName".to_string())
        )
    }

    /// Binds a variable to a placeholder in the statement by position. A reference to the variable
    /// is retained by the library and is released when the statement itself is released or a new
    /// variable is bound to the same position.
    ///
    /// * `pos` - the position which is to be bound. The position of a placeholder is determined by
    /// its location in the statement. Placeholders are numbered from left to right, starting from
    /// 1, and duplicate names do not count as additional placeholders.
    /// * `var` - a variable which is to be bound.
    pub fn bind_by_pos(&self, pos: u32, var: &Var) -> Result<()> {
        try_dpi!(
            externs::dpiStmt_bindByPos(self.inner, pos, var.inner()),
            Ok(()),
            ErrorKind::Statement("dpiStmt_bindByPos".to_string())
        )
    }

    /// Binds a value to a named placeholder in the statement without the need to create a variable
    /// directly. One is created implicitly and released when the statement is released or a new
    /// value is bound to the same name.
    ///
    /// * `name` - a string in the encoding used for CHAR data giving the name of the placeholder
    /// which is to be bound.
    /// * `native_type` - the type of data that is being bound. It is expected to be one of the
    /// values from the enumeration `ODPINativeTypeNum`.
    /// * `data` - the data which is to be bound, as a pointer to a `ODPIData` structure. A variable
    /// will be created based on the type of data being bound and a reference to this variable
    /// retained. Once the statement has been executed, this new variable will be released.
    pub fn bind_value_by_name(
        &self,
        name: &str,
        native_type: enums::ODPINativeTypeNum,
        data: &Data,
    ) -> Result<()> {
        let name_s: ODPIStr = TryFrom::try_from(name)?;

        try_dpi!(
            externs::dpiStmt_bindValueByName(
                self.inner,
                name_s.ptr(),
                name_s.len(),
                native_type,
                data.inner()
            ),
            Ok(()),
            ErrorKind::Statement("dpiStmt_bindValueByName".to_string())
        )
    }

    /// Binds a value to a placeholder in the statement without the need to create a variable
    /// directly. One is created implicitly and released when the statement is released or a new
    /// value is bound to the same position.
    ///
    /// * `pos` - the position which is to be bound. The position of a placeholder is determined by
    /// its location in the statement. Placeholders are numbered from left to right, starting from
    /// 1, and duplicate names do not count as additional placeholders.
    /// * `native_type` - the type of data that is being bound. It is expected to be one of the
    /// values from the enumeration `ODPINativeTypeNum`.
    /// * `data` - the data which is to be bound, as a pointer to a `ODPIData` structure. A variable
    /// will be created based on the type of data being bound and a reference to this variable
    /// retained. Once the statement has been executed, this new variable will be released.
    pub fn bind_value_by_pos(
        &self,
        pos: u32,
        native_type: enums::ODPINativeTypeNum,
        data: &Data,
    ) -> Result<()> {
        try_dpi!(
            externs::dpiStmt_bindValueByPos(self.inner, pos, native_type, data.inner()),
            Ok(()),
            ErrorKind::Statement("dpiStmt_bindValueByPos".to_string())
        )
    }

    /// Closes the statement and makes it unusable for further work immediately, rather than when
    /// the reference count reaches zero.
    ///
    /// * `tag` - a key to associate the statement with in the statement cache, in the encoding used
    /// for CHAR data. None is also acceptable in which case the statement is not tagged. This value
    /// is ignored for statements that are acquired through bind variables (REF CURSOR) or implicit
    /// results.
    pub fn close(&self, tag: Option<&str>) -> Result<()> {
        let tag_s: ODPIStr = TryFrom::try_from(tag)?;
        try_dpi!(
            externs::dpiStmt_close(self.inner, tag_s.ptr(), tag_s.len()),
            Ok(()),
            ErrorKind::Statement("dpiStmt_close".to_string())
        )
    }

    // /// Defines the variable that will be used to fetch rows from the statement. A reference to
    //the
    // /// variable will be retained until the next define is performed on the same position or the
    // /// statement is closed.
    // pub fn define(&self, pos: u32, var: &mut Var) -> Result<()> {
    //     Ok(())
    // }

    /// Defines the type of data that will be used to fetch rows from the statement. This is
    /// intended for use with the function `Statement::get_query_value()`, when the default
    /// data type derived from the column metadata needs to be overridden by the application.
    /// Internally, a variable is created with the specified data type and size.
    pub fn define_value(
        &self,
        pos: u32,
        oracle_type: enums::ODPIOracleTypeNum,
        native_type: enums::ODPINativeTypeNum,
        size: Option<u32>,
        size_is_bytes: Option<bool>,
    ) -> Result<()> {
        let inner_size = if let Some(sz) = size { sz } else { 0 };
        let inner_size_is_bytes = if let Some(szib) = size_is_bytes {
            if szib {
                1
            } else {
                0
            }
        } else {
            0
        };
        try_dpi!(
            externs::dpiStmt_defineValue(
                self.inner,
                pos,
                oracle_type,
                native_type,
                inner_size,
                inner_size_is_bytes,
                ptr::null_mut()
            ),
            Ok(()),
            ErrorKind::Statement("dpiStmt_execute".to_string())
        )
    }

    /// Executes the statement using the bound values. For queries this makes available metadata
    /// which can be acquired using the function dpiStmt_getQueryInfo(). For non-queries, out and
    /// in-out variables are populated with their values.
    ///
    /// * `mode` - one or more of the values from the enumeration `ODPIExecMode`, OR'ed together.
    pub fn execute(&self, mode: ODPIExecMode) -> Result<u32> {
        let mut cols_queried = 0;
        try_dpi!(
            externs::dpiStmt_execute(self.inner, mode, &mut cols_queried),
            Ok(cols_queried),
            ErrorKind::Statement("dpiStmt_execute".to_string())
        )
    }

    /// Executes the statement the specified number of times using the bound values. Each bound
    /// variable must have at least this many elements allocated or an error is returned.
    ///
    /// * `mode` - one or more of the values from the enumeration `ODPIExecMode`, OR'ed together.
    /// * `num_iters` - the number of times the statement is executed. Each iteration corresponds to
    /// one of the elements of the array that was bound earlier.
    pub fn execute_many(&self, mode: ODPIExecMode, num_iters: u32) -> Result<()> {
        try_dpi!(
            externs::dpiStmt_executeMany(self.inner, mode, num_iters),
            Ok(()),
            ErrorKind::Statement("dpiStmt_executeMany".to_string())
        )
    }

    /// Fetches a single row from the statement. If the statement does not refer to a query an error
    /// is returned. All columns that have not been defined prior to this call are implicitly
    /// defined using the metadata made available when the statement was executed.
    ///
    /// Returns a tuple of (found, row_index)
    pub fn fetch(&self) -> Result<(bool, u32)> {
        let mut found = 0;
        let mut buffer_row_index = 0;

        try_dpi!(
            externs::dpiStmt_fetch(self.inner, &mut found, &mut buffer_row_index),
            Ok((found == 1, buffer_row_index)),
            ErrorKind::Statement("dpiStmt_fetch".to_string())
        )
    }

    /// Returns the number of rows that are available in the buffers defined for the query. If no
    /// rows are currently available in the buffers, an internal fetch takes place in order to
    /// populate them, if rows are available. If the statement does not refer to a query an error
    /// is returned. All columns that have not been defined prior to this call are implicitly
    /// defined using the metadata made available when the statement was executed.
    ///
    /// * `max_rows` - the maximum number of rows to fetch. If the number of rows available exceeds
    /// this value only this number will be fetched.
    ///
    /// Returns a tuple representing (row_index, num_rows_fetched, more_rows).
    pub fn fetch_rows(&self, max_rows: u32) -> Result<(u32, u32, bool)> {
        let mut buffer_row_index = 0;
        let mut num_rows_fetched = 0;
        let mut more_rows = 0;

        try_dpi!(
            externs::dpiStmt_fetchRows(
                self.inner,
                max_rows,
                &mut buffer_row_index,
                &mut num_rows_fetched,
                &mut more_rows
            ),
            Ok((buffer_row_index, num_rows_fetched, more_rows == 1)),
            ErrorKind::Statement("dpiStmt_fetchRows".to_string())
        )
    }

    /// Returns the number of batch errors that took place during the last execution with batch mode
    /// enabled. Batch errors are only available when both the client and the server are at 12.1.
    pub fn get_batch_error_count(&self) -> Result<u32> {
        let mut count = 0;

        try_dpi!(
            externs::dpiStmt_getBatchErrorCount(self.inner, &mut count),
            Ok(count),
            ErrorKind::Statement("dpiStmt_getBatchErrorCount".to_string())
        )
    }

    /// Returns the batch errors that took place during the last execution with batch mode enabled.
    /// Batch errors are only available when both the client and the server are at 12.1.
    ///
    /// * `num_errors` - the size of the errors array in number of elements. The number of batch
    /// errors that are available can be determined using `get_batch_error_count()`.
    pub fn get_batch_errors(&self, num_errors: u32) -> Result<Vec<error::Info>> {
        let err_ptr = ptr::null_mut();

        try_dpi!(
            externs::dpiStmt_getBatchErrors(self.inner, num_errors, err_ptr),
            {
                let err_slice = unsafe { slice::from_raw_parts(err_ptr, num_errors as usize) };
                let odpi_vec = Vec::from(err_slice);
                let res_vec = odpi_vec.iter().map(|x| (*x).into()).collect();
                Ok(res_vec)
            },
            ErrorKind::Statement("dpiStmt_getBatchErrors".to_string())
        )
    }

    /// Returns the number of unique bind variables in the prepared statement.
    pub fn get_bind_count(&self) -> Result<u32> {
        let mut count = 0;
        try_dpi!(
            externs::dpiStmt_getBindCount(self.inner, &mut count),
            Ok(count),
            ErrorKind::Statement("dpiStmt_getBindCount".to_string())
        )
    }

    /// Returns the names of the unique bind variables in the prepared statement.
    pub fn get_bind_names(&self, num_bind_names: u32) -> Result<Vec<String>> {
        let mut actual_num_bind_names = num_bind_names;
        let mut names_vec: Vec<*const ::std::os::raw::c_char> =
            Vec::with_capacity(num_bind_names as usize);
        let mut names_len_vec: Vec<u32> = Vec::with_capacity(num_bind_names as usize);

        for _ in 0..num_bind_names {
            names_vec.push(ptr::null());
            names_len_vec.push(0);
        }

        try_dpi!(
            externs::dpiStmt_getBindNames(
                self.inner,
                &mut actual_num_bind_names,
                names_vec.as_mut_ptr(),
                names_len_vec.as_mut_ptr()
            ),
            {
                let mut res = Vec::new();

                for (idx, (name, name_len)) in
                    names_vec.iter().zip(names_len_vec.iter()).enumerate()
                {
                    if idx <= actual_num_bind_names as usize {
                        let name_s = ODPIStr::new(*name, *name_len);
                        res.push(name_s.into());
                    }
                }

                Ok(res)
            },
            ErrorKind::Statement("dpiStmt_getBindNames".to_string())
        )
    }

    /// Gets the array size used for performing fetches.
    pub fn get_fetch_array_size(&self) -> Result<u32> {
        let mut size = 0;

        try_dpi!(
            externs::dpiStmt_getFetchArraySize(self.inner, &mut size),
            Ok(size),
            ErrorKind::Statement("dpiStmt_getFetchArraySize".to_string())
        )
    }

    /// Returns the next implicit result available from the last execution of the statement.
    /// Implicit results are only available when both the client and server are 12.1 or higher.
    pub fn get_implicit_result(&self) -> Result<()> {
        Err(ErrorKind::Statement("Not Implemented!".to_string()).into())
    }

    /// Returns information about the statement.
    pub fn get_info(&self) -> Result<self::Info> {
        let mut info: ODPIStmtInfo = Default::default();

        try_dpi!(
            externs::dpiStmt_getInfo(self.inner, &mut info),
            Ok(Info::new(info)),
            ErrorKind::Statement("dpiStmt_getInfo".to_string())
        )
    }

    /// Returns the number of columns that are being queried.
    pub fn get_num_query_columns(&self) -> Result<u32> {
        let mut cols = 0;

        try_dpi!(
            externs::dpiStmt_getNumQueryColumns(self.inner, &mut cols),
            Ok(cols),
            ErrorKind::Statement("dpiStmt_getNumQueryColumns".to_string())
        )
    }

    /// Returns information about the column that is being queried.
    pub fn get_query_info(&self, pos: u32) -> Result<query::Info> {
        let mut qi: ODPIQueryInfo = Default::default();

        try_dpi!(
            externs::dpiStmt_getQueryInfo(self.inner, pos, &mut qi),
            Ok(query::Info::new(qi)),
            ErrorKind::Statement("dpiStmt_getQueryInfo".to_string())
        )
    }

    /// Returns the value of the column at the given position for the currently fetched row, without
    /// needing to provide a variable.
    pub fn get_query_value(&self, pos: u32) -> Result<(enums::ODPINativeTypeNum, Data)> {
        let mut data = ptr::null_mut();
        let mut native_type = 0;

        try_dpi!(
            externs::dpiStmt_getQueryValue(self.inner, pos, &mut native_type, &mut data),
            Ok((native_type.into(), TryFrom::try_from(data)?)),
            ErrorKind::Statement("dpiStmt_getQueryValue".to_string())
        )
    }

    /// Returns the number of rows affected by the last DML statement that was executed or the
    /// number of rows currently fetched from a query. In all other cases 0 is returned.
    pub fn get_row_count(&self) -> Result<u64> {
        let mut count = 0;

        try_dpi!(
            externs::dpiStmt_getRowCount(self.inner, &mut count),
            Ok(count),
            ErrorKind::Statement("dpiStmt_getRowCount".to_string())
        )
    }

    /// Returns an array of row counts affected by the last invocation of `Statement::executeMany()`
    /// with the array DML rowcounts mode enabled. This feature is only available if both client and
    /// server are at 12.1.
    pub fn get_row_counts(&self) -> Result<Vec<u64>> {
        Err(ErrorKind::Statement("Not Implemented!".to_string()).into())
    }

    /// Returns the id of the query that was just registered on the subscription by calling
    /// `Statement::execute()` on a statement prepared by calling `Subscription::prepare_stmt()`.
    pub fn get_subscr_query_id(&self) -> Result<u64> {
        Err(ErrorKind::Statement("Not Implemented!".to_string()).into())
    }

    /// Scrolls the statement to the position in the cursor specified by the mode and offset.
    ///
    /// * `mode` - one of the values from the enumeration `ODPIFetchMode`.
    /// * `offset` - a value which is used with the mode in order to determine the row position in
    /// the cursor.
    /// * `row_count_offset` - an offset to the row count used when calculating the desired row to
    /// be fetched. This is needed when a client has fetched multiple rows but has not yet
    /// consumed all of them. If this is not being done by the client, the value 0 is appropriate.
    pub fn scroll(
        &self,
        mode: enums::ODPIFetchMode,
        offset: i32,
        row_count_offset: i32,
    ) -> Result<()> {
        try_dpi!(
            externs::dpiStmt_scroll(self.inner, mode, offset, row_count_offset),
            Ok(()),
            ErrorKind::Statement("dpiStmt_scroll".to_string())
        )
    }

    /// Sets the array size used for performing fetches. All variables defined for fetching must
    /// have this many (or more) elements allocated for them. The higher this value is the less
    /// network round trips are required to fetch rows from the database but more memory is also
    /// required. A value of zero will reset the array size to the default value of
    /// DPI_DEFAULT_FETCH_ARRAY_SIZE.
    pub fn set_fetch_array_size(&self, _array_size: u32) -> Result<()> {
        Err(ErrorKind::Statement("Not Implemented!".to_string()).into())
    }
}

impl From<*mut ODPIStmt> for Statement {
    fn from(inner: *mut ODPIStmt) -> Self {
        Self { inner: inner }
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                externs::dpiStmt_release(self.inner);
            }
        }
    }
}

/// This structure is used for passing information about a statement from ODPI-C. It is used by the
/// function `Statement::getInfo()`.
pub struct Info {
    /// The ODPI-C stmtinfo struct.
    inner: ODPIStmtInfo,
}

impl Info {
    /// Create a new statement from an `ODPIStmtInfo` pointer
    #[doc(hidden)]
    pub fn new(inner: ODPIStmtInfo) -> Self {
        Self { inner: inner }
    }

    /// Specifies if the statement refers to a query or not.
    pub fn is_query(&self) -> bool {
        self.inner.is_query == 1
    }

    /// Specifies if the statement refers to a PL/SQL block or not.
    pub fn is_plsql(&self) -> bool {
        self.inner.is_plsql == 1
    }

    /// Specifies if the statement refers to DDL (data definition language) such as creating a table
    /// or not.
    pub fn is_ddl(&self) -> bool {
        self.inner.is_ddl == 1
    }

    /// Specifies if the statement refers to DML (data manipulation language) such as inserting,
    /// updating and deleting or not.
    pub fn is_dml(&self) -> bool {
        self.inner.is_dml == 1
    }

    /// Specifies the type of statement that has been prepared. The is_query, is_plsql, is_ddl and
    /// is_dml are all categorizations of this value. It will be one of the values from the
    /// enumeration `ODPIStatementType`.
    pub fn statement_type(&self) -> enums::ODPIStatementType {
        self.inner.statement_type
    }

    /// Specifies if the statement has a returning clause in it or not.
    pub fn is_returning(&self) -> bool {
        self.inner.is_returning == 1
    }
}
