use mimir::enums::ODPIFetchMode::Last;
use mimir::enums::ODPINativeTypeNum::{Bytes, Double, Int64};
use mimir::enums::ODPIOracleTypeNum::{Number, Varchar};
use mimir::enums::ODPIStatementType::Insert;
use mimir::error::Result;
use mimir::flags;
use mimir::{
    Connection, Context, Data, ODPIBytes, ODPIDataBuffer, ODPIStr, QueryInfo, TypeInfo, Var,
};
use rand::{self, Rng};
use std::convert::TryFrom;
use CREDS;

fn validate_data_type_info(data_type_info: &TypeInfo) -> Result<()> {
    assert_eq!(data_type_info.oracle_type_num(), Number);
    assert_eq!(data_type_info.default_native_type_num(), Double);
    assert_eq!(data_type_info.db_size_in_bytes(), 0);
    assert_eq!(data_type_info.client_size_in_bytes(), 0);
    assert_eq!(data_type_info.size_in_chars(), 0);
    assert_eq!(data_type_info.precision(), 38);
    assert_eq!(data_type_info.scale(), 0);
    assert!(data_type_info.object_type().is_none());
    Ok(())
}

fn validate_query_info(query_info: &QueryInfo) -> Result<()> {
    assert_eq!(query_info.name(), "ID");
    assert!(!query_info.null_ok());
    validate_data_type_info(&query_info.type_info())?;
    Ok(())
}

fn bind_by_name(conn: &Connection, username_var: &Var) -> Result<()> {
    let bind_by_name = conn.prepare_stmt(
        Some("select * from username where username = :username"),
        None,
        false,
    )?;

    bind_by_name.bind_by_name(":username", username_var)?;

    let cols = bind_by_name.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    assert_eq!(cols, 2);

    let query_cols = bind_by_name.get_num_query_columns()?;
    assert_eq!(query_cols, 2);

    let query_info = bind_by_name.get_query_info(1)?;
    validate_query_info(&query_info)?;

    let query_info_un = bind_by_name.get_query_info(2)?;
    assert_eq!(query_info_un.name(), "USERNAME");
    assert!(query_info_un.null_ok());
    let data_type_info = query_info_un.type_info();

    assert_eq!(data_type_info.oracle_type_num(), Varchar);
    assert_eq!(data_type_info.default_native_type_num(), Bytes);
    assert_eq!(data_type_info.db_size_in_bytes(), 256);
    assert_eq!(data_type_info.client_size_in_bytes(), 256);
    assert_eq!(data_type_info.size_in_chars(), 256);
    assert_eq!(data_type_info.precision(), 0);
    assert_eq!(data_type_info.scale(), 0);
    assert!(data_type_info.object_type().is_none());

    bind_by_name.fetch()?;
    let (id_type, id_data) = bind_by_name.get_query_value(1)?;
    assert_eq!(id_type, Double);
    assert!((id_data.get_double() - 1.0).abs() < ::std::f64::EPSILON);
    let (un_type, un_data) = bind_by_name.get_query_value(2)?;
    assert_eq!(un_type, Bytes);
    assert_eq!(un_data.get_string(), "jozias");

    bind_by_name.close(None)?;
    Ok(())
}

