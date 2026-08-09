use rusqlite::{params, Connection};

use crate::models::{Panel, PanelType};

fn panel_type_to_str(panel: &PanelType) -> &'static str {
    match panel {
        PanelType::Bt => "bt",
        PanelType::OnePanel => "1panel",
    }
}

pub fn insert(conn: &Connection, panel: &Panel) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO panels (id, name, url, panel_type, session_ref, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            panel.id,
            panel.name,
            panel.url,
            panel_type_to_str(&panel.panel_type),
            panel.session_ref,
            panel.created_at,
            panel.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, panel: &Panel) -> rusqlite::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE panels SET name=?1, url=?2, panel_type=?3, session_ref=?4, updated_at=?5 WHERE id=?6",
        params![
            panel.name,
            panel.url,
            panel_type_to_str(&panel.panel_type),
            panel.session_ref,
            now,
            panel.id,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM panels WHERE id=?1", params![id])
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Panel>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, panel_type, session_ref, created_at, updated_at FROM panels ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], row_to_panel)?;
    rows.collect()
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Panel>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, panel_type, session_ref, created_at, updated_at FROM panels WHERE id=?1",
    )?;
    let mut rows = stmt.query_map(params![id], row_to_panel)?;
    match rows.next() {
        Some(Ok(panel)) => Ok(Some(panel)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

fn row_to_panel(row: &rusqlite::Row) -> rusqlite::Result<Panel> {
    let panel_type: String = row.get(3)?;
    let panel_type = match panel_type.as_str() {
        "1panel" => PanelType::OnePanel,
        _ => PanelType::Bt,
    };
    Ok(Panel {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        panel_type,
        session_ref: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NewPanel;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrate::run(&conn).unwrap();
        conn
    }

    fn sample_panel() -> Panel {
        NewPanel {
            name: "BT Panel".to_string(),
            url: "https://panel.example.com:8888".to_string(),
            panel_type: PanelType::Bt,
        }
        .into_panel()
    }

    #[test]
    fn test_crud_roundtrip() {
        let conn = test_conn();
        let panel = sample_panel();
        insert(&conn, &panel).unwrap();

        let fetched = get(&conn, &panel.id).unwrap().unwrap();
        assert_eq!(fetched.name, "BT Panel");
        assert_eq!(fetched.url, "https://panel.example.com:8888");
        assert_eq!(fetched.panel_type, PanelType::Bt);

        let all = list(&conn).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_update() {
        let conn = test_conn();
        let mut panel = sample_panel();
        insert(&conn, &panel).unwrap();

        panel.name = "1Panel Prod".to_string();
        panel.panel_type = PanelType::OnePanel;
        update(&conn, &panel).unwrap();

        let fetched = get(&conn, &panel.id).unwrap().unwrap();
        assert_eq!(fetched.name, "1Panel Prod");
        assert_eq!(fetched.panel_type, PanelType::OnePanel);
    }

    #[test]
    fn test_delete() {
        let conn = test_conn();
        let panel = sample_panel();
        insert(&conn, &panel).unwrap();
        assert_eq!(list(&conn).unwrap().len(), 1);

        let removed = delete(&conn, &panel.id).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(list(&conn).unwrap().len(), 0);
    }
}
