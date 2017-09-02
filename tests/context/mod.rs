use mimir::{enums, flags};
use mimir::{AppContext, Context, ODPISubscrMessage};
use mimir::error::Result;
use std::ffi::CString;

extern "C" fn subscr_callback(
    _ctxt: *mut ::std::os::raw::c_void,
    _message: *mut ODPISubscrMessage,
) {
    // For testing
}

fn no_op(_ctxt: &Context) -> Result<()> {
    Ok(())
}

fn ccp(ctxt: &Context) -> Result<()> {
    let mut ccp = ctxt.init_common_create_params()?;
    let default_flags = ccp.get_create_mode();
    let new_flags = default_flags | flags::DPI_MODE_CREATE_THREADED;
    let enc_cstr = CString::new("UTF-8").expect("badness");
    let mut driver_name = String::from(env!("CARGO_PKG_NAME"));
    driver_name.push(' ');
    driver_name.push_str(env!("CARGO_PKG_VERSION"));

    ccp.set_create_mode(new_flags);
    ccp.set_edition("1.0")?;
    ccp.set_encoding(enc_cstr.as_ptr());
    ccp.set_nchar_encoding(enc_cstr.as_ptr());

    assert_eq!(
        ccp.get_create_mode(),
        flags::DPI_MODE_CREATE_THREADED | flags::DPI_MODE_CREATE_DEFAULT
    );
    assert_eq!(ccp.get_encoding(), "UTF-8");
    assert_eq!(ccp.get_nchar_encoding(), "UTF-8");
    assert_eq!(ccp.get_edition(), "1.0");
    assert_eq!(ccp.get_driver_name(), driver_name);
    Ok(())
}

fn conn_cp(ctxt: &Context) -> Result<()> {
    let mut conn = ctxt.init_conn_create_params()?;
    let auth_default_flags = conn.get_auth_mode();
    let auth_new_flags = auth_default_flags | flags::DPI_MODE_AUTH_SYSDBA;
    let purity_default_flags = conn.get_purity();
    let app_ctxt = AppContext::new("ns", "name", "value")?;
    let app_ctxt_1 = AppContext::new("ns", "name1", "value1")?;
    let mut app_ctxt_vec = Vec::new();
    app_ctxt_vec.push(app_ctxt);
    app_ctxt_vec.push(app_ctxt_1);

    assert_eq!(purity_default_flags, enums::ODPIPurity::DefaultPurity);

    conn.set_auth_mode(auth_new_flags);
    conn.set_connection_class("conn_class")?;
    conn.set_purity(enums::ODPIPurity::New);
    conn.set_new_password("password")?;
    conn.set_app_context(app_ctxt_vec)?;
    conn.set_external_auth(1);
    conn.set_tag("you're it")?;
    conn.set_match_any_tag(true);

    let new_app_ctxt_vec = conn.get_app_context()?;

    assert_eq!(
        conn.get_auth_mode(),
        flags::DPI_MODE_AUTH_SYSDBA | flags::DPI_MODE_AUTH_DEFAULT
    );
    assert_eq!(conn.get_connection_class(), "conn_class");
    assert_eq!(conn.get_purity(), enums::ODPIPurity::New);
    assert_eq!(conn.get_new_password(), "password");
    assert_eq!(conn.get_num_app_context(), 2);
    assert_eq!(new_app_ctxt_vec.len(), 2);

    for (idx, ac) in new_app_ctxt_vec.iter().enumerate() {
        assert_eq!(ac.get_namespace_name(), "ns");
        match idx {
            0 => {
                assert_eq!(ac.get_name(), "name");
                assert_eq!(ac.get_value(), "value");
            }
            1 => {
                assert_eq!(ac.get_name(), "name1");
                assert_eq!(ac.get_value(), "value1");
            }
            _ => assert!(false),
        }
    }

    assert_eq!(conn.get_external_auth(), 1);
    assert_eq!(conn.get_tag(), "you're it");
    assert!(conn.get_match_any_tag());
    assert_eq!(conn.get_out_tag(), "");
    assert!(!conn.get_out_tag_found());

    Ok(())
}

