use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::{fs, path::PathBuf};

use crate::task::Task;

pub struct Database {
    conn: Connection,
}

impl Database {
    fn get_path() -> Result<PathBuf> {
        let proj_dir = ProjectDirs::from("com", "Todo", "todo")
            .context("Could not determine the local store directory")?;
        let db_dir = proj_dir.data_dir();

        fs::create_dir_all(db_dir).context("Unable to create the local store directory")?;

        Ok(db_dir.join("tasks.db"))
    }

    pub fn load() -> Result<Self> {
        let db_path = Self::get_path()?;
        let conn = Connection::open(&db_path).context("Could not open the task database")?;

        let mut db = Database { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    fn initialize_schema(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                difficulty INTEGER,
                deadline INTEGER 
            )",
            [],
        )?;

        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM tasks", [])?;

        // Vacuum to reclaim space
        self.conn.execute("VACUUM", [])?;

        Ok(())
    }

    pub fn add(
        &mut self,
        title: String,
        description: Option<String>,
        difficulty: Option<u8>,
        deadline: Option<String>,
    ) -> Result<String> {
        let task = Task::new(title, description, difficulty, deadline)?;

        let (id, title, desc, diff, deadline) = task.translate_to_db();

        self.conn.execute(
            "INSERT INTO tasks (id, title, description, difficulty, deadline) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&id,title,desc,diff,deadline),
        )?;

        Ok(id[0..7].to_string())
    }

    pub fn remove(&mut self, id: String) -> Result<usize> {
        let pattern = format!("{id}%");
        let n = self
            .conn
            .execute("DELETE FROM tasks WHERE id LIKE ?1", [&pattern])?;

        Ok(n)
    }

    fn get_tasks(&self) -> Result<Vec<Task>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, description, difficulty, deadline FROM tasks")?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(Task::translate_from_db(row)?);
        }

        Ok(tasks)
    }

    pub fn list_tasks(&self) -> Result<()> {
        let tasks = self.get_tasks()?;

        if tasks.is_empty() {
            println!("No tasks found.");
            return Ok(());
        }

        for task in tasks {
            let desc = match task.desc {
                Some(d) => d,
                None => "".to_string(),
            };

            let diff = match task.difficulty {
                Some(d) => d.to_string(),
                None => "".to_string(),
            };

            let deadline = match task.deadline {
                Some(d) => d.to_string(),
                None => "".to_string(),
            };
            println!(
                "{} | {} | {} | {} | {}",
                &task.id[0..7].to_string(),
                task.title,
                desc,
                diff,
                deadline
            );
        }

        Ok(())
    }
}
