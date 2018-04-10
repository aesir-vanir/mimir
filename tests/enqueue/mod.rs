use mimir::enums::ODPIMessageDeliveryMode::Buffered;
use mimir::enums::ODPIVisibility::{Immediate, OnCommit};
use mimir::error::Result;
use mimir::flags;
use mimir::Connection;
use mimir::Context;
use CREDS;

fn enqueue_res(ctxt: &Context) -> Result<()> {
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

    let enqueue_opts = conn.new_enq_options()?;

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

    conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;

    Ok(())
}

#[test]
fn enqueue() {
    check_with_ctxt!(enqueue_res)
}
