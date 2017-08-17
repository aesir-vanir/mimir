// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! All of these functions are used for getting and setting the various members of the dpiData
//! structure. The members of the structure can be manipulated directly but some languages
//! (such as Go) do not have the ability to manipulate structures containing unions or the ability
//! to process macros. For this reason, none of these functions perform any error checking. They are
//! assumed to be replacements for direct manipulation of the various members of the structure.
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use error::{ErrorKind, Result};
use hex_slice::AsHex;
use objecttype::ObjectType;
use odpi::{enums, opaque};
use odpi::enums::ODPIOracleTypeNum;
use odpi::structs::{ODPIData, ODPIDataTypeInfo, ODPIDataValueUnion};
use std::slice;
use util::ODPIStr;

/// This structure is used for holding Oracle year to month interval data information.
#[derive(Default, Getters, Setters)]
pub struct YearsMonths {
    /// The years in an Oracle YEARS TO MONTHS interval.
    #[get = "pub"]
    #[set = "pub"]
    years: i32,
    /// The months in an Oracle YEARS TO MONTHS interval.
    #[get = "pub"]
    #[set = "pub"]
    months: i32,
}

/// This structure is used for passing data to and from the database for variables and for
/// manipulating object attributes and collection values.
#[derive(Debug)]
pub struct Data {
    /// The ODPI-C data pointer.
    inner: *mut ODPIData,
}

impl Data {
    /// Create a new `Data` struct;
    #[doc(hidden)]
    pub fn new(is_null: bool, val: ODPIDataValueUnion) -> Data {
        let mut odpi_data = ODPIData {
            is_null: if is_null { 1 } else { 0 },
            value: val,
        };
        Data {
            inner: &mut odpi_data,
        }
    }

    /// Get the `inner` value.
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIData {
        self.inner
    }

    /// Is the data null?
    pub fn null(&self) -> bool {
        unsafe { (*self.inner).is_null == 1 }
    }

    /// Get the value as a Vector of byes when the native type is DPI_NATIVE_TYPE_BYTES.
    pub fn get_bytes(&self) -> Vec<u8> {
        unsafe {
            let bytes = (*self.inner).value.as_bytes;
            let slice = slice::from_raw_parts(bytes.ptr as *mut u8, bytes.length as usize);
            slice.into()
        }
    }

