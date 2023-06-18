use anyhow::Result;
use git2::Repository;
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
    pub authors: Vec<AuthorInfo>,
}

// pub fn get_repository_info(path: &PathBuf) -> Option<RepositoryInfo> {
//     match Repository::open(path) {
//         Err(..) => println!("Git repository not found"),
//         Ok(repo) => match repo.head() {
//             Err(..) => println!("Head not found"),
//             Ok(head) => match repo.find_remote("origin") {
//                 Err(..) => println!("Warning: origin remote not found"),
//                 Ok(remote) => {
//                     if let Some(url) = remote.url() {
//                         let remote_url = match url.strip_suffix(".git") {
//                             Some(value) => value,
//                             None => url,
//                         };
//
//                         let authors = get_commit_authors(&repo).unwrap();
//
//                         return Some(RepositoryInfo {
//                             remote_url: remote_url.to_owned(),
//                             branch: head.shorthand().unwrap().to_owned(),
//                             sha: head.target().unwrap().to_string(),
//                             authors,
//                         });
//                     }
//                 }
//             },
//         },
//     }
//
//     None
// }

pub fn get_repository_info(path: &PathBuf) -> Result<RepositoryInfo> {
    let repo = Repository::open(path)?;
    let head = repo.head()?;
    let remote = repo.find_remote("origin")?;
    let url = remote.url().unwrap();

    let remote_url = match url.strip_suffix(".git") {
        Some(value) => value,
        None => url,
    };

    let authors = get_commit_authors(&repo).unwrap();

    Ok(RepositoryInfo {
        remote_url: remote_url.to_owned(),
        branch: head.shorthand().unwrap().to_owned(),
        sha: head.target().unwrap().to_string(),
        authors,
    })
}

pub fn get_commit_authors(repo: &Repository) -> Result<Vec<AuthorInfo>> {
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
                println!("Error walking the revisions {}, skipping", e);
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
