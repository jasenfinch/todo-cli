use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDate};
use colored::Colorize;
use sha1::{Digest, Sha1};
use std::borrow::Cow;
use std::{fmt::Display, str::FromStr, time::SystemTime};
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

#[derive(Debug)]
pub struct Difficulty {
    value: u8,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
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
    pub deadline: Option<NaiveDate>,
    pub tags: Option<Vec<String>>,
    pub pid: Option<String>,
    pub created: SystemTime,
    pub completed: bool,
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
            Some(d) => Some(NaiveDate::parse_from_str(&d, "%Y-%m-%d")?),
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
            Some(d) => Some(NaiveDate::from_str(&d)?),
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

fn difficulty_colored(difficulty: &Option<Difficulty>) -> String {
    match difficulty {
        None => "-".to_string(),
        Some(d) => {
            let val = d.value;
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
}

impl Tabled for Task {
    const LENGTH: usize = 9;

    fn fields(&self) -> Vec<std::borrow::Cow<'_, str>> {
        let created: DateTime<Local> = self.created.into();
        let created_str = created.format("%Y-%m-%d").to_string();
        let mut pid = self.pid.as_deref().unwrap_or("").to_string();

        if pid.chars().count() > 0 {
            pid = pid[0..7].to_string()
        }

        let deadline = match self.deadline {
            Some(d) => {
                let days_until = (d - Local::now().date_naive()).num_days();
                if days_until < 0 {
                    format!("{} days ago", -days_until).red().to_string()
                } else {
                    format!("in {} days", days_until).to_string()
                }
            }
            None => "".to_string(),
        };

        vec![
            Cow::Borrowed(&self.id[0..7]),
            Cow::Borrowed(&self.title),
            Cow::Owned(self.desc.as_deref().unwrap_or("-").to_string()),
            Cow::Owned(difficulty_colored(&self.difficulty)),
            Cow::Owned(deadline),
            Cow::Owned(
                self.tags
                    .as_ref()
                    .map(|t| t.join(", "))
                    .unwrap_or("".to_string()),
            ),
            Cow::Owned(pid),
            Cow::Owned(created_str),
            Cow::Owned(if self.completed { "✓" } else { "" }.to_string()),
        ]
    }

    fn headers() -> Vec<std::borrow::Cow<'static, str>> {
        vec![
            Cow::Borrowed("ID"),
            Cow::Borrowed("Title"),
            Cow::Borrowed("Description"),
            Cow::Borrowed("Difficulty"),
            Cow::Borrowed("Deadline"),
            Cow::Borrowed("Tags"),
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
            Some("2026-01-23".to_string()),
            None,
            None,
        )
        .is_ok());
        assert!(Task::new(
            "test".to_string(),
            Some("test".to_string()),
            Some(11),
            Some("2026-01-23".to_string()),
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
