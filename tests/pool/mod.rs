use mimir::enums;
use mimir::enums::ODPINativeTypeNum::{Bytes, Double};
use mimir::error::Result;
use mimir::flags;
use mimir::{Context, Pool};
use CREDS;

fn pool_res(ctxt: &Context) -> Result<()> {
    let mut ccp = ctxt.init_common_create_params()?;
    ccp.set_encoding("UTF-8")?;
    ccp.set_nchar_encoding("UTF-8")?;

    let pool = Pool::create(
        ctxt,
        Some(&CREDS[0]),
        Some(&CREDS[1]),
        Some("//oic.cbsnae86d3iv.us-east-2.rds.amazonaws.com/ORCL"),
        Some(ccp),
        None,
    )?;

    let ei = pool.get_encoding_info()?;
    assert_eq!(ei.encoding(), "UTF-8");
    assert_eq!(ei.nchar_encoding(), "UTF-8");
    assert_eq!(ei.max_bytes_per_char(), 4);
    assert_eq!(ei.max_bytes_per_nchar(), 4);

    let mut get_mode = pool.get_get_mode()?;
    assert_eq!(get_mode, enums::ODPIPoolGetMode::NoWait);
    pool.set_get_mode(enums::ODPIPoolGetMode::ForceGet)?;
    get_mode = pool.get_get_mode()?;
    assert_eq!(get_mode, enums::ODPIPoolGetMode::ForceGet);

    let mut max_lifetime_session = pool.get_max_lifetime_session()?;
    assert_eq!(max_lifetime_session, 0);
    pool.set_max_lifetime_session(3600)?;
    max_lifetime_session = pool.get_max_lifetime_session()?;
    assert_eq!(max_lifetime_session, 3600);

    let mut stmt_cache_size = pool.get_stmt_cache_size()?;
    assert_eq!(stmt_cache_size, 20);
    pool.set_stmt_cache_size(100)?;
    stmt_cache_size = pool.get_stmt_cache_size()?;
    assert_eq!(stmt_cache_size, 100);

    let mut timeout = pool.get_timeout()?;
    assert_eq!(timeout, 0);
    pool.set_timeout(3600)?;
    timeout = pool.get_timeout()?;
    assert_eq!(timeout, 3600);

    {
        let conn = pool.acquire_connection(None, None, None)?;

        let version_info = conn.get_server_version()?;
        assert_eq!(version_info.version(), "12.1.0.2.0");
        assert_eq!(version_info.version_num(), 1_201_000_200);
        assert_eq!(
            version_info.release(),
            "Oracle Database 12c Standard Edition Release 12.1.0.2.0 - \
             64bit Production"
        );

        {
            let stmt = conn.prepare_stmt(
                Some("select * from username where username = 'jozias'"),
                None,
                false,
            )?;

            stmt.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
            stmt.fetch()?;
            let (id_type, id_data) = stmt.get_query_value(1)?;
            let (username_type, username_data) = stmt.get_query_value(2)?;

            assert_eq!(id_type, Double);
            assert!((id_data.get_double() - 1.0) < ::std::f64::EPSILON);

            assert_eq!(username_type, Bytes);
            assert_eq!(username_data.get_string(), "jozias");

            let busy_count = pool.get_busy_count()?;
            assert_eq!(busy_count, 1);

            let open_count = pool.get_open_count()?;
            assert_eq!(open_count, 1);

            let pl_sql = conn.prepare_stmt(
                Some(
                    r"
BEGIN
   EXECUTE IMMEDIATE 'DROP TABLE person';
EXCEPTION
   WHEN OTHERS THEN
      IF SQLCODE != -942 THEN
         RAISE;
      END IF;
END;
",
                ),
                None,
                false,
            )?;
            pl_sql.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
        }
        conn.close(flags::DPI_MODE_CONN_CLOSE_DEFAULT, None)?;
    }
    pool.close(flags::DPI_MODE_POOL_CLOSE_DEFAULT)?;

    Ok(())
}

#[test]
fn pool() {
    check_with_ctxt!(pool_res)
}