fn pcp(ctxt: &Context) -> Result<()> {
    let mut pcp = ctxt.init_pool_create_params()?;
    assert_eq!(pcp.get_min_sessions(), 1);
    assert_eq!(pcp.get_max_sessions(), 1);
    assert_eq!(pcp.get_session_increment(), 0);
    assert_eq!(pcp.get_ping_interval(), 60);
    assert_eq!(pcp.get_ping_timeout(), 5000);
    assert!(pcp.get_homogeneous());
    assert!(!pcp.get_external_auth());
    assert_eq!(pcp.get_get_mode(), enums::ODPIPoolGetMode::NoWait);
    assert_eq!(pcp.get_out_pool_name(), "");

    pcp.set_min_sessions(10);
    pcp.set_max_sessions(100);
    pcp.set_session_increment(5);
    pcp.set_ping_interval(-1);
    pcp.set_ping_timeout(1000);
    pcp.set_homogeneous(false);
    pcp.set_external_auth(true);
    pcp.set_get_mode(enums::ODPIPoolGetMode::ForceGet);

    assert_eq!(pcp.get_min_sessions(), 10);
    assert_eq!(pcp.get_max_sessions(), 100);
    assert_eq!(pcp.get_session_increment(), 5);
    assert_eq!(pcp.get_ping_interval(), -1);
    assert_eq!(pcp.get_ping_timeout(), 1000);
    assert!(!pcp.get_homogeneous());
    assert!(pcp.get_external_auth());
    assert_eq!(pcp.get_get_mode(), enums::ODPIPoolGetMode::ForceGet);

    Ok(())
}

#[cfg_attr(feature = "cargo-clippy", allow(should_assert_eq))]
fn scp(ctxt: &Context) -> Result<()> {
    let mut scp = ctxt.init_subscr_create_params()?;
    assert_eq!(
        scp.get_subscr_namespace(),
        enums::ODPISubscrNamespace::DbChange
    );
    assert_eq!(scp.get_protocol(), enums::ODPISubscrProtocol::Callback);
    assert_eq!(scp.get_qos(), flags::DPI_SUBSCR_QOS_NONE);
    assert_eq!(scp.get_operations(), flags::DPI_OPCODE_ALL_OPS);
    assert_eq!(scp.get_port_number(), 0);
    assert_eq!(scp.get_timeout(), 0);
    assert_eq!(scp.get_name(), "");
    assert_eq!(scp.get_callback(), None);
    // TODO: test callback_context
    assert_eq!(scp.get_recipient_name(), "");

    scp.set_protocol(enums::ODPISubscrProtocol::HTTP);
    scp.set_qos(
        flags::DPI_SUBSCR_QOS_BEST_EFFORT | flags::DPI_SUBSCR_QOS_ROWIDS,
    );
    scp.set_operations(flags::DPI_OPCODE_ALTER | flags::DPI_OPCODE_DROP);
    scp.set_port_number(32_276);
    scp.set_timeout(10_000);
    scp.set_name("subscription")?;
    scp.set_callback(Some(subscr_callback));
    scp.set_recipient_name("yoda")?;

    assert_eq!(scp.get_protocol(), enums::ODPISubscrProtocol::HTTP);
    assert_eq!(
        scp.get_qos(),
        flags::DPI_SUBSCR_QOS_BEST_EFFORT | flags::DPI_SUBSCR_QOS_ROWIDS
    );
    assert_eq!(
        scp.get_operations(),
        flags::DPI_OPCODE_ALTER | flags::DPI_OPCODE_DROP
    );
    assert_eq!(scp.get_port_number(), 32_276);
    assert_eq!(scp.get_timeout(), 10_000);
    assert_eq!(scp.get_name(), "subscription");
    assert_eq!(scp.get_recipient_name(), "yoda");
    assert!(scp.get_callback() == Some(subscr_callback));

    Ok(())
}

#[test]
fn context() {
    check_with_ctxt!(no_op)
}

#[test]
fn common_create_params() {
    check_with_ctxt!(ccp)
}

#[test]
fn connection_create_params() {
    check_with_ctxt!(conn_cp)
}

#[test]
fn pool_create_params() {
    check_with_ctxt!(pcp)
}

#[test]
fn subscription_create_params() {
    check_with_ctxt!(scp)
}
