use anyhow::Result;
use clap::Parser;
use dialoguer::Confirm;
use todo_cli::{
    cli::{Cli, Commands},
    db::Database,
    display,
    task::Task,
};

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
            all,
            completed,
        } => display::list_tasks(db, view, columns, tags, pid, all, completed)?,
        Commands::Remove { ids, tags } => {
            let n = match (ids, tags) {
                (Some(ids), None) => db.remove_ids(ids)?,
                (None, Some(tags)) => db.remove_tags(tags)?,
                _ => unreachable!("clap enforces exactly one is present"),
            };
            println!("Removed {} task(s)", n);
        }
        Commands::Tags => {
            let tags = db.tags()?;
            println!("{}", tags.join("  "))
        }
        Commands::Clear { force } => {
            let mut confirm = true;

            if !force {
                confirm = Confirm::new()
                    .with_prompt("Are you sure you want to clear ALL tasks? This cannot be undone.")
                    .default(false)
                    .interact()?;
            }

            if confirm {
                db.clear()?;
            }
        }
    };

    Ok(())
}
