use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::display;

#[derive(Debug, Parser)]
#[command(name = "todo")]
#[command(author,version, long_about = None)]
#[command(after_help = r#"EXAMPLES:
    todo add "Fix bug" --diff 5 --deadline friday --tags work,urgent
    todo list --view minimal
    todo complete abc123
    todo remove def456 ghi789
    todo remove --tags work"#)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// The path to the task database directory
    #[arg(short = 'p')]
    pub path: Option<PathBuf>,
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

fn pid_validator(s: &str) -> Result<String, String> {
    if s.len() == 7 {
        Ok(s.to_owned())
    } else {
        Err("Parent ID must be 7 characters long".to_string())
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Add a task")]
    Add {
        /// The name of the task
        #[arg(value_name = "TASK")]
        title: Option<String>,

        /// A description of the task
        #[arg(short, long = "desc")]
        description: Option<String>,

        /// A value between 0 and 10 (0=trivial, 10=near-impossible)
        #[arg(long = "diff", value_parser = clap::value_parser!(u8).range(0..=10))]
        difficulty: Option<u8>,

        #[arg(short = 'l', long)]
        #[arg(long_help = DEADLINE_HELP )]
        deadline: Option<String>,

        #[arg(short, long, value_delimiter = ',')]
        #[arg(long_help = TAGS_HELP)]
        tags: Option<Vec<String>>,

        /// The parent task id if this is a subtask
        #[arg(short, long, value_name = "PARENT_ID")]
        #[arg(value_parser = pid_validator)]
        pid: Option<String>,
    },
    #[command(alias = "done", about = "Mark a task as complete")]
    Complete { id: String },
    #[command(about = "Update a task (only specified fields are changed)")]
    Update {
        id: String,

        /// The name of the task
        #[arg(long = "task", value_name = "TASK")]
        title: Option<String>,

        /// A description of the task
        #[arg(short, long = "desc")]
        description: Option<String>,

        /// A value between 0 and 10 (0=trivial, 10=near-impossible)
        #[arg(long = "diff", value_parser = clap::value_parser!(u8).range(0..=10))]
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
    #[command(alias = "ls", about = "List tasks")]
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

        /// Show all tasks including completed
        #[arg(long, conflicts_with = "completed")]
        all: bool,

        /// Show only completed tasks
        #[arg(long, conflicts_with = "all")]
        completed: bool,
    },
    #[command(about = "List all tags")]
    Tags,
    #[command(alias = "rm", about = "Remove tasks")]
    Remove {
        #[arg(
            value_name = "IDs",
            required_unless_present = "tags",
            conflicts_with = "tags",
            num_args = 1..
            )]
        ids: Option<Vec<String>>,
        #[arg(short, long, value_delimiter = ',', required_unless_present = "ids")]
        #[arg(long_help = TAGS_HELP)]
        tags: Option<Vec<String>>,
    },
    #[command(about = "Clear all tasks")]
    Clear {
        #[arg(short, long)]
        force: bool,
    },
}
