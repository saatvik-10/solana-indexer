pub use rusqlite::{Connection, Result as SqlResult};

pub fn init_db() -> SqlResult<Connection> {
    let conn = Connection::open("solana_indexer.db")?;

    conn.execute(
        "
        Create Table IF NOT EXISTS txn (
            monitored_address TEXT,
            sig TEXT PRIMARY KEY,
            slot INTEGER,
            block_time INTEGER,
            fee INTEGER,
            status TEXT,
            value_moved INTEGER
        )",
        [],
    )?;

    conn.execute(
        "
            Create Table IF NOT EXISTS token_transfers (
            sig TEXT,
            mint TEXT,
            from_address TEXT,
            to_address TEXT,
            amount INTEGER,
            decimals INTEGER,
            slot INTEGER,
            PRIMARY KEY (sig, mint, from_address, to_address)
            )
        ",
        [],
    )?;

    Ok(conn)
}

pub fn save_txn(
    db: &Connection,
    monitored_address: &str,
    sig: &str,
    slot: u64,
    block_time: i64,
    fee: u64,
    status: &str,
    value_moved: i64,
) -> SqlResult<()> {
    db.execute(
        "Insert OR Replace INTO txn (monitored_address, sig, slot, block_time, fee, status, value_moved) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [monitored_address, sig, &slot.to_string(), &block_time.to_string(), &fee.to_string(), status, &value_moved.to_string()],
    )?;
    Ok(())
}

pub fn save_token_trasfers(
    db: &Connection,
    sig: &str,
    mint: &str,
    from: &str,
    to: &str,
    amount: u64,
    decimals: u8,
    slot: u64,
) -> SqlResult<()> {
    db.execute(
        "
            Insert OR Replace Into token_transfers(
            sig, mint, from_address, to_address, amount, decimals, slot
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
        [
            sig,
            mint,
            from,
            to,
            &amount.to_string(),
            &decimals.to_string(),
            &slot.to_string(),
        ],
    )?;
    Ok(())
}

pub fn query_recent_txn(
    db: &Connection,
    address: &str,
    limit: usize,
) -> SqlResult<Vec<(String, u64, String, u64)>> {
    let mut statement = db.prepare(
        "
        Select sig, slot, status, fee FROM txn WHERE monitored_address = ?1 ORDER BY slot DESC LIMIT ?2
        "
    )?;

    let row = statement.query_map([address, &limit.to_string()], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;

    row.collect()
}

pub fn query_token_transfers(
    db: &Connection,
    mint: &str,
    limit: usize,
) -> SqlResult<Vec<(String, String, String, u64, u8)>> {
    let mut statement = db.prepare(
        "
            Select sig, from_address, to_address, amount, decimals FROM token_transfers WHERE mint = ?1
            ORDER BY slot DESC LIMIT ?2
        "
    )?;

    let row = statement.query_map([mint, &limit.to_string()], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    })?;

    row.collect()
}
