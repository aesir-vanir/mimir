use CREDS;
use mimir::flags;
use mimir::{Connection, Context, ODPISubscrMessage};
use mimir::enums::ODPIDeqMode::Remove;
use mimir::enums::ODPIMessageDeliveryMode::NotSet;
use mimir::enums::ODPINativeTypeNum::Bytes;
use mimir::enums::ODPIOracleTypeNum::{Clob, Varchar};
use mimir::enums::ODPIVisibility::OnCommit;
use mimir::error::Result;
use rand::{self, Rng};

extern "C" fn subscr_callback(
    _context: *mut ::std::os::raw::c_void,
    _message: *mut ODPISubscrMessage,
) {
    // For testing
}

fn conn(ctxt: &Context) -> Result<()> {
    let mut common_create_params = ctxt.init_common_create_params()?;
    common_create_params.set_encoding("UTF-8")?;
    common_create_params.set_nchar_encoding("UTF-8")?;
    common_create_params.set_create_mode(flags::DPI_MODE_CREATE_EVENTS);

    let mut common_connection_params = ctxt.init_conn_create_params()?;
    common_connection_params.set_auth_mode(flags::DPI_MODE_AUTH_DEFAULT);

    let conn = Connection::create(
        ctxt,
        Some(&CREDS[0]),
        Some(&CREDS[1]),
        Some("//oic.cbsnae86d3iv.us-east-2.rds.amazonaws.com/ORCL"),
        Some(common_create_params),
        None,
    )?;

    // break_execution / ping test
    conn.break_execution()?;
    conn.ping()?;

    // set_current_schema / get_current_schema test
    conn.set_current_schema("jozias")?;
    let current_schema = conn.get_current_schema()?;
    assert_eq!(current_schema, "jozias");

    let edition = conn.get_edition()?;
    assert_eq!(edition, "");

    conn.set_external_name("ext")?;
    let external_name = conn.get_external_name()?;
    assert_eq!(external_name, "ext");

    conn.set_internal_name("ext")?;
    let internal_name = conn.get_internal_name()?;
    assert_eq!(internal_name, "ext");

    let encoding_info = conn.get_encoding_info()?;
    assert_eq!(encoding_info.encoding(), "UTF-8");
    assert_eq!(encoding_info.nchar_encoding(), "UTF-8");
    assert_eq!(encoding_info.max_bytes_per_char(), 4);
    assert_eq!(encoding_info.max_bytes_per_nchar(), 4);

    conn.set_statement_cache_size(40)?;
    let statement_cache_size = conn.get_statement_cache_size()?;
    assert_eq!(statement_cache_size, 40);

    // begin_distrib_trans / get_ltxid / prepare_distrib_trans
    let mut rng = rand::thread_rng();
    conn.begin_distrib_trans(rng.gen::<i64>(), "One", "Two")?;
    let ltxid = conn.get_ltxid()?;
    assert_eq!(ltxid, "");
    let commit_needed = conn.prepare_distrib_trans()?;
    assert!(!commit_needed);

    // get_server_version
    let version_info = conn.get_server_version()?;
    assert_eq!(version_info.version(), "12.1.0.2.0");
    assert_eq!(version_info.version_num(), 1_201_000_200);
    assert_eq!(
        version_info.release(),
        "Oracle Database 12c Standard Edition Release 12.1.0.2.0 - \
         64bit Production"
    );

    // new_deq_options
    let deq_opts = conn.new_deq_options()?;
    let mode = deq_opts.get_mode()?;
    assert_eq!(mode, Remove);

    // new_enq_options
    let enq_opts = conn.new_enq_options()?;
    let visibility = enq_opts.get_visibility()?;
    assert_eq!(visibility, OnCommit);

    // new_msg_props
    let msg_props = conn.new_msg_props()?;
    let delivery_mode = msg_props.get_delivery_mode()?;
    assert_eq!(delivery_mode, NotSet);

    // new_subscr_props
    let mut subscr_create_params = ctxt.init_subscr_create_params()?;
    subscr_create_params.set_port_number(32_276);
    subscr_create_params.set_timeout(10_000);
    subscr_create_params.set_name("subscription")?;
    subscr_create_params.set_callback(Some(subscr_callback));
    subscr_create_params.set_recipient_name("yoda")?;

    // TODO: Fix this to run on VM.
    // let subscription = conn.new_subscription(subscr_create_params)?;
    // subscription.add_ref()?;
    // subscription.release()?;

    // new_temp_lob
    let clob = conn.new_temp_lob(Clob)?;
    let chunk_size = clob.get_chunk_size()?;
    assert_eq!(chunk_size, 8132);

    // new_var
    let var = conn.new_var(Varchar, Bytes, 5, 256, false, false)?;
    let sib = var.get_size_in_bytes()?;
    assert_eq!(sib, 1024);
    let num_elements_in_array = var.get_num_elements_in_array()?;
    assert_eq!(num_elements_in_array, 5);
    let data_arr = var.get_data()?;
    assert_eq!(data_arr.len(), 5);

    // prepare_stmt
    let _statement = conn.prepare_stmt(Some("select 1 from dual"), None, false)?;

    // sets
    conn.set_action("action")?;
    conn.set_client_identifier("client_identifier")?;
    conn.set_client_info("client_info")?;
    conn.set_db_op("insert")?;
    conn.set_module("module")?;

    Ok(())
}

#[test]
fn connection() {
    check_with_ctxt!(conn)
}
