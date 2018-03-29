// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! This structure represents instances of the types created by the SQL command CREATE OR REPLACE
//! TYPE and is available by handle to a calling application or driver. An object is created by
//! calling the function `ObjectType::createObject()` or by calling the function `Object::copy()`.
//! They are also created implicitly by creating a variable of the type `DPI_ORACLE_TYPE_OBJECT`.
//! Objects are destroyed when the last reference is released by calling the function
//! `Object::release()`. All of the attributes of the structure `ODPIBaseType` are included in this
//! structure in addition to the ones specific to this structure described below.
use data::Data;
use error::{ErrorKind, Result};
use objectattr::ObjectAttr;
use odpi::opaque::ODPIObject;
use odpi::{enums, externs, structs};
use std::ptr;

/// This structure represents instances of the types created by the SQL command CREATE OR REPLACE
/// TYPE
#[derive(Clone, Debug)]
pub struct Object {
    /// The ODPI-C Object pointer.
    pub inner: *mut ODPIObject,
}

impl Object {
    /// Get the `inner` value.
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIObject {
        self.inner
    }

    /// Sets the value of the element found at the specified index.
    pub fn append_element(
        &self,
        native_type: enums::ODPINativeTypeNum,
        data: &mut structs::ODPIData,
    ) -> Result<()> {
        try_dpi!(
            externs::dpiObject_appendElement(self.inner, native_type, data),
            Ok(()),
            ErrorKind::Object("dpiObject_appendElement".to_string())
        )
    }

    /// Creates an independent copy of an object and returns a reference to the newly created
    /// object. This reference should be released as soon as it is no longer needed.
    pub fn copy_object(&self) -> Result<Self> {
        let mut copied = ptr::null_mut();

        try_dpi!(
            externs::dpiObject_copy(self.inner, &mut copied),
            Ok(copied.into()),
            ErrorKind::Object("dpiObject_copy".to_string())
        )
    }

    /// Deletes an element from the collection. Note that the position ordinals of the remaining
    /// elements are not changed. The delete operation creates holes in the collection.
    pub fn delete_element_by_index(&self, index: i32) -> Result<()> {
        try_dpi!(
            externs::dpiObject_deleteElementByIndex(self.inner, index),
            Ok(()),
            ErrorKind::Object("dpiObject_deleteElementByIndex".to_string())
        )
    }

    /// Returns the value of one of the object's attributes.
    pub fn get_attribute_value(
        &self,
        attr: &ObjectAttr,
        info: &structs::ODPIObjectAttrInfo,
    ) -> Result<structs::ODPIData> {
        let mut data_blah: structs::ODPIData = Default::default();

        try_dpi!(
            externs::dpiObject_getAttributeValue(
                self.inner,
                attr.inner(),
                info.type_info.default_native_type_num,
                &mut data_blah
            ),
            Ok(data_blah),
            ErrorKind::Object("dpiObject_getAttributeValue".to_string())
        )
    }

    /// Returns whether an element exists at the specified index.
    pub fn get_element_exists_by_index(&self, index: i32) -> Result<bool> {
        let mut exists = 0;

        try_dpi!(
            externs::dpiObject_getElementExistsByIndex(self.inner, index, &mut exists),
            Ok(exists == 1),
            ErrorKind::Object("dpiObject_getElementExistsByIndex".to_string())
        )
    }

    /// Returns the value of the element found at the specified index.
    pub fn get_element_value_by_index(
        &self,
        index: i32,
        native_type: enums::ODPINativeTypeNum,
    ) -> Result<structs::ODPIData> {
        let mut value: structs::ODPIData = Default::default();

        try_dpi!(
            externs::dpiObject_getElementValueByIndex(self.inner, index, native_type, &mut value),
            Ok(value),
            ErrorKind::Object("dpiObject_getElementExistsByIndex".to_string())
        )
    }

    /// Returns the first index used in a collection.
    pub fn get_first_index(&self) -> Result<(i32, bool)> {
        let mut idx = 0;
        let mut exists = 0;

        try_dpi!(
            externs::dpiObject_getFirstIndex(self.inner, &mut idx, &mut exists),
            Ok((idx, exists == 1)),
            ErrorKind::Object("dpiObject_getFirstIndex".to_string())
        )
    }

    /// Returns the last index used in a collection.
    pub fn get_last_index(&self) -> Result<(i32, bool)> {
        let mut idx = 0;
        let mut exists = 0;

        try_dpi!(
            externs::dpiObject_getLastIndex(self.inner, &mut idx, &mut exists),
            Ok((idx, exists == 1)),
            ErrorKind::Object("dpiObject_getLastIndex".to_string())
        )
    }

    /// Returns the next index used in a collection following the specified index.
    pub fn get_next_index(&self, index: i32) -> Result<(i32, bool)> {
        let mut idx = 0;
        let mut exists = 0;

        try_dpi!(
            externs::dpiObject_getNextIndex(self.inner, index, &mut idx, &mut exists),
            Ok((idx, exists == 1)),
            ErrorKind::Object("dpiObject_getNextIndex".to_string())
        )
    }

    /// Returns the previous index used in a collection preceding the specified index.
    pub fn get_prev_index(&self, index: i32) -> Result<(i32, bool)> {
        let mut idx = 0;
        let mut exists = 0;

        try_dpi!(
            externs::dpiObject_getPrevIndex(self.inner, index, &mut idx, &mut exists),
            Ok((idx, exists == 1)),
            ErrorKind::Object("dpiObject_getPrevIndex".to_string())
        )
    }

    /// Returns the number of elements in a collection.
    pub fn get_size(&self) -> Result<i32> {
        let mut size = 0;

        try_dpi!(
            externs::dpiObject_getSize(self.inner, &mut size),
            Ok(size),
            ErrorKind::Object("dpiObject_getSize".to_string())
        )
    }

    /// Sets the value of one of the object’s attributes.
    pub fn set_attribute_value(
        &self,
        attribute: &ObjectAttr,
        native_type: enums::ODPINativeTypeNum,
        value: &Data,
    ) -> Result<()> {
        try_dpi!(
            externs::dpiObject_setAttributeValue(
                self.inner,
                attribute.inner(),
                native_type,
                value.inner()
            ),
            Ok(()),
            ErrorKind::Object("dpiObject_setAttributeValue".to_string())
        )
    }

    /// Sets the value of the element found at the specified index.
    pub fn set_element_value_by_index(
        &self,
        index: i32,
        native_type: enums::ODPINativeTypeNum,
        value: &Data,
    ) -> Result<()> {
        try_dpi!(
            externs::dpiObject_setElementValueByIndex(
                self.inner,
                index,
                native_type,
                value.inner()
            ),
            Ok(()),
            ErrorKind::Object("dpiObject_setElementValueByIndex".to_string())
        )
    }

    /// Trims a number of elements from the end of a collection.
    pub fn trim(&self, num_to_trim: u32) -> Result<()> {
        try_dpi!(
            externs::dpiObject_trim(self.inner, num_to_trim),
            Ok(()),
            ErrorKind::Object("dpiObject_trim".to_string())
        )
    }
}

impl From<*mut ODPIObject> for Object {
    fn from(inner: *mut ODPIObject) -> Self {
        Self { inner }
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                externs::dpiObject_release(self.inner);
            }
        }
    }
}
