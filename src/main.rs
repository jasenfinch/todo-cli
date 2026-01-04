use anyhow::Result;
use clap::{Parser, Subcommand};
use todo::db::Database;

/// A Todo list CLI
#[derive(Debug, Parser)]
#[command(name = "todo")]
#[command(about = "A todo list CLI", long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "List all tasks")]
    List,
    #[command(about = "Clear all tasks")]
    Clear,
    #[command(about = "Add a task")]
    Add {
        /// The task title
        #[arg(value_name = "TITLE")]
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
    },
    #[command(about = "Remove a task")]
    Remove {
        #[arg(value_name = "ID")]
        id: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut db = Database::load()?;

    match args.command {
        Commands::List => db.list_tasks()?,
        Commands::Clear => {
            db.clear()?;
        }
        Commands::Add {
            title,
            description,
            difficulty,
            deadline,
        } => {
            let id = db.add(title, description, difficulty, deadline)?;
            println!("Added task with ID {id}");
        }
        Commands::Remove { id } => {
            let n = db.remove(id)?;
            println!("Removed {n} tasks")
        }
    };

    Ok(())
}
