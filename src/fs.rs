use walkdir::DirEntry;

pub fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}

pub fn is_excluded(entry: &DirEntry) -> bool {
    let exclude = vec!["nxcache", "node_modules"];
    entry
        .file_name()
        .to_str()
        .map(|s| exclude.contains(&s))
        .unwrap_or(false)
}
