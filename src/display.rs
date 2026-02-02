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

pub fn list_tasks(db: Database, view: ViewMode) -> Result<()> {
    let tasks = db.get_tasks()?;

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    let mut table = Table::new(tasks);
    table.with(Style::psql()).with(AlignmentStrategy::PerLine);

    match view {
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

    println!("{}", table);

    Ok(())
}
