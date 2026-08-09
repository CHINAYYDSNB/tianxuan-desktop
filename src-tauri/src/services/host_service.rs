use rusqlite::{params, Connection};
use serde_json;

use crate::models::{AuthType, Host};

fn auth_type_to_str(auth: &AuthType) -> &'static str {
    match auth {
        AuthType::Key => "key",
        AuthType::Password => "password",
    }
}

pub fn insert(conn: &Connection, host: &Host) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO hosts (id, name, address, port, username, auth_type, auth_ref, group_name, tags, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            host.id,
            host.name,
            host.address,
            host.port,
            host.username,
            auth_type_to_str(&host.auth_type),
            host.auth_ref,
            host.group_name,
            serde_json::to_string(&host.tags).unwrap_or_else(|_| "[]".to_string()),
            host.created_at,
            host.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, host: &Host) -> rusqlite::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE hosts SET name=?1, address=?2, port=?3, username=?4, auth_type=?5, auth_ref=?6, group_name=?7, tags=?8, updated_at=?9 WHERE id=?10",
        params![
            host.name,
            host.address,
            host.port,
            host.username,
            auth_type_to_str(&host.auth_type),
            host.auth_ref,
            host.group_name,
            serde_json::to_string(&host.tags).unwrap_or_else(|_| "[]".to_string()),
            now,
            host.id,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM hosts WHERE id=?1", params![id])
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Host>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, address, port, username, auth_type, auth_ref, group_name, tags, created_at, updated_at FROM hosts ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], row_to_host)?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Host>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, address, port, username, auth_type, auth_ref, group_name, tags, created_at, updated_at FROM hosts WHERE id=?1",
    )?;
    let mut rows = stmt.query_map(params![id], row_to_host)?;
    match rows.next() {
        Some(Ok(host)) => Ok(Some(host)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

fn row_to_host(row: &rusqlite::Row) -> rusqlite::Result<Host> {
    let auth_type: String = row.get(5)?;
    let tags_json: String = row.get(8)?;
    let auth_type = match auth_type.as_str() {
        "key" => AuthType::Key,
        _ => AuthType::Password,
    };
    let tags: Vec<String> =
        serde_json::from_str(&tags_json).unwrap_or_default();
    Ok(Host {
        id: row.get(0)?,
        name: row.get(1)?,
        address: row.get(2)?,
        port: row.get(3)?,
        username: row.get(4)?,
        auth_type,
        auth_ref: row.get(6)?,
        group_name: row.get(7)?,
        tags,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrate::run(&conn).unwrap();
        conn
    }

    fn sample_host() -> Host {
        Host::new(
            "Test Server".to_string(),
            "47.100.33.169".to_string(),
            22,
            "root".to_string(),
            AuthType::Password,
            "keyring-ref-1".to_string(),
            "生产".to_string(),
            vec!["bt".to_string()],
        )
    }

    #[test]
    fn test_crud_roundtrip() {
        let conn = test_conn();
        let host = sample_host();
        insert(&conn, &host).unwrap();

        let fetched = get(&conn, &host.id).unwrap().unwrap();
        assert_eq!(fetched.id, host.id);
        assert_eq!(fetched.name, "Test Server");
        assert_eq!(fetched.address, "47.100.33.169");
        assert_eq!(fetched.port, 22);
        assert_eq!(fetched.username, "root");
        assert_eq!(fetched.auth_type, AuthType::Password);
        assert_eq!(fetched.auth_ref, "keyring-ref-1");
        assert_eq!(fetched.group_name, "生产");
        assert_eq!(fetched.tags, vec!["bt"]);

        let all = list(&conn).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_update() {
        let conn = test_conn();
        let mut host = sample_host();
        insert(&conn, &host).unwrap();

        host.name = "Renamed".to_string();
        host.port = 2222;
        update(&conn, &host).unwrap();

        let fetched = get(&conn, &host.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Renamed");
        assert_eq!(fetched.port, 2222);
    }

    #[test]
    fn test_delete() {
        let conn = test_conn();
        let host = sample_host();
        insert(&conn, &host).unwrap();
        assert_eq!(list(&conn).unwrap().len(), 1);

        let removed = delete(&conn, &host.id).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(list(&conn).unwrap().len(), 0);
        assert!(get(&conn, &host.id).unwrap().is_none());
    }

    #[test]
    fn test_db_open_migrates() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = db::open(&db_path).unwrap();
        let all = list(&conn).unwrap();
        assert!(all.is_empty());
    }
}
