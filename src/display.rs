use crate::db::Database;
use anyhow::Result;
use clap::ValueEnum;
use tabled::{
    settings::{formatting::AlignmentStrategy, location::ByColumnName, Remove, Style},
    Table,
};

#[derive(ValueEnum, Debug, Clone)]
pub enum ViewMode {
    Minimal,
    Compact,
    Full,
}

impl ViewMode {
    fn select_cols(&self, table: &mut Table) {
        match self {
            ViewMode::Minimal => {
                table
                    .with(Remove::column(ByColumnName::new("Description")))
                    .with(Remove::column(ByColumnName::new("Difficulty")))
                    .with(Remove::column(ByColumnName::new("Deadline")))
                    .with(Remove::column(ByColumnName::new("Tags")))
                    .with(Remove::column(ByColumnName::new("Parent")))
                    .with(Remove::column(ByColumnName::new("Created")))
                    .with(Remove::column(ByColumnName::new("Complete")));
            }
            ViewMode::Compact => {
                table
                    .with(Remove::column(ByColumnName::new("Description")))
                    .with(Remove::column(ByColumnName::new("Created")))
                    .with(Remove::column(ByColumnName::new("Complete")));
            }
            ViewMode::Full => (),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Column {
    Id,
    Task,
    Description,
    Difficulty,
    Deadline,
    Tags,
    Parent,
    Created,
    Complete,
}

impl Column {
    fn as_str(&self) -> &'static str {
        match self {
            Column::Id => "ID",
            Column::Task => "Task",
            Column::Description => "Description",
            Column::Difficulty => "Difficulty",
            Column::Deadline => "Deadline",
            Column::Tags => "Tags",
            Column::Parent => "Parent",
            Column::Created => "Created",
            Column::Complete => "Complete",
        }
    }

    fn available() -> Vec<String> {
        vec![
            "ID".to_string(),
            "Task".to_string(),
            "Description".to_string(),
            "Difficulty".to_string(),
            "Deadline".to_string(),
            "Tags".to_string(),
            "Parent".to_string(),
            "Created".to_string(),
            "Complete".to_string(),
        ]
    }
}

pub fn list_tasks(
    db: Database,
    view: ViewMode,
    columns: Option<Vec<Column>>,
    tags: Option<Vec<String>>,
    pid: Option<String>,
    include_completed: bool,
    completed: bool,
) -> Result<()> {
    let tasks = db.get_tasks(tags, pid, include_completed, completed)?;

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    let mut table = Table::new(tasks);
    table.with(Style::modern()).with(AlignmentStrategy::PerLine);

    if columns.is_some() {
        let columns: Vec<&str> = columns
            .as_ref()
            .map(|cols| cols.iter().map(|c| c.as_str()).collect())
            .unwrap_or_else(|| vec!["ID", "Task", "Difficulty", "Deadline", "Tags", "Parent"]);
        let mut rem_cols = Column::available();
        rem_cols.retain(|x| !columns.contains(&x.as_str()));

        for col in rem_cols {
            table.with(Remove::column(ByColumnName::new(col)));
        }
    } else {
        view.select_cols(&mut table);
    }

    println!("{}", table);

    Ok(())
}
