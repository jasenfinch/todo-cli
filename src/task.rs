use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate};
use sha1::{Digest, Sha1};
use std::{fmt::Display, str::FromStr, time::SystemTime};

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
