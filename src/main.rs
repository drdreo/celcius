use dotenv::dotenv;
use quicli::prelude::*;
use std::env;
use structopt::StructOpt;

mod git;
mod pr;

type CliResult<T> = Result<T, quicli::prelude::Error>;

/// Useful utility to utilize when utility is needed.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Cli {
    // Quick and easy logging setup you get for free with quicli
    #[structopt(flatten)]
    verbose: Verbosity,

    #[structopt(subcommand)]
    cmd: CliSubcommands,
}

#[derive(Debug, StructOpt)]
enum CliSubcommands {
    #[structopt(
        name = "clean",
        about = "Utility to clean up all dangling local branches."
    )]
    Clean {
        #[structopt(
            name = "dry-run",
            long,
            help = "Lists all disconnected branches instead of deleting them."
        )]
        dry_run: bool,
    },
    #[structopt(name = "rewrite", about = "Rewrite commit history.")]
    Rewrite {
        #[structopt(
            name = "date",
            long,
            help = "The date used for the commits. Format: %Y-%m-%d %H:% e.g.: 2025-01-01 14:32. They will be randomly spread around the provided hour."
        )]
        date: String,
        #[structopt(
            short = "c",
            long = "count",
            help = "The amount of commits to rewrite. Default 1.",
            default_value = "1"
        )]
        count: usize,
    },
    #[structopt(name = "prstats", about = "Get PR statistics of the repository.")]
    PrStats {
        #[structopt(long, env = "GITHUB_API_TOKEN", help = "Your GitHub API token.")]
        token: Option<String>,
        #[structopt(default_value = "Studio", long, help = "The GitHub repository name.")]
        repo: String,
        #[structopt(
            default_value = "nordicfactory",
            long,
            help = "The GitHub repository owner."
        )]
        owner: String,

        #[structopt(
            short = "d",
            long = "days",
            help = "Set created in the past days limit",
            default_value = "14"
        )]
        days: u32,
    },
}

fn main() -> CliResult<()> {
    let args = Cli::from_args();
    args.verbose.setup_env_logger("celcius")?;
    dotenv().ok();

    match args.cmd {
        CliSubcommands::Clean { dry_run } => {
            match git::check_branches() {
                Ok(orphan_branches) => {
                    println!("Orphaned branches: {:?}", orphan_branches);
                    if dry_run {
                        println!("Did a dry run. No branches were removed.");
                    } else {
                        git::delete_local_branches(orphan_branches)?;
                    }
                }
                Err(error) => {
                    eprintln!("Error checking branches: {}", error);
                    // Handle the error as appropriate for your application
                }
            }
        }
        CliSubcommands::Rewrite { date, count } => match git::rewrite_date_of_commit(&date, count) {
            Ok(commits) => {
                println!("commits rewritten: {:?}", commits);
            }
            Err(error) => {
                eprintln!("Error rewriting history: {}", error);
            }
        },
        CliSubcommands::PrStats {
            repo,
            owner,
            days,
            token,
        } => {
            let api_token = token
                .or_else(|| env::var("GITHUB_API_TOKEN").ok())
                .expect("GITHUB_API_TOKEN not provided.");
            match pr::print_pr_statistics(api_token, repo, owner, days) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Error printing PR statistics: {}", error);
                }
            }
        }
    }

    Ok(())
}
