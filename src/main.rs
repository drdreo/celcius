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
    args.verbose.setup_env_logger("celcius")?;

    // todo: create new vector that stores only local branches, return them from that function and call next function delete_branches with it
    check_branches()?;

    let branches_to_delete = &["local-only-branch"];
    delete_local_branches(branches_to_delete)?;

    Ok(())
}

fn check_branches() -> CliResult<()>{
    let output = Command::new("git")
        .arg("fetch")
        .arg("--prune")
        .output()
        .expect("failed to execute command");

    if !output.status.success() {
        bail!("command failed: {}", output.status);
    }

    let _branches_output = Command::new("git")
        .arg("remote")
        .arg("update")
        .arg("--prune")
        .output()
        .expect("failed to execute command");

    let local_branches = Command::new("git")
        .arg("branch")
        .output()
        .expect("failed to execute command");

    if !local_branches.status.success() {
        bail!("command failed: {}", local_branches.status);
    }

    let local_branches_str = String::from_utf8(local_branches.stdout)?;

    let remote_branches = Command::new("git")
        .arg("branch")
        .arg("-r")
        .output()
        .expect("failed to execute command");

    if !remote_branches.status.success() {
        bail!("command failed: {}", remote_branches.status);
    }

    let remote_branches_str = String::from_utf8(remote_branches.stdout)?;

    let local_branches_vec: Vec<_> = local_branches_str
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let remote_branches_vec: Vec<_> = remote_branches_str
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    info!("Local branches: {:?}", local_branches_vec);
    info!("Remote branches: {:?}", remote_branches_vec);

    for branch in local_branches_vec {
        // skip currently checked out branches
        if branch.starts_with('*') {
            continue;
        }

        let remote_branch = format!("origin/{}", branch);
        if !remote_branches_vec.contains(&remote_branch) {
            println!("Branch {} is no longer on the remote host", branch);
        }
    }

    Ok(())
}

fn delete_local_branch(name: &str) {
    let output = Command::new("git")
        .arg("branch")
        .arg("-D")
        .arg(name)
        .output()
        .expect("failed to execute command");

    if !output.status.success() {
       warn!("failed to delete local branch '{}': {}", name, String::from_utf8_lossy(&output.stderr));
    }

}

fn delete_local_branches(names: &[&str]) -> CliResult<()> {
    for name in names {
        delete_local_branch(name);
        info!("Deleted local branch '{}'", name);
    }
    Ok(())
}