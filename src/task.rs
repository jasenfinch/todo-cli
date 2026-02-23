use crate::deadline::Deadline;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Local};
use colored::Colorize;
use dialoguer::{Input, theme::ColorfulTheme};
use rusqlite::ToSql;
use rusqlite::types::FromSql;
use sha1::{Digest, Sha1};
use std::borrow::Cow;
use std::{fmt::Display, time::SystemTime};
use tabled::Tabled;

#[derive(Debug, Clone)]
pub struct ID {
    value: String,
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<String> for ID {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<ID> for String {
    fn from(value: ID) -> Self {
        value.value
    }
}

impl ToSql for ID {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.value.clone().into())
    }
}

impl FromSql for ID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(ID::from(value.as_str()?.to_string()))
    }
}

impl ID {
    fn new(task_title: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let content = format!("blob {}\0{}{}", task_title.len(), task_title, timestamp);

        Self {
            value: generate_hash(&content),
        }
    }

    pub fn short(&self) -> String {
        self.value[0..7].to_string()
    }
}

fn generate_hash(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();

    format!("{:x}", result)
}

#[derive(Debug, Clone, Copy)]
pub struct Difficulty {
    value: u8,
}

impl Difficulty {
    fn new(value: u8) -> Result<Self> {
        if (0..=10).contains(&value) {
            Ok(Self { value })
        } else {
            Err(anyhow!(
                "Difficulty value out of range. The value should be between 0 and 10.",
            ))
        }
    }

    fn colour(&self) -> String {
        let val = self.value;
        let s = val.to_string();
        match val {
            0..=3 => s.green().to_string(),
            4..=6 => s.yellow().to_string(),
            7..=8 => s.bright_red().to_string(),
            9..=10 => s.red().bold().to_string(),
            _ => s.to_string(),
        }
    }
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.colour())
    }
}

impl ToSql for Difficulty {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.value.into())
    }
}

impl From<u8> for Difficulty {
    fn from(value: u8) -> Self {
        Self { value }
    }
}

impl From<Difficulty> for u8 {
    fn from(value: Difficulty) -> Self {
        value.value
    }
}

#[derive(Debug)]
pub struct Task {
    pub id: ID,
    pub title: String,
    pub desc: Option<String>,
    pub difficulty: Option<Difficulty>,
    pub deadline: Option<Deadline>,
    pub tags: Option<Vec<String>>,
    pub pid: Option<ID>,
    pub created: SystemTime,
    pub completed: Option<SystemTime>,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} Task: {}",
            if self.completed.is_some() {
                "✓".green()
            } else {
                "✗".red()
            },
            self.title
        )?;

        if let Some(desc) = &self.desc {
            writeln!(f, "  Description: {}", desc)?;
        }

        if let Some(diff) = &self.difficulty {
            writeln!(f, "  Difficulty: {}", diff)?;
        }

        if let Some(deadline) = &self.deadline {
            writeln!(f, "  Deadline: {} ({})", deadline, deadline.days_until())?;
        }

        if let Some(tags) = &self.tags
            && !tags.is_empty()
        {
            writeln!(f, "  Tags: {}", tags.join(", "))?;
        }

        writeln!(f, "  ID: {}", self.id)?;

        if let Some(pid) = &self.pid {
            writeln!(f, "  Parent: {}", pid)?;
        }

        let created: DateTime<Local> = self.created.into();
        writeln!(f, "  Created: {}", created.format("%H:%M:%S %d-%m-%Y"))?;

        if let Some(time) = self.completed {
            let completed: DateTime<Local> = time.into();
            writeln!(f, "  Completed: {}", completed.format("%H:%M:%S %d-%m-%Y"))?;
        }

        Ok(())
    }
}

impl Task {
    pub fn new(
        title: String,
        desc: Option<String>,
        difficulty: Option<u8>,
        deadline: Option<String>,
        tags: Option<Vec<String>>,
        pid: Option<String>,
    ) -> Result<Self> {
        let date = match deadline {
            Some(d) => Some(Deadline::parse(&d)?),
            None => None,
        };

        let difficulty = match difficulty {
            Some(d) => Some(Difficulty::new(d)?),
            None => None,
        };

        let pid = pid.map(ID::from);

        let task = Task {
            id: ID::new(&title),
            title,
            desc,
            difficulty,
            deadline: date,
            tags,
            pid,
            created: SystemTime::now(),
            completed: None,
        };

        Ok(task)
    }

