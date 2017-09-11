use CREDS;
use chrono::{Datelike, Timelike, Utc};
use mimir::Connection;
use mimir::Context;
use mimir::enums::ODPIMessageDeliveryMode::NotSet;
use mimir::enums::ODPIMessageState::Ready;
use mimir::error::Result;
use mimir::flags;

fn msg(ctxt: &Context) -> Result<()> {
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

    let msg_props = conn.new_msg_props()?;
    let num_attempts = msg_props.get_num_attempts()?;
    assert_eq!(num_attempts, 0);

    let mut correlation = msg_props.get_correlation()?;
    assert_eq!(correlation, "");
    msg_props.set_correlation("ABC_")?;
    correlation = msg_props.get_correlation()?;
    assert_eq!(correlation, "ABC_");

    let mut delay = msg_props.get_delay()?;
    assert_eq!(delay, 0);
    msg_props.set_delay(5000)?;
    delay = msg_props.get_delay()?;
    assert_eq!(delay, 5000);

    let delivery_mode = msg_props.get_delivery_mode()?;
    assert_eq!(delivery_mode, NotSet);
    let enq_time = msg_props.get_enq_time()?;
    let now = Utc::now();
    assert_eq!(enq_time.year(), now.year());
    assert_eq!(enq_time.month(), now.month());
    assert_eq!(enq_time.day(), now.day());
    assert_eq!(enq_time.hour(), now.hour());

    let mut exception_q = msg_props.get_exception_q()?;
    assert_eq!(exception_q, "");
    msg_props.set_exception_q("ex_q")?;
    exception_q = msg_props.get_exception_q()?;
    assert_eq!(exception_q, "ex_q");

    let mut expiration = msg_props.get_expiration()?;
    assert_eq!(expiration, -1);
    msg_props.set_expiration(360)?;
    expiration = msg_props.get_expiration()?;
    assert_eq!(expiration, 360);

    let mut orig_msg_id = msg_props.get_original_msg_id()?;
    assert_eq!(orig_msg_id, "");
    msg_props.set_original_msg_id("orig_msg_id")?;
    orig_msg_id = msg_props.get_original_msg_id()?;
    assert_eq!(orig_msg_id, "orig_msg_id");

    let mut priority = msg_props.get_priority()?;
    assert_eq!(priority, 0);
    msg_props.set_priority(-1)?;
    priority = msg_props.get_priority()?;
    assert_eq!(priority, -1);

    let state = msg_props.get_state()?;
    assert_eq!(state, Ready);

    conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;

    Ok(())
}

#[test]
fn msg_props() {
    check_with_ctxt!(msg)
}
