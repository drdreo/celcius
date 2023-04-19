use chrono::{Duration, Utc};
use console::{Emoji, style};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use quicli::prelude::*;
use reqwest::blocking::Client;
use serde::Deserialize;
use statrs::statistics::Statistics;

// use std::fs::write;

//#[derive(Debug, Deserialize)]
//struct User {
//    login: String,
//    avatar_url: String,
//}

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");


#[derive(Debug, Deserialize)]
pub struct PullRequest {
    number: f64,
    //    title: String,
    //    user: User,
    created_at: String,
    //    updated_at: String,
    //    closed_at: Option<String>,
    //    merged_at: Option<String>,
    //    draft: bool,
    //    additions: Option<f64>,
    //    deletions: f64,
    //    changed_files: f64,
    //    commits: f64,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestDetails {
    //    number: f64,
    title: String,
    //    user: User,
    //    created_at: String,
    //    updated_at: String,
    //    closed_at: Option<String>,
    //    merged_at: Option<String>,
    //    draft: bool,
    additions: Option<f64>,
    deletions: f64,
    changed_files: f64,
    commits: f64,
    comments: f64,
}

//#[derive(Debug, Deserialize)]
//struct PRFile {
//    sha: String,
//    filename: String,
//    status: String,
//    additions: f64,
//    deletions: f64,
//    changes: f64,
//}

fn get_pull_requests(
    repo: &String,
    owner: &String,
    days: u32,
) -> Result<Vec<PullRequest>, Box<dyn std::error::Error>> {
    let now = Utc::now();
    let date_limit = now - Duration::days(days.into());

    println!("{} {}Analysing PRs of the last {} days for {}/{}",
             style("[1/2]").bold().dim(),
             LOOKING_GLASS,
             days, owner, repo);

    let mut all_pull_requests: Vec<PullRequest> = Vec::new();
    let token = "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";
    let client = Client::new();

    let mut cur_page = 1;
    let page_limit = 10;

    loop {
        debug!("Getting {} PRs of page {}", page_limit, cur_page);

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state=closed&per_page={}&page={}",
            owner, repo, page_limit, cur_page
        );
        let res = client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "drdreo/pr-stats-cli")
            .send()?;

        if !res.status().is_success() {
            return Err(format!("API returned error status code: {}", res.status()).into());
        }

        // let response_text = res.text()?;
        // println!("{}", response_text);
        // let response_text = res.text()?;
        // write("prs.txt", &response_text)?;

        // let page_pull_requests = serde_json::from_str::<Vec<PullRequest>>(&response_text)?;
        let page_pull_requests = res.json::<Vec<PullRequest>>()?;
        let date_limit_found = page_pull_requests.iter().any(|pr| pr.created_at.parse::<chrono::DateTime<Utc>>().unwrap() < date_limit);
        all_pull_requests.extend(page_pull_requests);

        if date_limit_found {
            break;
        }

        cur_page += 1;
    }

    debug!("Found {} PRs", all_pull_requests.len());

    Ok(all_pull_requests)
}

pub fn get_pull_request_details(
    repo: &String,
    owner: &String,
    pr_number: f64,
) -> Result<PullRequestDetails, Box<dyn std::error::Error>> {
    debug!("Getting PR details for: {}", pr_number);

    let pr_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );
    let token = "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";
    let client = Client::new();

    let res = client
        .get(&pr_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "drdreo/pr-stats-cli")
        .send()?;

    if !res.status().is_success() {
        return Err(format!("Error fetching PR details: {}", res.status()).into());
    }

    //    let response_text = res.text()?;?
    //        write("pr.json", &response_text)?;
    //    let pr: PullRequestDetails = serde_json::from_str(&response_text)?;

    let pr = res.json::<PullRequestDetails>()?;

    // get PR files
    //    let files_url = pr_url + "/files";
    //    let files_res = client
    //        .get(&files_url)
    //        .header("Authorization", format!("Bearer {}", token))
    //        .header("Accept", "application/vnd.github+json")
    //        .header("User-Agent", "drdreo/pr-stats-cli")
    //        .send()?;
    //
    //    if !files_res.status().is_success() {
    //        return Err(format!("Error fetching PR files: {}", files_res.status()).into());
    //    }
    //
    //        write("pr.json", &response_text)?;
    //
    //
    ////    let pr_files = files_res.json::<Vec<PRFile>>()?;
    //
    //    let response_text = files_res.text()?;
    //    write("pr_files.json", &response_text)?;
    //
    //    let pr_files = serde_json::from_str::<Vec<PRFile>>(&response_text)?;
    //
    //
    //    pr.additions = pr_files.iter().map(|file| file.additions).sum();
    //    pr.deletions = pr_files.iter().map(|file| file.deletions).sum();
    ////    pr.changes = pr_files.iter().map(|file| file.changes).sum();

    Ok(pr)
}

