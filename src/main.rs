use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use todo::{db::Database, display};

/// A Todo list CLI
#[derive(Debug, Parser)]
#[command(name = "todo")]
#[command(about = "A todo list CLI", long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// The path to the task database directory
    #[arg(short = 'p')]
    path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "List tasks")]
    List {
        #[arg(short, long, default_value = "compact")]
        view: display::ViewMode,
        #[arg(short, long, value_delimiter = ',', conflicts_with = "view")]
        columns: Option<Vec<display::Column>>,
    },
    #[command(about = "Clear all tasks")]
    Clear,
    #[command(about = "Add a task")]
    Add {
        /// The name of the task
        #[arg(value_name = "TASK")]
        title: String,
        /// A description of the task
        #[arg(short)]
        description: Option<String>,
        /// A value between 0 and 10. 0 is trivial and 10 is near-impossible
        #[arg(short = 'D')]
        difficulty: Option<u8>,
        /// The task deadline in the format YYYY-MM-DD
        #[arg(short = 'l')]
        deadline: Option<String>,
        /// Tags associated with a task
        #[arg(short, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        /// The parent task id if this is a subtask
        #[arg(short, long)]
        pid: Option<String>,
    },
    #[command(about = "Remove a task")]
    Remove {
        #[arg(value_name = "ID")]
        id: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut db = Database::load(args.path)?;

    match args.command {
        Commands::List { view, columns } => display::list_tasks(db, view, columns)?,
        Commands::Clear => {
            db.clear()?;
        }
        Commands::Add {
            title,
            description,
            difficulty,
            deadline,
            tags,
            pid,
        } => {
            let id = db.add(title, description, difficulty, deadline, tags, pid)?;
            println!("Added task with ID {id}");
        }
        Commands::Remove { id } => {
            let n = db.remove(id)?;
            println!("Removed {n} tasks")
        }
    };

    Ok(())
}
