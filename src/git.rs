use quicli::prelude::*;
use std::process::Command;

type CliResult<T> = Result<T, quicli::prelude::Error>;

pub fn check_branches() -> CliResult<Vec<String>> {
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

    let local_branches_vec: Vec<String> = local_branches_str
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let remote_branches_vec: Vec<String> = remote_branches_str
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    info!("Local branches: {:?}", local_branches_vec);
    info!("Remote branches: {:?}", remote_branches_vec);

    let mut orphan_branches: Vec<String> = Vec::new();

    for branch in local_branches_vec {
        // skip currently checked out branches
        if branch.starts_with('*') {
            continue;
        }

        let remote_branch = format!("origin/{}", branch);
        if !remote_branches_vec.contains(&remote_branch) {
            println!("Branch {} is no longer on the remote host", branch);
            orphan_branches.push(branch);
        }
    }

    Ok(orphan_branches)
}

fn delete_local_branch(name: &String) {
    let output = Command::new("git")
        .arg("branch")
        .arg("-D")
        .arg(name)
        .output()
        .expect("failed to execute command");

    if !output.status.success() {
        warn!(
            "failed to delete local branch '{}': {}",
            &name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

pub fn delete_local_branches(names: Vec<String>) -> CliResult<()> {
    for name in names {
        delete_local_branch(&name);
        info!("Deleted local branch '{}'", name);
    }
    Ok(())
}