fn extract_pull_request_stats(
    repo: String,
    owner: String,
    days: u32,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), Box<dyn std::error::Error>> {
    let pull_requests = get_pull_requests(&repo, &owner, days)?;

    let mut additions = Vec::new();
    let mut deletions = Vec::new();
    let mut changed_files = Vec::new();
    let mut commits = Vec::new();
    let mut comments = Vec::new();

    let amount_pull_request = pull_requests.len();

    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");


    println!("{} {}Fetching PR details",
             style("[2/2]").bold().dim(),
             LOOKING_GLASS);

    let multi_pb = MultiProgress::new();
    // let pb_title = ProgressBar::new(amount_pull_request.try_into().unwrap());
    // pb_title.set_style(spinner_style.clone());
    // pb_title.set_prefix("[PR]");
    let pb_title = multi_pb.add(ProgressBar::new(amount_pull_request.try_into().unwrap()));
    pb_title.set_style(spinner_style.clone());
    pb_title.set_prefix("[PR]");

    let pb = multi_pb.insert_after(&pb_title, ProgressBar::new(amount_pull_request.try_into().unwrap()));

    for pr in pull_requests {
        let pr_details = get_pull_request_details(&repo, &owner, pr.number)?;
        pb_title.set_message(format!("{0}", pr_details.title));
        pb_title.inc(1);
        pb.inc(1);

        additions.push(pr_details.additions.unwrap_or(0.0));
        deletions.push(pr_details.deletions);
        changed_files.push(pr_details.changed_files);
        commits.push(pr_details.commits);
        comments.push(pr_details.comments);
    }

    pb_title.finish_and_clear();
    pb.finish_and_clear();

    Ok((additions, deletions, changed_files, commits, comments))
}

pub fn print_pr_statistics(repo: String, owner: String, days: u32) -> Result<(), Box<dyn std::error::Error>> {
    let (additions, deletions, changed_files, commits, comments) =
        extract_pull_request_stats(repo, owner, days)?;
    let net_loc = additions
        .iter()
        .zip(deletions.iter())
        .map(|(add, del)| add - del)
        .collect();

    println!("{}", style("PULL REQUEST STATISTICS").bold());
    println!("{} {}", style("TOTAL PRs:").blue().bold(), additions.len());

    // TODO: total amount of authors
    // TODO: stats per author
    // TODO: link to PR
    print_stats("COMMITS", &commits);
    print_stats("CHANGED FILES", &changed_files);
    print_stats("NET LOC", &net_loc);
    print_stats("ADDITIONS", &additions);
    print_stats("DELETIONS", &deletions);
    print_stats("COMMENTS", &comments);
    Ok(())
}

fn print_stats(headline: &str, data: &Vec<f64>) {
    println!("\n____ {} _____", style(headline).green().bold());
    println!("Total: {}", style(data.iter().sum::<f64>()).bold());
    println!("Min: {}", style(data.min()).bold());
    println!("Max: {}", style(data.max()).bold());
    println!("Mean: {:.2}", style(data.mean()).bold());
    println!("Std. Dev: {:.2}", style(data.std_dev()).bold());
}
