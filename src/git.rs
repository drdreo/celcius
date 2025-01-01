use chrono::{DateTime, NaiveDateTime, Utc};
use quicli::prelude::*;
use std::process::Command;

#[derive(Debug)]
pub struct CommitDateUpdate {
    commit_hash: String,
    original_date: DateTime<Utc>,
    new_date: DateTime<Utc>,
}

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

pub fn rewrite_date_of_commit(date: &String, count: usize) -> CliResult<Vec<CommitDateUpdate>> {
    let new_date = NaiveDateTime::parse_from_str(date.as_str(), "%Y-%m-%d %H:%M")
        .expect("Failed to parse date");

    debug!("Rewriting history to {} ", new_date);

    // Get commit hashes and dates
    let git_log_output = Command::new("git")
        .args(["log", "-n", &count.to_string(), "--format=%H%n%aI"])
        .output()
        .expect("failed to execute `git log`");

    if !git_log_output.status.success() {
        bail!("`git log` failed: {}", git_log_output.status);
    }

    let logs = String::from_utf8(git_log_output.stdout)?;
    let mut commits = Vec::new();
    let mut current_commit = None;

    for line in logs.lines() {
        if current_commit.is_none() {
            current_commit = Some(line.to_string());
        } else {
            let current_date = Some(line.to_string());

            // We have both hash and date, create CommitDateUpdate
            let original_date = DateTime::parse_from_rfc3339(&current_date.unwrap())
                .with_context(|err| format!("Failed to parse original date: {}", err))?
                .with_timezone(&Utc);

            let commit_update = CommitDateUpdate {
                commit_hash: current_commit.unwrap(),
                original_date,
                new_date: new_date.and_utc().into(),
            };
            debug!(
                "Update {} from {} to {} ",
                commit_update.commit_hash, commit_update.original_date, commit_update.new_date
            );

            commits.push(commit_update);

            current_commit = None;
        }
    }

    if commits.len() != count {
        bail!("Expected {} commits but found {}", count, commits.len());
    }

    //    let mut lines = logs.lines();
    //    let commit_hash = lines.next().ok_or_else(|| format_err!("No commit found"))?;
    //    let original_date_str = lines.next().ok_or_else(|| format_err!("No date found"))?;
    //
    //    let original_date = DateTime::parse_from_rfc3339(original_date_str)
    //        .with_context(|err| format!("Failed to parse original date: {}", err))?
    //        .with_timezone(&Utc);

    // Start interactive rebase
    let range = format!("HEAD~{}", count);
    Command::new("git")
        .args(["rebase", "-i", &range, "--committer-date-is-author-date"])
        .env("GIT_SEQUENCE_EDITOR", "sed -i 's/^pick/edit/'")
        .output()
        .with_context(|_| "Failed to start rebase")?;

    // For each commit in rebase
    for commit in commits.iter() {
        Command::new("git")
            .args([
                "commit",
                "--amend",
                "--no-edit",
                &format!("--date={}", commit.new_date),
            ])
            .env("GIT_COMMITTER_DATE", date)
            .output()
            .with_context(|_| "Failed to amend commit")?;

        // Continue to next commit
        Command::new("git")
            .args(["rebase", "--continue"])
            .output()
            .with_context(|_| "Failed to continue rebase")?;
    }

    // After all commits are processed...
    Command::new("git")
        .args(["push", "--force-with-lease", "origin", "HEAD"])
        .output()
        .with_context(|_| "Failed to force push changes")?;

    Ok(commits)
}