    /// Get the value as a boolean when the native type is DPI_NATIVE_TYPE_BOOLEAN.
    pub fn get_boolean(&self) -> bool {
        unsafe { (*self.inner).value.as_boolean == 1 }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_BOOLEAN.
    pub fn set_boolean(&self, val: bool) {
        unsafe { (*self.inner).value.as_boolean = if val { 1 } else { 0 } }
    }

    /// Get the value as a `f64` when the native type is DPI_NATIVE_TYPE_DOUBLE.
    pub fn get_double(&self) -> f64 {
        unsafe { (*self.inner).value.as_double }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_DOUBLE.
    pub fn set_double(&self, val: f64) {
        unsafe { (*self.inner).value.as_double = val }
    }

    /// Get the value as a `Duration` when the native type is DPI_NATIVE_TYPE_INTERVAL_DS.
    pub fn get_duration(&self) -> Duration {
        let odpi_int_ds = unsafe { (*self.inner).value.as_interval_ds };
        let mut dur = Duration::days(odpi_int_ds.days as i64);
        dur = dur + Duration::hours(odpi_int_ds.hours as i64);
        dur = dur + Duration::minutes(odpi_int_ds.minutes as i64);
        dur = dur + Duration::seconds(odpi_int_ds.seconds as i64);
        dur = dur + Duration::nanoseconds(odpi_int_ds.fseconds as i64);
        dur
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_INTERVAL_DS.
    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    pub fn set_duration(&self, val: Duration) {
        let mut odpi_int_ds = unsafe { (*self.inner).value.as_interval_ds };
        odpi_int_ds.days = val.num_days() as i32;
        odpi_int_ds.hours = val.num_hours() as i32;
        odpi_int_ds.minutes = val.num_minutes() as i32;
        odpi_int_ds.seconds = val.num_seconds() as i32;
        odpi_int_ds.fseconds = if let Some(ns) = val.num_nanoseconds() {
            ns as i32
        } else {
            0
        };
    }

    /// Get the value as a `f32` when the native type is DPI_NATIVE_TYPE_FLOAT.
    pub fn get_float(&self) -> f32 {
        unsafe { (*self.inner).value.as_float }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_FLOAT.
    pub fn set_float(&self, val: f32) {
        unsafe { (*self.inner).value.as_float = val }
    }

    /// Get the value as an `i64` when the native type is DPI_NATIVE_TYPE_INT64.
    pub fn get_int64(&self) -> i64 {
        unsafe { (*self.inner).value.as_int_64 }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_INT64.
    pub fn set_int64(&self, val: i64) {
        unsafe { (*self.inner).value.as_int_64 = val }
    }

    /// Returns the value of the data when the native type is DPI_NATIVE_TYPE_LOB.
    pub fn get_lob(&self) -> *mut opaque::ODPILob {
        unsafe { (*self.inner).value.as_lob }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_LOB.
    pub fn set_lob(&self, val: *mut opaque::ODPILob) {
        unsafe { (*self.inner).value.as_lob = val }
    }

    /// Returns the value of the data when the native type is DPI_NATIVE_TYPE_OBJECT.
    pub fn get_object(&self) -> *mut opaque::ODPIObject {
        unsafe { (*self.inner).value.as_object }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_OBJECT.
    pub fn set_object(&self, val: *mut opaque::ODPIObject) {
        unsafe { (*self.inner).value.as_object = val }
    }

    /// Returns the value of the data when the native type is DPI_NATIVE_TYPE_STMT.
    pub fn get_stmt(&self) -> *mut opaque::ODPIStmt {
        unsafe { (*self.inner).value.as_stmt }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_STMT.
    pub fn set_stmt(&self, val: *mut opaque::ODPIStmt) {
        unsafe { (*self.inner).value.as_stmt = val }
    }

    /// Get the value as a `String` when the native type is DPI_NATIVE_TYPE_BYTES.
    pub fn get_string(&self) -> String {
        unsafe {
            let odpi_bytes = (*self.inner).value.as_bytes;
            let odpi_s = ODPIStr::new(odpi_bytes.ptr, odpi_bytes.length);
            odpi_s.into()
        }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_BYTES.
    pub fn set_string(&self, val: &str) {
        let val_s = ODPIStr::from(val);
        let mut bytes = unsafe { (*self.inner).value.as_bytes };
        bytes.ptr = val_s.ptr() as *mut i8;
        bytes.length = val_s.len();
    }

    /// Get the value as a `u64` when the native type is DPI_NATIVE_TYPE_UINT64.
    pub fn get_uint64(&self) -> u64 {
        unsafe { (*self.inner).value.as_uint_64 }
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_UINT64.
    pub fn set_uint64(&self, val: u64) {
        unsafe { (*self.inner).value.as_uint_64 = val }
    }

    /// Get the value as a `Utc` when the native type is DPI_NATIVE_TYPE_TIMESTAMP.
    pub fn get_utc(&self) -> DateTime<Utc> {
        let odpi_ts = unsafe { (*self.inner).value.as_timestamp };
        let y = odpi_ts.year as i32;
        let m = odpi_ts.month as u32;
        let d = odpi_ts.day as u32;
        let h = odpi_ts.hour as u32;
        let mi = odpi_ts.minute as u32;
        let s = odpi_ts.second as u32;
        Utc.ymd(y, m, d).and_hms_nano(h, mi, s, odpi_ts.fsecond)
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_TIMESTAMP.
    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    pub fn set_utc(&self, val: DateTime<Utc>) {
        let mut odpi_ts = unsafe { (*self.inner).value.as_timestamp };
        odpi_ts.year = val.year() as i16;
        odpi_ts.month = val.month() as u8;
        odpi_ts.day = val.day() as u8;
        odpi_ts.hour = val.hour() as u8;
        odpi_ts.minute = val.minute() as u8;
        odpi_ts.second = val.second() as u8;
    }

    /// Get the value as a `YearsMonths` when the native type is DPI_NATIVE_TYPE_INTERVAL_YM.
    pub fn get_years_months(&self) -> YearsMonths {
        let odpi_int_ym = unsafe { (*self.inner).value.as_interval_ym };
        let mut ym: YearsMonths = Default::default();
        ym.set_years(odpi_int_ym.years);
        ym.set_months(odpi_int_ym.months);
        ym
    }

    /// Sets the value of the data when the native type is DPI_NATIVE_TYPE_INTERVAL_YM.
    pub fn set_years_months(&self, val: YearsMonths) {
        let mut odpi_int_ym = unsafe { (*self.inner).value.as_interval_ym };
        odpi_int_ym.years = *val.years();
        odpi_int_ym.months = *val.months();
    }

    /// Convert `Data` to a `String` given the Oracle Data Type.
    pub fn to_string(&self, type_info: &TypeInfo) -> Result<String> {
        use std::io::{self, Write};
        if self.null() {
            Ok("(null)".to_string())
        } else {
            let oracle_type = type_info.oracle_type_num();
            let native_type = type_info.default_native_type_num();
            writeln!(io::stdout(), "{} {}", oracle_type, native_type)?;
            let res = match oracle_type {
                ODPIOracleTypeNum::Char | ODPIOracleTypeNum::Varchar => self.get_string(),
                ODPIOracleTypeNum::Date => self.get_utc().to_rfc3339(),
                ODPIOracleTypeNum::Number => self.get_double().to_string(),
                ODPIOracleTypeNum::Raw => format!("{:x}", self.get_bytes().as_hex()),
                _ => return Err(ErrorKind::Length.into()),
            };
            Ok(res)
        }
    }

    /// Get the data length (after conversion to a `String`)
    pub fn len(&self, type_info: &TypeInfo) -> Result<usize> {
        Ok(self.to_string(type_info)?.len())
    }
}

impl From<*mut ODPIData> for Data {
    fn from(inner: *mut ODPIData) -> Data {
        Data { inner: inner }
    }
}

/// Wrapper for `ODPIDataTypeInfo` struct.
#[derive(Clone, Copy, Debug, Default)]
pub struct TypeInfo {
    /// The ODPI-C data type info struct.
    inner: ODPIDataTypeInfo,
}

impl TypeInfo {
    /// Create a new `TypeInfo` struct.
    pub fn new(inner: ODPIDataTypeInfo) -> TypeInfo {
        TypeInfo { inner: inner }
    }

    /// Get the `oracle_type_num` value.
    ///
    /// Specifies the type of the column that is being queried. It will be one of the values from
    /// the enumeration `ODPIOracleTypeNum`.
    pub fn oracle_type_num(&self) -> enums::ODPIOracleTypeNum {
        self.inner.oracle_type_num
    }

    /// Get the `default_native_type_num` value.
    ///
    /// Specifies the default native type for the column that is being queried. It will be one of
    /// the values from the enumeration `ODPINativeTypeNum`.
    pub fn default_native_type_num(&self) -> enums::ODPINativeTypeNum {
        self.inner.default_native_type_num
    }

    /// Get the `db_size_in_bytes` value.
    ///
    /// Specifies the size in bytes (from the database's perspective) of the column that is being
    /// queried. This value is only populated for strings and binary columns. For all other columns
    /// the value is zero.
    pub fn db_size_in_bytes(&self) -> u32 {
        self.inner.db_size_in_bytes
    }

    /// Get the `client_size_in_bytes` value.
    ///
    /// Specifies the size in bytes (from the client's perspective) of the column that is being
    /// queried. This value is only populated for strings and binary columns. For all other columns
    /// the value is zero.
    pub fn client_size_in_bytes(&self) -> u32 {
        self.inner.client_size_in_bytes
    }

    /// Get the `size_in_chars` value.
    ///
    /// Specifies the size in characters of the column that is being queried. This value is only
    /// populated for string columns. For all other columns the value is zero.
    pub fn size_in_chars(&self) -> u32 {
        self.inner.size_in_chars
    }

    /// Get the `precision` value.
    ///
    /// Specifies the precision of the column that is being queried. This value is only populated
    /// for numeric and timestamp columns. For all other columns the value is zero.
    pub fn precision(&self) -> i16 {
        self.inner.precision
    }

    /// Get the `scale` value.
    ///
    /// Specifies the scale of the column that is being queried. This value is only populated for
    /// numeric columns. For all other columns the value is zero.
    pub fn scale(&self) -> i8 {
        self.inner.scale
    }

    /// Get the `object_type` value.
    ///
    /// Specifies a reference to the type of the object that is being queried. This value is only
    /// populated for named type columns. For all other columns the value is None.
    pub fn object_type(&self) -> Option<ObjectType> {
        if self.inner.object_type.is_null() {
            None
        } else {
            Some(self.inner.object_type.into())
        }
    }
}

impl From<ODPIDataTypeInfo> for TypeInfo {
    fn from(inner: ODPIDataTypeInfo) -> TypeInfo {
        TypeInfo { inner: inner }
    }
}
