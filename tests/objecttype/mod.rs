use CREDS;
use chrono::{TimeZone, Utc};
use mimir::{Connection, Context, Data, ODPIData, ODPIObjectAttrInfo, ODPIObjectTypeInfo, ODPIStr,
            Object, ObjectAttr, ObjectType, Statement};
use mimir::enums;
use mimir::error::Result;
use mimir::flags;
use std::ffi::CString;

fn validate_object_attr_info(idx: usize, attr_info: &ODPIObjectAttrInfo) -> Result<()> {
    let name_s = ODPIStr::new(attr_info.name, attr_info.name_length);
    let name: String = name_s.into();
    match idx {
        0 => assert_eq!(name, "NUMBERVALUE"),
        1 => assert_eq!(name, "STRINGVALUE"),
        2 => assert_eq!(name, "FIXEDCHARVALUE"),
        3 => assert_eq!(name, "DATEVALUE"),
        4 => assert_eq!(name, "TIMESTAMPVALUE"),
        5 => assert_eq!(name, "SUBOBJECTVALUE"),
        6 => assert_eq!(name, "SUBOBJECTARRAY"),
        _ => assert!(false),
    }
    Ok(())
}

fn validate_object_type_info(type_info: &ODPIObjectTypeInfo) -> Result<()> {
    let schema = ODPIStr::new(type_info.schema, type_info.schema_length);
    let name = ODPIStr::new(type_info.name, type_info.name_length);
    let schema_str: String = schema.into();
    let name_str: String = name.into();

    assert_eq!(schema_str, "ODPIC");
    assert_eq!(name_str, "UDT_OBJECT");
    assert_eq!(type_info.is_collection, 0);
    assert_eq!(type_info.num_attributes, 7);

    let element_type_info = type_info.element_type_info;
    // TODO: Why does this fail?
    // assert_eq!(
    //     element_type_info.oracle_type_num,
    //     enums::ODPIOracleTypeNum::Max
    // );
    assert_eq!(
        element_type_info.default_native_type_num,
        enums::ODPINativeTypeNum::Invalid
    );
    assert!(element_type_info.object_type.is_null());

    Ok(())
}

fn validate_bytes(idx: usize, attr_data: &ODPIData) -> Result<()> {
    let data_bytes = unsafe { attr_data.value.as_bytes };
    let o_str = ODPIStr::new(data_bytes.ptr, data_bytes.length);
    let data_str: String = o_str.into();
    if idx == 1 {
        assert_eq!(data_str, "First row");
    } else if idx == 2 {
        assert_eq!(data_str, "First     ");
    } else {
        assert!(false);
    }
    Ok(())
}

fn validate_double(idx: usize, attr_data: &ODPIData) -> Result<()> {
    if idx == 0 {
        unsafe { assert!(attr_data.value.as_double - 1.0 < ::std::f64::EPSILON) };
    } else {
        assert!(false);
    }
    Ok(())
}

fn validate_timestamp(idx: usize, attr_data: &ODPIData) -> Result<()> {
    let odpi_ts = unsafe { attr_data.value.as_timestamp };
    let y = i32::from(odpi_ts.year);
    let m = u32::from(odpi_ts.month);
    let d = u32::from(odpi_ts.day);
    let h = u32::from(odpi_ts.hour);
    let mi = u32::from(odpi_ts.minute);
    let se = u32::from(odpi_ts.second);
    let ts = Utc.ymd(y, m, d).and_hms_nano(h, mi, se, odpi_ts.fsecond);

    if idx == 3 {
        let expected = Utc.ymd(2007, 3, 6).and_hms_nano(0, 0, 0, 0);
        assert_eq!(ts, expected);
    } else if idx == 4 {
        let expected = Utc.ymd(2008, 9, 12).and_hms_nano(16, 40, 0, 0);
        assert_eq!(ts, expected);
    } else {
        assert!(false);
    }
    Ok(())
}

fn validate_subobject(obj_type: &ObjectType) -> Result<()> {
    let type_info = obj_type.get_info()?;
    let schema = ODPIStr::new(type_info.schema, type_info.schema_length);
    let name = ODPIStr::new(type_info.name, type_info.name_length);
    let schema_str: String = schema.into();
    let name_str: String = name.into();

    assert_eq!(schema_str, "ODPIC");
    assert_eq!(name_str, "UDT_SUBOBJECT");
    assert_eq!(type_info.is_collection, 0);
    assert_eq!(type_info.num_attributes, 2);
    Ok(())
}

fn validate_objectarr(obj_type: &ObjectType) -> Result<()> {
    let type_info = obj_type.get_info()?;
    let schema = ODPIStr::new(type_info.schema, type_info.schema_length);
    let name = ODPIStr::new(type_info.name, type_info.name_length);
    let schema_str: String = schema.into();
    let name_str: String = name.into();

    assert_eq!(schema_str, "ODPIC");
    assert_eq!(name_str, "UDT_OBJECTARRAY");
    assert_eq!(type_info.is_collection, 1);
    assert_eq!(type_info.num_attributes, 0);

    let element_type_info = type_info.element_type_info;
    assert_eq!(
        element_type_info.oracle_type_num,
        enums::ODPIOracleTypeNum::Object
    );
    assert_eq!(
        element_type_info.default_native_type_num,
        enums::ODPINativeTypeNum::Object
    );
    assert!(!element_type_info.object_type.is_null());

    let arr_obj_type: ObjectType = element_type_info.object_type.into();
    validate_subobject(&arr_obj_type)?;

    Ok(())
}

