use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use dialoguer::Confirm;
use std::path::PathBuf;
use todo::{db::Database, display, task::Task};

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

const DEADLINE_HELP: &str = r#"Deadline for the task

Supported formats:
  Keywords:
    today               - Today's date
    tomorrow, tmr       - Tomorrow
    monday, mon         - Next Monday (or any weekday)
    
  Relative:
    +5d, 5d             - 5 days from now
    +2w, 2weeks         - 2 weeks from now
    +1m, 1month         - 1 month from now
    
  Special:
    eow, endofweek      - End of current week (Sunday)
    eom, endofmonth     - Last day of current month
    eoy, endofyear      - December 31st
    
  Exact dates:
    2026-02-10          - ISO format (YYYY-MM-DD)
    02/10/2026          - UK format (DD/MM/YYYY)
    10-02-2026          - US format (MM-DD-YYYY)
    
  Examples:
    --deadline today
    --deadline friday
    --deadline +5d
    --deadline 2026-12-31"#;

const TAGS_HELP: &str = r#"Tags associated with a task
  Examples:
    --tags work
    --tags work,project"#;

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

        #[arg(short = 'l', long)]
        #[arg(long_help = DEADLINE_HELP )]
        deadline: Option<String>,

        #[arg(short, long, value_delimiter = ',')]
        #[arg(long_help = TAGS_HELP)]
        tags: Option<Vec<String>>,

        /// The parent task id if this is a subtask
        #[arg(short, long, value_name = "PARENT_ID")]
        pid: Option<String>,
    },
    #[command(about = "Mark a task as complete")]
    Complete { id: String },
    #[command(about = "Update a task")]
    Update {
        id: String,

        /// The name of the task
        #[arg(long = "task", value_name = "TASK")]
        title: Option<String>,

        /// A description of the task
        #[arg(short, long = "desc")]
        description: Option<String>,

        /// A value between 0 and 10. 0 is trivial and 10 is near-impossible
        #[arg(short = 'D', long = "diff")]
        difficulty: Option<u8>,

        #[arg(short = 'l', long)]
        #[arg(long_help = DEADLINE_HELP )]
        deadline: Option<String>,

        #[arg(short, long, value_delimiter = ',')]
        #[arg(long_help = TAGS_HELP)]
        tags: Option<Vec<String>>,

        /// The parent task id if this is a subtask
        #[arg(short, long, value_name = "PARENT_ID")]
        pid: Option<String>,
    },
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

        /// Show only tasks with specific tags
        #[arg(short, long, value_delimiter = ',', conflicts_with = "pid")]
        tags: Option<Vec<String>>,

        /// Show only the task with parent ID along with its child tasks
        #[arg(short, long, conflicts_with = "tags")]
        pid: Option<String>,

        /// Include completed tasks
        #[arg(long, conflicts_with = "completed")]
        include_completed: bool,

        /// Show only completed tasks
        #[arg(long, conflicts_with = "include_completed")]
        completed: bool,
    },
    #[command(about = "Remove tasks")]
    Remove {
        #[arg(value_name = "IDs", conflicts_with = "tags", num_args = 1..)]
        ids: Option<Vec<String>>,
        #[arg(short, long, value_delimiter = ',', conflicts_with = "ids")]
        #[arg(long_help = TAGS_HELP)]
        tags: Option<Vec<String>>,
    },
    #[command(about = "Clear all tasks")]
    Clear {
        #[arg(short, long)]
        force: bool,
    },
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
        Commands::Update {
            id,
            title,
            description,
            difficulty,
            deadline,
            tags,
            pid,
        } => {
            let mut task_title = "".to_string();

            if let Some(t) = title {
                task_title = t
            }

            let task = Task::new(task_title, description, difficulty, deadline, tags, pid)?;

            let id = db.update(id, task)?;
            println!("Updated task with ID {id}");
        }
        Commands::Next => {
            let task = db.next()?;
            println!("{}", task)
        }
        Commands::Show { id } => {
            let task = db.get_task(id)?;
            println!("{}", task)
        }
        Commands::List {
            view,
            columns,
            tags,
            pid,
            include_completed,
            completed,
        } => display::list_tasks(db, view, columns, tags, pid, include_completed, completed)?,
        Commands::Remove { ids, tags } => {
            if let Some(tags) = tags {
                let n = db.remove_tags(tags)?;
                println!("Removed {} tasks", n)
            } else if let Some(ids) = ids {
                let n = db.remove_ids(ids)?;
                println!("Removed {} tasks", n)
            } else {
                bail!("Must provide either task IDs or --tags.");
            }
        }
        Commands::Clear { force } => {
            let mut confirm = true;

            if !force {
                confirm = Confirm::new()
                    .with_prompt("Are you that you want to clear all tasks?")
                    .interact()?;
            }

            if confirm {
                db.clear()?;
            }
        }
    };

    Ok(())
}
