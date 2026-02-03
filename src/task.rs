use crate::deadline::Deadline;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input};
use sha1::{Digest, Sha1};
use std::borrow::Cow;
use std::{fmt::Display, time::SystemTime};
use tabled::Tabled;

fn generate_hash(content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();

    format!("{:x}", result)
}

fn generate_content_hash(title: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let content = format!("blob {}\0{}{}", title.len(), title, timestamp);

    generate_hash(&content)
}

#[derive(Debug, Clone, Copy)]
pub struct Difficulty {
    value: u8,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.colour())
    }
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

impl From<u8> for Difficulty {
    fn from(value: u8) -> Self {
        Self { value }
    }
}

impl Into<u8> for Difficulty {
    fn into(self) -> u8 {
        self.value
    }
}

#[derive(Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub desc: Option<String>,
    pub difficulty: Option<Difficulty>,
    pub deadline: Option<Deadline>,
    pub tags: Option<Vec<String>>,
    pub pid: Option<String>,
    pub created: SystemTime,
    pub completed: bool,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} Task: {}",
            if self.completed { "✓" } else { "☐" },
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

        if let Some(tags) = &self.tags {
            if !tags.is_empty() {
                writeln!(f, "  Tags: {}", tags.join(", "))?;
            }
        }

        writeln!(f, "  ID: {}", self.id)?;

        if let Some(pid) = &self.pid {
            writeln!(f, "  Parent: {}", pid)?;
        }

        let created: DateTime<Local> = self.created.into();
        writeln!(f, "  Created: {}", created.format("%H:%M:%S %d-%m-%Y"))?;

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

        let task = Task {
            id: generate_content_hash(&title),
            title,
            desc,
            difficulty,
            deadline: date,
            tags,
            pid,
            created: SystemTime::now(),
            completed: false,
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
            .with_prompt("Deadline (DD-MM-YYYY, optional)")
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

    pub fn translate_to_db(
        self,
    ) -> (
        String,
        String,
        Option<String>,
        Option<u8>,
        Option<String>,
        Option<Vec<String>>,
        Option<String>,
        i64,
        bool,
    ) {
        let timestamp = self
            .created
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        (
            self.id,
            self.title,
            self.desc,
            self.difficulty.map(|d| d.into()),
            self.deadline.map(|d| d.to_string()),
            self.tags,
            self.pid,
            timestamp,
            self.completed,
        )
    }

    pub fn translate_from_db(
        row: (
            String,
            String,
            Option<String>,
            Option<u8>,
            Option<String>,
            Option<String>,
            i64,
            bool,
            Option<Vec<String>>,
        ),
    ) -> Result<Self> {
        let diff = match row.3 {
            Some(d) => Some(Difficulty::new(d)?),
            None => None,
        };

        let deadline = match row.4 {
            Some(d) => Some(Deadline::parse(&d)?),
            None => None,
        };

        let created = DateTime::from_timestamp(row.6, 0)
            .expect("Unable to parse created time stamp from the task db");

        Ok(Self {
            id: row.0,
            title: row.1,
            desc: row.2,
            difficulty: diff,
            deadline,
            tags: row.8,
            pid: row.5,
            created: created.into(),
            completed: row.7,
        })
    }
}

impl Tabled for Task {
    const LENGTH: usize = 9;

    fn fields(&self) -> Vec<std::borrow::Cow<'_, str>> {
        let created: DateTime<Local> = self.created.into();
        let created_str = created.format("%d-%m-%Y").to_string();
        let mut pid = self.pid.as_deref().unwrap_or("").to_string();

        if pid.chars().count() > 0 {
            pid = pid[0..7].to_string()
        }

        let deadline = match &self.deadline {
            Some(d) => d.days_until(),
            None => "".to_string(),
        };

        let difficulty = match self.difficulty {
            Some(d) => d.colour(),
            None => "".to_string(),
        };

        vec![
            Cow::Borrowed(&self.title),
            Cow::Owned(self.desc.as_deref().unwrap_or("").to_string()),
            Cow::Owned(difficulty),
            Cow::Owned(deadline),
            Cow::Owned(
                self.tags
                    .as_ref()
                    .map(|t| t.join(", "))
                    .unwrap_or("".to_string()),
            ),
            Cow::Borrowed(&self.id[0..7]),
            Cow::Owned(pid),
            Cow::Owned(created_str),
            Cow::Owned(if self.completed { "✓" } else { "" }.to_string()),
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
    fn test_task() {
        assert!(Task::new(
            "test".to_string(),
            Some("test".to_string()),
            Some(4),
            Some("23-01-2026".to_string()),
            None,
            None,
        )
        .is_ok());
        assert!(Task::new(
            "test".to_string(),
            Some("test".to_string()),
            Some(11),
            Some("23-01-2026".to_string()),
            Some(vec!["Work".to_string()]),
            None
        )
        .is_err());
        assert!(Task::new(
            "test".to_string(),
            Some("test".to_string()),
            Some(11),
            Some("incorrect".to_string()),
            None,
            None
        )
        .is_err());
    }
}