fn validate_object(idx: usize, attr_info: &ODPIObjectAttrInfo, attr_data: &ODPIData) -> Result<()> {
    let nested_obj_type_ptr = attr_info.type_info.object_type;

    if nested_obj_type_ptr.is_null() {
        assert!(false);
    } else {
        let nested_obj_type: ObjectType = nested_obj_type_ptr.into();
        let odpi_obj_ptr = unsafe { attr_data.value.as_object };
        let odpi_obj: Object = odpi_obj_ptr.into();

        if idx == 5 {
            validate_subobject(&nested_obj_type)?;
        } else if idx == 6 {
            validate_objectarr(&nested_obj_type)?;

            let (first_index, first_index_exists) = odpi_obj.get_first_index()?;
            assert_eq!(first_index, 0);
            assert!(first_index_exists);

            let (last_index, last_index_exists) = odpi_obj.get_last_index()?;
            assert_eq!(last_index, 1);
            assert!(last_index_exists);

            let (next_index, next_index_exists) = odpi_obj.get_next_index(0)?;
            assert_eq!(next_index, 1);
            assert!(next_index_exists);
            let (next_index_1, next_index_exists_1) = odpi_obj.get_next_index(1)?;
            assert_eq!(next_index_1, 0);
            assert!(!next_index_exists_1);

            let (prev_index, prev_index_exists) = odpi_obj.get_prev_index(1)?;
            assert_eq!(prev_index, 0);
            assert!(prev_index_exists);
            let (prev_index_1, prev_index_exists_1) = odpi_obj.get_prev_index(0)?;
            assert_eq!(prev_index_1, 0);
            assert!(!prev_index_exists_1);

            let mut size = odpi_obj.get_size()?;
            assert_eq!(size, 2);

            odpi_obj.trim(1)?;
            size = odpi_obj.get_size()?;
            assert_eq!(size, 1);
        }
    }

    Ok(())
}

fn validate_query_value(
    idx: usize,
    obj: &Object,
    obj_attr: &ObjectAttr,
    attr_info: &ODPIObjectAttrInfo,
) -> Result<()> {
    let attr_data = obj.get_attribute_value(obj_attr, attr_info)?;
    match attr_info.type_info.default_native_type_num {
        enums::ODPINativeTypeNum::Bytes => validate_bytes(idx, &attr_data)?,
        enums::ODPINativeTypeNum::Double => validate_double(idx, &attr_data)?,
        enums::ODPINativeTypeNum::Timestamp => validate_timestamp(idx, &attr_data)?,
        enums::ODPINativeTypeNum::Object => validate_object(idx, attr_info, &attr_data)?,
        _ => {
            assert!(false);
        }
    }
    Ok(())
}

fn validate_object_type(object_col: &Statement, object_type: &ObjectType) -> Result<()> {
    let attrs = object_type.get_attributes(7)?;
    let mut obj_attrs = Vec::new();
    let mut attr_infos = Vec::new();

    for (idx, obj_attr) in attrs.iter().enumerate() {
        let attr: ObjectAttr = (*obj_attr).into();
        let attr_info = attr.get_info()?;

        validate_object_attr_info(idx, &attr_info)?;

        obj_attrs.push(attr);
        attr_infos.push(attr_info);
    }

    let type_info = object_type.get_info()?;
    validate_object_type_info(&type_info)?;

    object_col.fetch()?;

    // Create an object of this type.
    let created_obj = object_type.create()?;
    let _created: Object = created_obj.into();
    // let (first_idx, exists) = created.get_first_index()?;
    // assert_eq!(first_idx, 0);
    // assert_eq!(exists, 1);

    // Get the object value out of the query.
    let (object_col_type, object_col_ptr) = object_col.get_query_value(1)?;
    assert_eq!(object_col_type, enums::ODPINativeTypeNum::Object);
    let data: Data = object_col_ptr.into();
    let obj: Object = data.get_object().into();

    for (idx, (obj_attr, attr_info)) in obj_attrs.iter().zip(attr_infos.iter()).enumerate() {
        validate_query_value(idx, &obj, obj_attr, attr_info)?;
    }

    for obj_attr in obj_attrs {
        obj_attr.release()?;
    }
    Ok(())
}

fn obj_type(ctxt: &Context) -> Result<()> {
    let mut ccp = ctxt.init_common_create_params()?;
    let enc_cstr = CString::new("UTF-8").expect("badness");
    ccp.set_encoding(enc_cstr.as_ptr());
    ccp.set_nchar_encoding(enc_cstr.as_ptr());

    let conn = Connection::create(
        ctxt,
        Some(&CREDS[2]),
        Some(&CREDS[3]),
        Some("//oic.cbsnae86d3iv.us-east-2.rds.amazonaws.com/ORCL"),
        Some(ccp),
        None,
    )?;

    // Query with object
    let object_col = conn.prepare_stmt(
        Some(
            "select ObjectCol \
             from TestObjects \
             order by IntCol",
        ),
        None,
        false,
    )?;

    let cols = object_col.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    assert_eq!(cols, 1);

    let query_info = object_col.get_query_info(1)?;
    let type_info = query_info.type_info();
    assert!(type_info.object_type().is_some());

    if let Some(object_type) = type_info.object_type() {
        validate_object_type(&object_col, &object_type)?;
    }

    object_col.close(None)?;
    conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;
    object_col.release()?;

    Ok(())
}

#[test]
fn objecttype() {
    check_with_ctxt!(obj_type)
}