fn stmt_res(ctxt: &Context) -> Result<()> {
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
    let username_var = conn.new_var(Varchar, Bytes, 1, 256, false, false)?;
    username_var.set_from_bytes(0, "jozias")?;

    // bind_by_name / execute / get_num_query_columns / get_query_info test
    bind_by_name(&conn, &username_var)?;

    // bind_by_pos / execute test
    let bind_by_pos = conn.prepare_stmt(
        Some("select * from username where username = :username"),
        None,
        false,
    )?;
    bind_by_pos.bind_by_pos(1, &username_var)?;
    let mut cols = bind_by_pos.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    assert_eq!(cols, 2);

    // bind_value_by_name / execute test
    let blah: ODPIStr = TryFrom::try_from("test")?;
    let enc = String::from("UTF-8\0");

    let odpi_bytes = ODPIBytes {
        ptr: blah.ptr() as *mut i8,
        length: blah.len(),
        encoding: enc.as_ptr() as *const ::std::os::raw::c_char,
    };

    let t_data = Data::new(
        false,
        ODPIDataBuffer {
            as_bytes: odpi_bytes,
        },
    );
    let bind_by_value_name = conn.prepare_stmt(
        Some(
            "select * from username \
             where username = :username",
        ),
        None,
        false,
    )?;
    bind_by_value_name.bind_value_by_name(":username", Bytes, &t_data)?;
    cols = bind_by_value_name.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    assert_eq!(cols, 2);

    // bind_value_by_pos / execute test
    let bind_by_value_pos = conn.prepare_stmt(
        Some(
            "select * from username \
             where username = :username",
        ),
        None,
        false,
    )?;
    let t_data_1 = Data::new(
        false,
        ODPIDataBuffer {
            as_bytes: odpi_bytes,
        },
    );
    bind_by_value_pos.bind_value_by_pos(1, Bytes, &t_data_1)?;
    cols = bind_by_value_pos.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    assert_eq!(cols, 2);

    // execute / fetch test
    let fetch = conn.prepare_stmt(
        Some("select * from username where username = :username"),
        None,
        false,
    )?;
    fetch.bind_by_pos(1, &username_var)?;
    cols = fetch.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    assert_eq!(cols, 2);
    let (found, bbp_buffer_row_index) = fetch.fetch()?;
    assert!(found);
    assert_eq!(bbp_buffer_row_index, 0);

    // execute / fetch_rows test
    let fetch_rows = conn.prepare_stmt(
        Some(
            "select * from username \
             where username = :username",
        ),
        None,
        false,
    )?;
    fetch_rows.bind_by_pos(1, &username_var)?;
    cols = fetch_rows.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    assert_eq!(cols, 2);
    let (buffer_row_index, num_rows_fetched, more_rows) = fetch_rows.fetch_rows(10)?;
    assert_eq!(buffer_row_index, 0);
    assert_eq!(num_rows_fetched, 1);
    assert!(!more_rows);

    // get_bind_count / get_bind_names / get_batch_error_count / get_info tests
    let bn = conn.prepare_stmt(
        Some("insert into username values (:id, :username)"),
        None,
        false,
    )?;
    let bind_count = bn.get_bind_count()?;
    assert_eq!(bind_count, 2);
    let names = bn.get_bind_names(2)?;
    assert_eq!(names.len(), 2);
    for (idx, name) in names.iter().enumerate() {
        match idx {
            0 => assert_eq!(name, "ID"),
            1 => assert_eq!(name, "USERNAME"),
            _ => assert!(false),
        }
    }
    let error_count = bn.get_batch_error_count()?;
    assert_eq!(error_count, 0);
    let info = bn.get_info()?;
    assert!(info.is_dml());
    assert_eq!(info.statement_type(), Insert);

    // execute /fetch_rows / get_row_count / scroll test
    let all_users = conn.prepare_stmt(Some("select * from username"), None, false)?;
    let au_cols = all_users.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    assert_eq!(au_cols, 2);
    all_users.fetch_rows(10)?;
    let row_count = all_users.get_row_count()?;
    assert!(row_count >= 2);
    all_users.scroll(Last, 0, 0)?;

    // execute_many test
    let em = conn.prepare_stmt(
        Some("insert into username values (:id, :username)"),
        None,
        false,
    )?;
    // setup the id binds.
    let id_var = conn.new_var(Number, Int64, 2, 0, false, false)?;
    let id_data = id_var.get_data()?;
    let mut rng = rand::thread_rng();
    for data in id_data.iter_mut() {
        (*data).is_null = 0;
        (*data).value.as_int_64 = rng.gen::<i64>().abs();
    }
    em.bind_by_pos(1, &id_var)?;

    // setup the username binds
    let un_var = conn.new_var(Varchar, Bytes, 2, 256, true, false)?;

    for i in 0..2 {
        un_var.set_from_bytes(i, "jozias")?;
    }
    em.bind_by_pos(2, &un_var)?;

    em.execute_many(flags::DPI_MODE_EXEC_DEFAULT, 2)?;

    bind_by_pos.close(None)?;
    bind_by_value_name.close(None)?;
    bind_by_value_pos.close(None)?;
    fetch.close(None)?;
    fetch_rows.close(None)?;
    bn.close(None)?;
    all_users.close(None)?;
    em.close(None)?;

    conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;

    Ok(())
}

#[test]
fn statement() {
    check_with_ctxt!(stmt_res)
}
