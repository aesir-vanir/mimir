use CREDS;
use mimir::Connection;
use mimir::Context;
use mimir::enums::ODPIOracleTypeNum::Blob;
use mimir::error::Result;
use mimir::flags;
use std::ffi::CString;

fn lob_res(ctxt: &Context) -> Result<()> {
    let mut ccp = ctxt.init_common_create_params()?;
    let enc_cstr = CString::new("UTF-8").expect("badness");
    ccp.set_encoding(enc_cstr.as_ptr());
    ccp.set_nchar_encoding(enc_cstr.as_ptr());

    let conn = Connection::create(
        ctxt,
        Some(&CREDS[0]),
        Some(&CREDS[1]),
        Some("//oic.cbsnae86d3iv.us-east-2.rds.amazonaws.com/ORCL"),
        Some(ccp),
        None,
    )?;

    let temp_lob = conn.new_temp_lob(Blob)?;

    let size_in_bytes = temp_lob.get_buffer_size(1024)?;
    assert_eq!(size_in_bytes, 1024);
    let chunk_size = temp_lob.get_chunk_size()?;
    assert_eq!(chunk_size, 8132);

    temp_lob.open_resource()?;
    let is_open = temp_lob.get_is_resource_open()?;
    assert!(is_open);

    let mut buffer: Vec<i8> = ::std::iter::repeat(0).take(8132).collect();
    temp_lob.write_bytes(&buffer, 1)?;

    let size = temp_lob.get_size()?;
    assert_eq!(size, 8132);

    buffer.clear();
    buffer = ::std::iter::repeat(1).take(8132).collect();
    temp_lob.write_bytes(&buffer, 8133)?;

    let size_after_2 = temp_lob.get_size()?;
    assert_eq!(size_after_2, 16_264);

    let outbuf = temp_lob.read_bytes(8132, 2)?;
    assert_eq!(outbuf, [0, 1]);

    temp_lob.close_resource()?;
    let is_open_after_close = temp_lob.get_is_resource_open()?;
    assert!(!is_open_after_close);

    temp_lob.release()?;
    conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;
    conn.release()?;

    Ok(())
}

#[test]
fn lob() {
    check_with_ctxt!(lob_res)
}
