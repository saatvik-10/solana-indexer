pub use rusqlite::{Connection, Result as SqlResult};

pub fn init_db() -> SqlResult<Connection> {
    let conn = Connection::open("solana_indexer.db")?;

    conn.execute(
        "
        Create Table IF NOT EXISTS txn (
            sig TEXT PRIMARY KEY,
            slot INTEGER,
            block_time INTEGER,
            fee INTEGER,
            status TEXT,
            value_moved INTEGER
        )",
        [],
    )?;
    Ok(conn)
}

pub fn save_txn(
    db: &Connection,
    sig: &str,
    slot: u64,
    block_time: i64,
    fee: u64,
    status: &str,
    value_moved: i64,
) -> SqlResult<()> {
    db.execute(
        "Insert OR Replace INTO txn (sig, slot, block_time, fee, status, value_moved) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [sig, &slot.to_string(), &block_time.to_string(), &fee.to_string(), status, &value_moved.to_string()],
    )?;
    Ok(())
}
