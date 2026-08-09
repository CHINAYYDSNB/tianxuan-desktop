use rusqlite::Connection;

pub fn run(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS hosts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 22,
            username TEXT NOT NULL,
            auth_type TEXT NOT NULL,
            auth_ref TEXT NOT NULL DEFAULT '',
            group_name TEXT NOT NULL DEFAULT '默认',
            tags TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS panels (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            panel_type TEXT NOT NULL DEFAULT 'bt',
            session_ref TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS command_history (
            id TEXT PRIMARY KEY,
            command TEXT NOT NULL,
            host_count INTEGER NOT NULL DEFAULT 0,
            executed_at TEXT NOT NULL,
            success_count INTEGER NOT NULL DEFAULT 0,
            fail_count INTEGER NOT NULL DEFAULT 0
        );
        ",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_run() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(tables.contains(&"hosts".to_string()));
        assert!(tables.contains(&"panels".to_string()));
        assert!(tables.contains(&"command_history".to_string()));
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        run(&conn).unwrap();
    }
}
