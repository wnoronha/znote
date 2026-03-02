mod commands;
mod handlers;
mod hooks;
mod models;
mod storage;
mod tests;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use commands::{
    AgentCommands, BookmarkCommands, Cli, Commands, ConfigCommands, ItemCommands, NoteCommands,
    SearchCommands, TaskCommands, ValidateCommands,
};

fn expand_data_dir(raw: &str) -> Result<PathBuf> {
    let expanded = if let Some(stripped) = raw.strip_prefix("~/") {
        let home = std::env::var("HOME").context("$HOME is not set")?;
        PathBuf::from(home).join(stripped)
    } else {
        PathBuf::from(raw)
    };
    Ok(expanded)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let data_dir = expand_data_dir(&cli.data_dir)?;

    match cli.command {
        Commands::Note { command } => match command {
            NoteCommands::Add(args) => handlers::note::add(&data_dir, &args)?,
            NoteCommands::List => handlers::note::list(&data_dir)?,
            NoteCommands::View { id } => handlers::note::view(&data_dir, &id)?,
            NoteCommands::Update(args) => handlers::note::update(&data_dir, &args)?,
            NoteCommands::Edit { id } => handlers::note::edit(&data_dir, &id)?,
            NoteCommands::Delete { id } => handlers::note::delete(&data_dir, &id)?,
        },
        Commands::Bookmark { command } => match command {
            BookmarkCommands::Add(args) => handlers::bookmark::add(&data_dir, &args)?,
            BookmarkCommands::List => handlers::bookmark::list(&data_dir)?,
            BookmarkCommands::View { id } => handlers::bookmark::view(&data_dir, &id)?,
            BookmarkCommands::Update(args) => handlers::bookmark::update(&data_dir, &args)?,
            BookmarkCommands::Edit { id } => handlers::bookmark::edit(&data_dir, &id)?,
            BookmarkCommands::Delete { id } => handlers::bookmark::delete(&data_dir, &id)?,
        },
        Commands::Task { command } => match command {
            TaskCommands::Add(args) => handlers::task::add(&data_dir, &args)?,
            TaskCommands::List => handlers::task::list(&data_dir)?,
            TaskCommands::View { id } => handlers::task::view(&data_dir, &id)?,
            TaskCommands::Update(args) => handlers::task::update(&data_dir, &args)?,
            TaskCommands::Edit { id } => handlers::task::edit(&data_dir, &id)?,
            TaskCommands::Delete { id } => handlers::task::delete(&data_dir, &id)?,
            TaskCommands::Item { task_id, command } => match command {
                ItemCommands::Add(args) => handlers::task::item_add(&data_dir, &task_id, &args)?,
                ItemCommands::Check { index } => {
                    handlers::task::item_check(&data_dir, &task_id, index, true)?
                }
                ItemCommands::Uncheck { index } => {
                    handlers::task::item_check(&data_dir, &task_id, index, false)?
                }
                ItemCommands::Update(args) => {
                    handlers::task::item_update(&data_dir, &task_id, &args)?
                }
                ItemCommands::Remove { index } => {
                    handlers::task::item_remove(&data_dir, &task_id, index)?
                }
            },
        },
        Commands::Search { command } => match command {
            SearchCommands::Rip { args } => handlers::search::rip(&data_dir, &args)?,
            SearchCommands::Query { expr } => handlers::query::run(&data_dir, &expr)?,
        },
        Commands::Config { command } => match command {
            ConfigCommands::Show => handlers::config::show(&data_dir)?,
        },
        Commands::Validate { command } => match command {
            ValidateCommands::Frontmatter => handlers::validate::frontmatter(&data_dir)?,
        },
        Commands::Graph(args) => handlers::graph::run(&data_dir, &args)?,
        Commands::Serve(args) => handlers::serve::run(&args.host, args.port, &data_dir).await?,
        Commands::Completions { shell } => handlers::completions::generate_completions(&shell),
        Commands::Complete { entity } => handlers::completions::complete_ids(&data_dir, &entity)?,
        Commands::Agent { command } => match command {
            AgentCommands::Skill => handlers::agent::skill()?,
        },
    }

    Ok(())
}
