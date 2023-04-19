use quicli::prelude::*;
use structopt::StructOpt;

mod git;
mod pr;

type CliResult<T> = Result<T, quicli::prelude::Error>;

// Add cool slogan for your app here, e.g.:
/// Make your repo great again
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
    #[structopt(name = "prstats", about = "Get PR statistics of the repository.")]
    PrStats {
        #[structopt(default_value = "Studio", long, help = "The GitHub repository name.")]
        repo: String,
        #[structopt(
            default_value = "nordicfactory",
            long,
            help = "The GitHub repository owner."
        )]
        owner: String,

        #[structopt(short = "d", long = "days", help = "Set created in the past days limit", default_value = "14")]
        days: u32,
    },
}

fn main() -> CliResult<()> {
    let args = Cli::from_args();
    args.verbose.setup_env_logger("celcius")?;

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
        CliSubcommands::PrStats { repo, owner, days } => {
            match pr::print_pr_statistics(repo, owner, days) {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Error printing PR statistics: {}", error);
                }
            }
        }
    }

    Ok(())
}
