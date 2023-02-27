use std::process::Command;

use quicli::prelude::*;
use structopt::StructOpt;

type CliResult<T> = Result<T, quicli::prelude::Error>;

// Add cool slogan for your app here, e.g.:
/// Make your repo great again
#[derive(Debug, StructOpt)]
struct Cli {
    // Quick and easy logging setup you get for free with quicli
    #[structopt(flatten)]
    verbose: Verbosity,
}

fn main() -> CliResult<()> {
    let args = Cli::from_args();
    args.verbose.setup_env_logger("head")?;

    // fetch latest changes from remote
      let output = Command::new("git")
        .arg("branch")
        .arg("--list")
        .output()
        .expect("failed to execute command");

    if !output.status.success() {
        bail!("command failed: {}", output.status);
    }

    let branches = String::from_utf8(output.stdout)?;

    for branch in branches.lines() {
        let local_branch = branch.trim_start_matches("* ").trim();

        let remote_branch_cmd = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("--symbolic-full-name")
            .arg("@{u}")
            .arg(local_branch)
            .output()
            .expect("failed to execute command");

        let branch_status = if remote_branch_cmd.status.success() {
            "on remote"
        } else {
            "not on remote"
        };

        println!("{} is {}", local_branch, branch_status);
    }

    Ok(())
}
