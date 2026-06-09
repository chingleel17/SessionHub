use std::collections::BTreeSet;
use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension};

use crate::types::SessionTodo;

fn has_todos_table(connection: &Connection) -> Result<bool, String> {
    let exists = connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'todos' LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| format!("failed to inspect todos table: {error}"))?;

    Ok(exists.is_some())
}

fn read_todo_columns(connection: &Connection) -> Result<BTreeSet<String>, String> {
    let mut statement = connection
        .prepare("PRAGMA table_info(todos)")
        .map_err(|error| format!("failed to inspect todos schema: {error}"))?;
    let column_iter = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("failed to read todos schema rows: {error}"))?;

    let mut columns = BTreeSet::new();
    for column in column_iter {
        columns.insert(
            column.map_err(|error| format!("failed to read todos schema column: {error}"))?,
        );
    }

    Ok(columns)
}

fn select_column(columns: &BTreeSet<String>, name: &str, fallback_sql: &str) -> String {
    if columns.contains(name) {
        name.to_string()
    } else {
        format!("{fallback_sql} AS {name}")
    }
}

pub(crate) fn read_session_todos_internal(session_dir: &Path) -> Result<Vec<SessionTodo>, String> {
    let session_db_path = session_dir.join("session.db");
    if !session_db_path.exists() {
        return Ok(Vec::new());
    }

    let connection =
        Connection::open_with_flags(&session_db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| format!("failed to open {}: {error}", session_db_path.display()))?;

    if !has_todos_table(&connection)? {
        return Ok(Vec::new());
    }

    let columns = read_todo_columns(&connection)?;
    let query = format!(
        "SELECT {}, {}, {}, {}, {} FROM todos ORDER BY id ASC",
        select_column(&columns, "id", "''"),
        select_column(&columns, "title", "''"),
        select_column(&columns, "status", "'pending'"),
        select_column(&columns, "description", "NULL"),
        select_column(&columns, "updated_at", "NULL")
    );

    let mut statement = connection
        .prepare(&query)
        .map_err(|error| format!("failed to prepare todos query: {error}"))?;
    let todo_iter = statement
        .query_map([], |row| {
            Ok(SessionTodo {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                description: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|error| format!("failed to query todos: {error}"))?;

    let mut todos = Vec::new();
    for todo in todo_iter {
        todos.push(todo.map_err(|error| format!("failed to read todo row: {error}"))?);
    }

    Ok(todos)
}

#[tauri::command]
pub fn read_session_todos(session_dir: String) -> Result<Vec<SessionTodo>, String> {
    read_session_todos_internal(Path::new(&session_dir))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rusqlite::Connection;

    use super::read_session_todos_internal;

    fn create_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = env::temp_dir().join(format!("sessionhub-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn returns_empty_when_session_db_is_missing() {
        let session_dir = create_temp_dir("todos-missing-db");

        let todos = read_session_todos_internal(&session_dir).expect("missing db should not fail");

        assert!(todos.is_empty());
        let _ = fs::remove_dir_all(&session_dir);
    }

    #[test]
    fn returns_empty_when_todos_table_is_missing() {
        let session_dir = create_temp_dir("todos-missing-table");
        let session_db = session_dir.join("session.db");
        Connection::open(&session_db).expect("db should open");

        let todos =
            read_session_todos_internal(&session_dir).expect("missing table should not fail");

        assert!(todos.is_empty());
        let _ = fs::remove_dir_all(&session_dir);
    }

    #[test]
    fn reads_existing_todos_and_tolerates_missing_optional_columns() {
        let session_dir = create_temp_dir("todos-read");
        let session_db = session_dir.join("session.db");
        let connection = Connection::open(&session_db).expect("db should open");
        connection
            .execute(
                "CREATE TABLE todos (id TEXT PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL)",
                [],
            )
            .expect("todos table should be created");
        connection
            .execute(
                "INSERT INTO todos (id, title, status) VALUES ('a', 'First task', 'pending'), ('b', 'Second task', 'done')",
                [],
            )
            .expect("todos should be inserted");

        let todos = read_session_todos_internal(&session_dir).expect("todos should read");

        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].id, "a");
        assert_eq!(todos[0].title, "First task");
        assert_eq!(todos[0].status, "pending");
        assert_eq!(todos[0].description, None);
        assert_eq!(todos[0].updated_at, None);
        assert_eq!(todos[1].id, "b");
        assert_eq!(todos[1].status, "done");
        let _ = fs::remove_dir_all(&session_dir);
    }
}
