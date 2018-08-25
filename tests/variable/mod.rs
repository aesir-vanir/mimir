use mimir::enums::ODPINativeTypeNum::{Bytes, Int64};
use mimir::enums::ODPIOracleTypeNum::{Number, Varchar};
use mimir::flags;
use mimir::Result;
use mimir::{Connection, Context, Data, ODPIData};
use std::convert::TryFrom;
use CREDS;

fn var_res(ctxt: &Context) -> Result<()> {
    let mut ccp = ctxt.init_common_create_params()?;
    ccp.set_encoding("UTF-8")?;
    ccp.set_nchar_encoding("UTF-8")?;

    let conn = Connection::create(
        ctxt,
        Some(&CREDS[0]),
        Some(&CREDS[1]),
        Some("//oic.cbsnae86d3iv.us-east-2.rds.amazonaws.com/ORCL"),
        Some(ccp),
        None,
    )?;

    let var = conn.new_var(Number, Int64, 2, 0, false, false)?;
    let num_elements = var.get_num_elements_in_array()?;
    assert_eq!(num_elements, 2);
    let size_in_bytes = var.get_size_in_bytes()?;
    assert_eq!(size_in_bytes, 22);

    let str_test = conn.new_var(Varchar, Bytes, 2, 256, false, false)?;
    str_test.set_from_bytes(0, "jozias")?;
    let str_test_data = str_test.get_data()?;
    assert_eq!(str_test_data.len(), 2);
    for (idx, d) in str_test_data.iter_mut().enumerate() {
        let data: Data = TryFrom::try_from(d as *mut ODPIData)?;
        match idx {
            0 => assert_eq!(data.get_string(), "jozias"),
            1 => assert_eq!(data.get_string(), ""),
            _ => assert!(false),
        }
    }

    conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;

    Ok(())
}

#[test]
fn variable() {
    check_with_ctxt!(var_res)
}
