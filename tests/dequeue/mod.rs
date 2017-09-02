use CREDS;
use mimir::Connection;
use mimir::Context;
use mimir::error::Result;
use mimir::enums::ODPIDeqMode::{Browse, Remove};
use mimir::enums::ODPIDeqNavigation::{FirstMsg, NextMsg};
use mimir::enums::ODPIVisibility::{Immediate, OnCommit};
use mimir::flags;
use std::ffi::CString;

fn dequeue_res(ctxt: &Context) -> Result<()> {
    let mut ccp = ctxt.init_common_create_params()?;
    let enc_cstr = CString::new("UTF-8").expect("badness");
    ccp.set_encoding(enc_cstr.as_ptr());
    ccp.set_nchar_encoding(enc_cstr.as_ptr());
    ccp.set_create_mode(flags::DPI_MODE_CREATE_EVENTS);

    let conn = Connection::create(
        ctxt,
        Some(&CREDS[0]),
        Some(&CREDS[1]),
        Some("//oic.cbsnae86d3iv.us-east-2.rds.amazonaws.com/ORCL"),
        Some(ccp),
        None,
    )?;

    let dequeue_opts = conn.new_deq_options()?;

    dequeue_opts.set_consumer_name(Some("jozias"))?;
    let consumer_name = dequeue_opts.get_consumer_name()?;
    assert_eq!(consumer_name, "jozias");

    dequeue_opts.set_correlation(Some("joz%"))?;
    let correlation = dequeue_opts.get_correlation()?;
    assert_eq!(correlation, "joz%");

    dequeue_opts.set_msg_id(Some("uno"))?;
    // TODO: Fix get_msg_id (causes SIGSEV)
    // let msg_id = dequeue_opts.get_msg_id()?;
    // assert_eq!(msg_id, "uno");

    dequeue_opts.set_wait(100_000)?;
    let wait = dequeue_opts.get_wait()?;
    assert_eq!(wait, 100_000);

    dequeue_opts.set_transformation(Some("tsfm"))?;
    let transformation = dequeue_opts.get_transformation()?;
    assert_eq!(transformation, "tsfm");

    let mut visibility = dequeue_opts.get_visibility()?;
    assert_eq!(visibility, OnCommit);
    dequeue_opts.set_visibility(Immediate)?;
    visibility = dequeue_opts.get_visibility()?;
    assert_eq!(visibility, Immediate);

    let mut mode = dequeue_opts.get_mode()?;
    assert_eq!(mode, Remove);
    dequeue_opts.set_mode(Browse)?;
    mode = dequeue_opts.get_mode()?;
    assert_eq!(mode, Browse);

    let mut nav = dequeue_opts.get_navigation()?;
    assert_eq!(nav, NextMsg);
    dequeue_opts.set_navigation(FirstMsg)?;
    nav = dequeue_opts.get_navigation()?;
    assert_eq!(nav, FirstMsg);

    conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;

    Ok(())
}

#[test]
fn dequeue() {
    check_with_ctxt!(dequeue_res)
}
