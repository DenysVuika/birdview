use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use git2::{Commit, Repository};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct AuthorInfo {
    pub name: String,
    pub commits: i64,
}

#[derive(Debug, Serialize)]
pub struct RepositoryInfo {
    pub remote_url: String,
    pub branch: String,
    pub sha: String,
    pub tags: Vec<String>,
    /// last commit timestamp
    pub timestamp: DateTime<Utc>,
}

pub fn checkout_branch(path: &PathBuf, branch: &str) -> Result<()> {
    let repo = Repository::open(path)?;
    let refname = branch; // or a tag (v0.1.1) or a commit (8e8128)
    let (object, reference) = repo.revparse_ext(refname).expect("Object not found");

    repo.checkout_tree(&object, None)
        .expect("Failed to checkout");

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");
    Ok(())
}

pub fn get_repository_info(path: &PathBuf) -> Result<RepositoryInfo> {
    let repo = Repository::open(path)?;
    let head = repo.head()?;
    let remote = repo.find_remote("origin")?;
    let url = remote.url().unwrap();

    let remote_url = match url.strip_suffix(".git") {
        Some(value) => value,
        None => url,
    };

    let mut tags = vec![];
    let tag_names = &repo.tag_names(None)?;
    for name in tag_names.into_iter().flatten() {
        tags.push(name.to_owned());
    }

    let commit = head.peel_to_commit()?;
    display_commit(&commit);

    let timestamp = Utc.timestamp_opt(commit.time().seconds(), 0).unwrap();

    Ok(RepositoryInfo {
        remote_url: remote_url.to_owned(),
        branch: head.shorthand().unwrap().to_owned(),
        sha: head.target().unwrap().to_string(),
        tags,
        timestamp,
    })
}

fn display_commit(commit: &Commit) {
    let timestamp = commit.time().seconds();
    let tm = Utc.timestamp_opt(timestamp, 0).unwrap();

    log::info!(
        "commit {}\nAuthor: {}\nDate:   {}\n\n    {}",
        commit.id(),
        commit.author(),
        tm,
        commit.message().unwrap_or("no commit message")
    );
}

pub fn get_repository_authors(path: &PathBuf) -> Result<Vec<AuthorInfo>> {
    let repo = Repository::open(path)?;
    let authors = get_commit_authors(&repo)?;
    Ok(authors)
}

fn get_commit_authors(repo: &Repository) -> Result<Vec<AuthorInfo>> {
    let mut rev_walker = repo.revwalk()?;
    rev_walker.push_head()?;

    let mut authors: Vec<AuthorInfo> = rev_walker
        .map(|r| {
            let oid = r?;
            repo.find_commit(oid)
        })
        .filter_map(|c| match c {
            Ok(commit) => Some(commit),
            Err(e) => {
                log::error!("Error walking the revisions {}, skipping", e);
                None
            }
        })
        .fold(
            HashMap::new(),
            |mut result: HashMap<String, AuthorInfo>, cur| {
                if let Some(name) = cur.author().name() {
                    let author_name = name.to_string();
                    let mut author = result.entry(author_name).or_insert(AuthorInfo {
                        name: name.to_string(),
                        commits: 0,
                    });
                    author.commits += 1;
                }
                result
            },
        )
        .into_values()
        .collect();

    authors.sort_by(|a, b| b.commits.cmp(&a.commits));
    Ok(authors)
}
