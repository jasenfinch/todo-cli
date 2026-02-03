use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use todo::{db::Database, display, task::Task};

/// A Todo list CLI
#[derive(Debug, Parser)]
#[command(name = "todo")]
#[command(about = "A task management and productivity CLI tool", long_about = None)]
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
    #[command(about = "Add a task")]
    Add {
        /// The name of the task
        #[arg(value_name = "TASK")]
        title: Option<String>,
        /// A description of the task
        #[arg(short, long = "desc")]
        description: Option<String>,
        /// A value between 0 and 10. 0 is trivial and 10 is near-impossible
        #[arg(short = 'D', long = "diff")]
        difficulty: Option<u8>,
        /// The task deadline in the format DD-MM-YYYY
        #[arg(short = 'l', long)]
        deadline: Option<String>,
        /// Tags associated with a task
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        /// The parent task id if this is a subtask
        #[arg(short, long, value_name = "PARENT_ID")]
        pid: Option<String>,
    },
    #[command(about = "Mark a task as complete")]
    Complete { id: String },
    #[command(about = "Show the next task to undertake based on task difficulty and deadline")]
    Next,
    #[command(about = "Show information about a task")]
    Show { id: String },
    #[command(about = "List tasks")]
    List {
        #[arg(short, long, default_value = "compact")]
        view: display::ViewMode,
        #[arg(short, long, value_delimiter = ',', conflicts_with = "view")]
        columns: Option<Vec<display::Column>>,
    },
    #[command(about = "Remove a task")]
    Remove {
        #[arg(value_name = "ID")]
        id: String,
    },
    #[command(about = "Clear all tasks")]
    Clear,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut db = Database::load(args.path)?;

    match args.command {
        Commands::Add {
            title,
            description,
            difficulty,
            deadline,
            tags,
            pid,
        } => {
            let task: Task;
            if let Some(t) = title {
                task = Task::new(t, description, difficulty, deadline, tags, pid)?;
            } else {
                task = Task::interactive()?;
            }
            let id = db.add(task)?;
            println!("Added task with ID {id}");
        }
        Commands::Complete { id } => {
            let id = db.completed(id)?;
            println!("Task with ID {id} marked as complete");
        }
        Commands::Next => {
            let task = db.next()?;
            println!("{}", task)
        }
        Commands::Show { id } => {
            let task = db.get_task(id)?;
            println!("{}", task)
        }
        Commands::List { view, columns } => display::list_tasks(db, view, columns)?,
        Commands::Remove { id } => {
            let n = db.remove(id)?;
            println!("Removed {n} tasks")
        }
        Commands::Clear => {
            db.clear()?;
        }
    };

    Ok(())
}
