use rusqlite::Connection;
use std::path::Path;

pub mod migrate;

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate::run(&conn)?;
    Ok(conn)
}
