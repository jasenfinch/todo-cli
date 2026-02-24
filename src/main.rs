use anyhow::Result;
use clap::Parser;
use todo_cli::{
    cli::{Cli, Commands},
    db::Database,
};

fn main() -> Result<()> {
    let args = Cli::parse();
    let db = Database::load(args.path)?;

    match args.command {
        Commands::Add {
            title,
            description,
            difficulty,
            deadline,
            tags,
            pid,
        } => Commands::add(db, title, description, difficulty, deadline, tags, pid)?,
        Commands::Complete { id } => Commands::complete(db, id)?,
        Commands::Update {
            id,
            title,
            description,
            difficulty,
            deadline,
            tags,
            pid,
        } => Commands::update(db, id, title, description, difficulty, deadline, tags, pid)?,
        Commands::Next => Commands::next(db)?,
        Commands::Show { id } => Commands::show(db, id)?,
        Commands::List {
            view,
            columns,
            tags,
            pid,
            all,
            completed,
        } => Commands::list(db, view, columns, tags, pid, all, completed)?,
        Commands::Remove { ids, tags } => Commands::remove(db, ids, tags)?,
        Commands::Tags => Commands::tags(db)?,
        Commands::Clear { force } => Commands::clear(db, force)?,
    };

    Ok(())
}
