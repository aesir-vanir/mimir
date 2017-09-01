use CREDS;
use mimir::Connection;
use mimir::Context;
use mimir::enums::ODPIMessageDeliveryMode::Buffered;
use mimir::enums::ODPIVisibility::{Immediate, OnCommit};
use mimir::error::Result;
use mimir::flags;
use std::ffi::CString;

fn enqueue_res(ctxt: &Context) -> Result<()> {
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

    let enqueue_opts = conn.new_enq_options()?;
    enqueue_opts.add_ref()?;

    enqueue_opts.set_delivery_mode(Buffered)?;

    enqueue_opts.set_transformation(Some("tsfm"))?;
    // TODO: Fix this test, doesn't seem to work.
    // let transformation = enqueue_opts.get_transformation()?;
    // assert_eq!(transformation, "tsfm");

    let mut visibility = enqueue_opts.get_visibility()?;
    assert_eq!(visibility, OnCommit);
    enqueue_opts.set_visibility(Immediate)?;
    visibility = enqueue_opts.get_visibility()?;
    assert_eq!(visibility, Immediate);

    enqueue_opts.release()?;
    conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;

    Ok(())
}

#[test]
fn enqueue() {
    check_with_ctxt!(enqueue_res)
}
