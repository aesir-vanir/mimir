// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Object type handles are used to represent types such as those created by the SQL command CREATE
//! OR REPLACE TYPE. They are created using the function `Connection::get_object_type()` or
//! implicitly when fetching from a column containing objects by calling the function
//! `Statement::get_query_info()`. Object types are also retrieved when used as attributes in
//! another object by calling the function `ObjectAttribute::get_info()` or as the element type of a
//! collection by calling the function `ObjectType::get_info()`. They are destroyed when the last
//! reference is released by calling the function `ObjectType::release()`.
use error::{ErrorKind, Result};
use object::Object;
use odpi::externs;
use odpi::opaque::{ODPIObject, ODPIObjectAttr, ODPIObjectType};
use odpi::structs::ODPIObjectTypeInfo;
use std::ptr;

/// Object type handles are used to represent types such as those created by the SQL command CREATE
/// OR REPLACE TYPE.
#[derive(Debug)]
pub struct ObjectType {
    /// A pointer to the opaque `ODPIObjectType`.
    inner: *mut ODPIObjectType,
}

impl ObjectType {
    /// Get the pointer to the inner ODPI struct.
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIObjectType {
        self.inner
    }

    /// Adds a reference to the object type. This is intended for situations where a reference to
    /// the object type needs to be maintained independently of the reference returned when the
    /// object type was created.
    pub fn add_ref(&self) -> Result<()> {
        try_dpi!(
            externs::dpiObjectType_addRef(self.inner),
            Ok(()),
            ErrorKind::ObjectType("dpiObjectType_addRef".to_string())
        )
    }

    /// Creates an object of the specified type and returns a reference to it. This reference should
    ///  be released as soon as it is no longer needed.
    pub fn create(&self) -> Result<Object> {
        let mut object: *mut ODPIObject = ptr::null_mut();

        try_dpi!(
            externs::dpiObjectType_createObject(self.inner, &mut object),
            Ok(object.into()),
            ErrorKind::ObjectType("dpiObjectType_createObject".to_string())
        )
    }

    /// Returns the list of attributes that belong to the object type.
    pub fn get_attributes(&self, length: u16) -> Result<Vec<*mut ODPIObjectAttr>> {
        let mut buffer: Vec<*mut ODPIObjectAttr> = Vec::with_capacity(length as usize);
        let buf_ptr = buffer.as_mut_ptr();

        try_dpi!(
            externs::dpiObjectType_getAttributes(self.inner, length, buf_ptr),
            {
                unsafe { buffer.set_len(length as usize) };
                Ok(buffer)
            },
            ErrorKind::ObjectType("dpiObjectType_getAttributes".to_string())
        )
    }

    /// Returns information about the object type.
    pub fn get_info(&self) -> Result<ODPIObjectTypeInfo> {
        let mut object_type_info: ODPIObjectTypeInfo = Default::default();

        try_dpi!(
            externs::dpiObjectType_getInfo(self.inner, &mut object_type_info),
            Ok(object_type_info),
            ErrorKind::ObjectType("dpiObjectType_getInfo".to_string())
        )
    }

    /// Releases a reference to the object type. A count of the references to the object type is
    /// maintained and when this count reaches zero, the memory associated with the object type is
    /// freed.
    pub fn release(&self) -> Result<()> {
        try_dpi!(
            externs::dpiObjectType_release(self.inner),
            Ok(()),
            ErrorKind::ObjectType("dpiObjectType_release".to_string())
        )
    }
}

impl From<*mut ODPIObjectType> for ObjectType {
    fn from(oot: *mut ODPIObjectType) -> ObjectType {
        ObjectType { inner: oot }
    }
}