    pub fn interactive() -> Result<Self> {
        let theme = ColorfulTheme::default();

        let title: String = Input::with_theme(&theme)
            .with_prompt("Task")
            .interact_text()?;

        let desc: Option<String> = Input::with_theme(&theme)
            .with_prompt("Description (optional)")
            .allow_empty(true)
            .interact_text()
            .ok()
            .filter(|s: &String| !s.is_empty());

        let difficulty = Input::with_theme(&theme)
            .with_prompt("Difficulty (0-10, optional)")
            .allow_empty(true)
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.is_empty() {
                    return Ok(());
                }

                match input.parse::<u8>() {
                    Ok(n) if n <= 10 => Ok(()),
                    Ok(_) => Err("Difficulty must be between 0 and 10"),
                    Err(_) => Err("Please enter a valid number"),
                }
            })
            .interact_text()
            .ok()
            .filter(|s: &String| !s.is_empty())
            .and_then(|s| s.parse().ok());

        let deadline: Option<String> = Input::with_theme(&theme)
            .with_prompt("Deadline (today, tomorrow, +5d, YYYY-MM-DD, or empty)")
            .allow_empty(true)
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.is_empty() {
                    return Ok(());
                }

                match Deadline::parse(input) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Invalid deadline format. Use: today, tomorrow, monday, +5d, 2026-02-10, etc. See `todo add --help` for more information."),
                }
            })
            .interact_text()
            .ok()
            .filter(|s: &String| !s.is_empty());

        let tags_str: Option<String> = Input::with_theme(&theme)
            .with_prompt("Tags (comma-separated, optional)")
            .allow_empty(true)
            .interact_text()
            .ok()
            .filter(|s: &String| !s.is_empty());

        let tags = tags_str.map(|s| s.split(',').map(|t| t.trim().to_string()).collect());

        let pid: Option<String> = Input::with_theme(&theme)
            .with_prompt("Parent task ID (optional)")
            .allow_empty(true)
            .interact_text()
            .ok()
            .filter(|s: &String| !s.is_empty());

        let task = Task::new(title, desc, difficulty, deadline, tags, pid)?;
        Ok(task)
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Task {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row<'_>) -> Result<Self, rusqlite::Error> {
        let diff: Option<u8> = row.get(3)?;
        let deadline: Option<String> = row.get(4)?;
        let created: i64 = row.get(6)?;

        Ok(Self {
            id: row.get::<_, String>(0)?.into(),
            title: row.get(1)?,
            desc: row.get(2)?,
            difficulty: diff
                .map(Difficulty::new)
                .transpose()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            deadline: deadline
                .map(|d| Deadline::parse(&d))
                .transpose()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            tags: None,
            pid: row.get::<_, Option<String>>(5)?.map(Into::into),
            created: DateTime::from_timestamp(created, 0)
                .expect("invalid timestamp")
                .into(),
            completed: row.get::<_, Option<i64>>(7)?.map(|t| {
                DateTime::from_timestamp(t, 0)
                    .expect("invalid timestamp")
                    .into()
            }),
        })
    }
}

impl Tabled for Task {
    const LENGTH: usize = 9;

    fn fields(&self) -> Vec<std::borrow::Cow<'_, str>> {
        let created: DateTime<Local> = self.created.into();
        let created_str = created.format("%d-%m-%Y").to_string();

        let pid = match &self.pid {
            Some(p) => p.short(),
            None => "".to_string(),
        };

        let deadline = match &self.deadline {
            Some(d) => d.days_until(),
            None => "".to_string(),
        };

        let difficulty = match self.difficulty {
            Some(d) => d.colour(),
            None => "".to_string(),
        };

        vec![
            Cow::Owned(truncate_string(&self.title, 30)),
            Cow::Owned(
                self.desc
                    .as_deref()
                    .map(|d| truncate_string(d, 40))
                    .unwrap_or("".to_string()),
            ),
            Cow::Owned(difficulty),
            Cow::Owned(deadline),
            Cow::Owned(truncate_string(
                &self
                    .tags
                    .as_ref()
                    .map(|t| t.join(", "))
                    .unwrap_or("".to_string()),
                30,
            )),
            Cow::Borrowed(&self.id.value[0..7]),
            Cow::Owned(pid),
            Cow::Owned(created_str),
            Cow::Owned(if self.completed.is_some() { "✓" } else { "" }.to_string()),
        ]
    }

    fn headers() -> Vec<std::borrow::Cow<'static, str>> {
        vec![
            Cow::Borrowed("Task"),
            Cow::Borrowed("Description"),
            Cow::Borrowed("Difficulty"),
            Cow::Borrowed("Deadline"),
            Cow::Borrowed("Tags"),
            Cow::Borrowed("ID"),
            Cow::Borrowed("Parent"),
            Cow::Borrowed("Created"),
            Cow::Borrowed("Complete"),
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_difficulty() {
        assert_eq!(Difficulty::new(5).unwrap().value, 5);
        assert!(Difficulty::new(11).is_err())
    }

    #[test]
    fn test_task_creation_with_valid_date() {
        let result = Task::new(
            "test".to_string(),
            Some("test".to_string()),
            Some(4),
            Some("23/01/2026".to_string()),
            None,
            None,
        );
        assert!(result.is_ok(), "Task creation failed: {:?}", result.err());
    }

    #[test]
    fn test_task_creation_with_invalid_date() {
        let result = Task::new(
            "test".to_string(),
            Some("test".to_string()),
            Some(4),
            Some("23-01-2026".to_string()),
            None,
            None,
        );
        assert!(
            result.is_err(),
            "Task creation should fail with an invalid date"
        );
    }

    #[test]
    fn test_task_creation_with_invalid_difficulty() {
        assert!(
            Task::new(
                "test".to_string(),
                Some("test".to_string()),
                Some(11),
                Some("23/01/2026".to_string()),
                Some(vec!["Work".to_string()]),
                None
            )
            .is_err()
        );
    }
}
