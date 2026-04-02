mod artifact;
mod cli;
mod commands;
mod error;
mod lock;
mod output;
mod schema;
mod validate;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};
use schema::find_ark_root;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match &cli.command {
        Command::Init => commands::init::run(&cwd),

        Command::Types => {
            let root = find_ark_root(&cwd)?;
            commands::types::run(&root, &cli.format)
        }

        Command::Fields(args) => {
            let root = find_ark_root(&cwd)?;
            commands::fields::run(
                &root,
                &args.artifact_type,
                args.field.as_deref(),
                &cli.format,
            )
        }

        Command::List(args) => {
            let root = find_ark_root(&cwd)?;
            commands::list::run(&root, args, &cli.format)
        }

        Command::Next(args) => {
            let root = find_ark_root(&cwd)?;
            commands::next::run(&root, &args.artifact_type, args.count, &cli.format)
        }

        Command::Show(args) => {
            let root = find_ark_root(&cwd)?;
            commands::show::run(&root, &args.id, &cli.format)
        }

        Command::New(args) => {
            let root = find_ark_root(&cwd)?;
            commands::new::run(&root, args)
        }

        Command::Edit(args) => {
            let root = find_ark_root(&cwd)?;
            commands::edit::run(&root, args)
        }

        Command::Lint(args) => {
            let root = find_ark_root(&cwd)?;
            commands::lint::run(&root, args.target.as_deref(), args.fix)
        }

        Command::Archive(args) => {
            let root = find_ark_root(&cwd)?;
            commands::archive::run(&root, &args.artifact_type)
        }

        Command::Rebalance(args) => {
            let root = find_ark_root(&cwd)?;
            commands::rebalance::run(&root, &args.artifact_type, args.gap)
        }

        Command::Stats(args) => {
            let root = find_ark_root(&cwd)?;
            commands::stats::run(
                &root,
                args.artifact_type.as_deref(),
                args.by.as_deref(),
                &cli.format,
            )
        }

        Command::Search(args) => {
            let root = find_ark_root(&cwd)?;
            commands::search::run(
                &root,
                &args.pattern,
                args.artifact_type.as_deref(),
                &cli.format,
            )
        }

        Command::Completions(args) => commands::completions::run(args.shell),

        Command::SchemaHelp => commands::schema_help::run(),
    }
}
