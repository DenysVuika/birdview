use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use git2::{Commit, Repository, RepositoryState};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct AuthorInfo {
    pub name: String,
    pub commits: i64,
}

pub struct GitProject {
    repository: Repository,
    working_dir: PathBuf,
}

impl GitProject {
    pub fn open(working_dir: &PathBuf) -> Result<Self> {
        let repository = Repository::open(working_dir)?;
        println!("state {:?}", repository.state());

        Ok(GitProject {
            repository,
            working_dir: working_dir.clone(),
        })
    }

    pub fn is_clean(&self) -> bool {
        self.repository.state() == RepositoryState::Clean
    }

    /// Get the remote url
    pub fn remote_url(&self) -> Result<String> {
        let remote = &self.repository.find_remote("origin")?;
        let url = remote.url().unwrap().to_owned();

        let remote_url = match url.strip_suffix(".git") {
            Some(value) => value.to_owned(),
            None => url,
        };

        Ok(remote_url)
    }

    /// Get the branch name
    pub fn branch(&self) -> Result<String> {
        let head = &self.repository.head()?;
        let branch = head.shorthand().unwrap().to_owned();
        Ok(branch)
    }

    /// Get the head sha
    pub fn sha(&self) -> Result<String> {
        let head = &self.repository.head()?;
        let sha = head.target().unwrap().to_string();
        Ok(sha)
    }

    pub fn authors(&self) -> Result<Vec<AuthorInfo>> {
        let repo = &self.repository;
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

    /// Checkout a branch (main), or a tag (v0.1.1) or a commit (8e8128)
    pub fn checkout(&self, ref_name: &String) -> Result<()> {
        log::info!("Checking out: {}", ref_name);
        if *ref_name == self.branch()? {
            log::info!("Branch {} already checked out", ref_name);
            return Ok(());
        }

        let repo = &self.repository;
        let (object, reference) = repo.revparse_ext(ref_name).expect("Object not found");

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

    pub fn tags(&self) -> Vec<String> {
        let mut tags = vec![];
        let tag_names = &self.repository.tag_names(None).unwrap();

        for name in tag_names.into_iter().flatten() {
            tags.push(name.to_owned());
            // let obj = repo.revparse_single(name)?;
            // if let Some(tag) = obj.as_tag() {
            //     println!("tag: {}, {}", tag.name().unwrap(), tag.message().unwrap())
            // } else if let Some(commit) = obj.as_commit() {
            //     println!("commit: {}, {}", name, commit.author().to_string())
            // }
        }
        tags
    }

    pub fn timestamp(&self) -> Result<DateTime<Utc>> {
        let head = &self.repository.head()?;
        let commit = head.peel_to_commit()?;
        // self.display_commit(&commit);

        let timestamp = Utc.timestamp_opt(commit.time().seconds(), 0).unwrap();
        Ok(timestamp)
    }

    pub fn display_commit(&self, commit: &Commit) {
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
}
