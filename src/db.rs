use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::{fs, path::PathBuf};

use crate::task::Task;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    fn get_path(path: Option<PathBuf>) -> Result<PathBuf> {
        let db_dir = match path {
            Some(dir) => dir,
            None => {
                let proj_dir = ProjectDirs::from("com", "Todo", "todo")
                    .context("Could not determine the local store directory")?;
                proj_dir.data_dir().to_path_buf()
            }
        };

        fs::create_dir_all(&db_dir).context("Unable to create the local store directory")?;

        Ok(db_dir.join("tasks.db"))
    }

    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        let db_path = Self::get_path(path)?;
        let conn = Connection::open(&db_path).context("Could not open the task database")?;

        let mut db = Database { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    fn initialize_schema(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                difficulty INTEGER,
                deadline INTEGER, 
                completed BOOLEAN DEFAULT 0,
                parent_id TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES tasks(id) ON DELETE CASCADE
            );
            
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            );

            CREATE TABLE IF NOT EXISTS task_tags (
                task_id TEXT NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (task_id, tag_id),
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_parent ON tasks(parent_id);
            CREATE INDEX IF NOT EXISTS idx_task_tags_task ON task_tags(task_id);
            CREATE INDEX IF NOT EXISTS idx_task_tags_tag ON task_tags(tag_id);
            ",
        )?;

        self.conn.execute("PRAGMA foreign_keys = ON", [])?;

        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM tasks", [])?;
        self.conn.execute("DELETE FROM tags", [])?;
        self.conn.execute("DELETE FROM task_tags", [])?;

        self.conn.execute("VACUUM", [])?;

        Ok(())
    }

    pub fn add(&mut self, task: Task) -> Result<String> {
        let (id, title, desc, diff, deadline, tags, mut pid, created, completed) =
            task.translate_to_db();

        if pid.is_some() {
            let parent_id = pid.unwrap();
            let pattern = format!("{parent_id}%");
            pid = self
                .conn
                .query_row("SELECT id FROM tasks WHERE id LIKE ?1", [&pattern], |row| {
                    row.get(0)
                })
                .context("Unable to find parent ID")?;
        }

        self.conn.execute(
            "INSERT INTO tasks (id, title, description, difficulty, deadline, parent_id, created_at, completed) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (&id,title,desc,diff,deadline,pid,created,completed),
        )?;

        let tags = tags.unwrap_or_default();

        if !tags.is_empty() {
            for tag in tags {
                self.conn
                    .execute("INSERT OR IGNORE INTO tags(name) VALUES (?1)", (&tag,))?;

                let tag_id: i64 =
                    self.conn
                        .query_row("SELECT id FROM tags WHERE name = ?1", [&tag], |row| {
                            row.get(0)
                        })?;

                self.conn.execute(
                    "INSERT INTO task_tags(task_id,tag_id) VALUES (?1,?2)",
                    (&id, tag_id),
                )?;
            }
        }

        Ok(id[0..7].to_string())
    }

    pub fn completed(&mut self, id: String) -> Result<String> {
        let pattern = format!("{}%", id);
        let n = self.conn.execute(
            "UPDATE tasks SET completed = 1 WHERE id LIKE ?1",
            [&pattern],
        )?;

        if n == 0 {
            bail!("No task found matching '{}'", id);
        }

        Ok(id)
    }

    pub fn tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT tags.name 
                FROM tags",
        )?;

        let tags = stmt
            .query_map([], |r| r.get(0))?
            .map(|r| r.map_err(anyhow::Error::from))
            .collect::<Result<Vec<String>, anyhow::Error>>()?;

        Ok(tags)
    }

    pub fn update(&mut self, id: String, mut updates: Task) -> Result<String> {
        let existing = self.get_task(id.clone())?;

        if updates.title.is_empty() {
            updates.title = existing.title;
        }
        if updates.desc.is_none() {
            updates.desc = existing.desc;
        }
        if updates.difficulty.is_none() {
            updates.difficulty = existing.difficulty;
        }
        if updates.deadline.is_none() {
            updates.deadline = existing.deadline;
        }
        if updates.pid.is_none() {
            updates.pid = existing.pid;
        }
        updates.id = existing.id.clone();
        updates.created = existing.created;

        let db_task = updates.translate_to_db();

        self.conn.execute(
            "UPDATE tasks SET 
            title = ?2,
            description = ?3,
            difficulty = ?4,
            deadline = ?5,
            parent_id = ?6,
            completed = ?7
         WHERE id = ?1",
            (
                &db_task.0, &db_task.1, &db_task.2, &db_task.3, &db_task.4, &db_task.6, db_task.8,
            ),
        )?;

        if let Some(new_tags) = db_task.5 {
            self.update_task_tags(&db_task.0, &new_tags)?;
        }

        Ok(id)
    }

    fn update_task_tags(&mut self, task_id: &str, tags: &[String]) -> Result<()> {
        self.conn
            .execute("DELETE FROM task_tags WHERE task_id = ?1", [task_id])?;

        for tag_name in tags {
            self.conn
                .execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", [tag_name])?;

            let tag_id: i64 =
                self.conn
                    .query_row("SELECT id FROM tags WHERE name = ?1", [tag_name], |row| {
                        row.get(0)
                    })?;

            self.conn.execute(
                "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                (task_id, tag_id),
            )?;
        }

        Ok(())
    }

    pub fn next(&self) -> Result<Task> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, difficulty, deadline, parent_id, created_at, completed
         FROM tasks
         WHERE completed = 0
         ORDER BY 
            CASE WHEN deadline IS NULL THEN 1 ELSE 0 END,  
            deadline ASC,
            difficulty DESC
         LIMIT 1",
        )?;

        let mut row = stmt.query_row([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                None,
            ))
        })?;

        let tags = self.get_tags(&row.0)?;
        if !tags.is_empty() {
            row.8 = Some(tags);
        }
        let task = Task::translate_from_db(row)?;

        Ok(task)
    }

    pub fn remove_ids(&mut self, ids: Vec<String>) -> Result<usize> {
        let mut valid_ids = Vec::new();

        for id in ids {
            match self.get_task(id.clone()) {
                Ok(_) => valid_ids.push(id),
                Err(e) => {
                    eprintln!("Warning: {}", e)
                }
            };
        }

        let mut n = 0;
        for id in valid_ids {
            let pattern = format!("{id}%");
            n += self
                .conn
                .execute("DELETE FROM tasks WHERE id LIKE ?1", [&pattern])?;
        }

        Ok(n)
    }

    pub fn remove_tags(&self, tags: Vec<String>) -> Result<usize> {
        let mut n = 0;

        for tag in tags {
            let mut stmt = self.conn.prepare(
                "SELECT t.id
                 FROM tasks t
                 JOIN task_tags tt ON t.id = tt.task_id
                 JOIN tags tg ON tt.tag_id = tg.id
                 WHERE tg.name = ?1",
            )?;

            let ids = stmt
                .query_map([tag], |row| row.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;

            for id in ids {
                let pattern = format!("{id}%");
                n += self
                    .conn
                    .execute("DELETE FROM tasks WHERE id LIKE ?1", [&pattern])?;
            }
        }

        Ok(n)
    }

    fn get_tags(&self, id: &String) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT tags.name 
                FROM tags
                JOIN task_tags ON tags.id = task_tags.tag_id
                WHERE task_tags.task_id = ?1
                ORDER BY tags.name",
        )?;

        let tags = stmt
            .query_map([id], |r| r.get(0))?
            .map(|r| r.map_err(anyhow::Error::from))
            .collect::<Result<Vec<String>, anyhow::Error>>()?;

        Ok(tags)
    }

    pub fn get_task(&self, id: String) -> Result<Task> {
        let pattern = format!("{id}%");
        let mut row =
            self.conn
                .query_row("SELECT id, title, description, difficulty, deadline, parent_id, created_at, completed FROM tasks WHERE id LIKE ?1", [&pattern], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        None,
                    ))
                })
        .context(format!("No task found matching ID '{}'", id))?;

        let tags = self.get_tags(&row.0)?;
        if !tags.is_empty() {
            row.8 = Some(tags);
        }

        Task::translate_from_db(row)
    }

    pub fn get_tasks(
        &self,
        tags: Option<Vec<String>>,
        pid: Option<String>,
        all: bool,
        only_completed: bool,
    ) -> Result<Vec<Task>> {
        let mut query = String::from(
            "SELECT DISTINCT t.id, t.title, t.description, t.difficulty, t.deadline, 
                t.parent_id, t.created_at, t.completed
         FROM tasks t",
        );

        let mut joins = Vec::new();
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(tag_list) = &tags {
            joins.push("JOIN task_tags tt ON t.id = tt.task_id");
            joins.push("JOIN tags tg ON tt.tag_id = tg.id");

            let placeholders = tag_list.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            conditions.push(format!("tg.name IN ({})", placeholders));

            for tag in tag_list {
                params.push(Box::new(tag.clone()));
            }
        }

        if let Some(parent_id) = &pid {
            let pattern = format!("{}%", parent_id);
            conditions.push("t.parent_id LIKE ?".to_string());
            params.push(Box::new(pattern));
        }

        if only_completed {
            conditions.push("t.completed = 1".to_string());
        } else if !all {
            conditions.push("t.completed = 0".to_string());
        }

        if !joins.is_empty() {
            query.push(' ');
            query.push_str(&joins.join(" "));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(
            " ORDER BY 
            CASE WHEN t.deadline IS NULL THEN 1 ELSE 0 END,
            t.deadline ASC,
            t.difficulty DESC",
        );

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    None,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut tasks = Vec::new();
        for mut row in rows {
            let tags = self.get_tags(&row.0)?;
            if !tags.is_empty() {
                row.8 = Some(tags);
            }
            tasks.push(Task::translate_from_db(row)?);
        }

        Ok(tasks)
    }
}
