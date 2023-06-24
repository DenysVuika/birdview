BEGIN;

CREATE TABLE IF NOT EXISTS projects (
    name TEXT,
    created_on DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    origin TEXT
);

CREATE TABLE IF NOT EXISTS snapshots (
    pid INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_on DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    sha TEXT,
    timestamp DATETIME
);

CREATE TABLE IF NOT EXISTS tags (
    pid INTEGER NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS authors (
    sid INTEGER NOT NULL,
    name TEXT NOT NULL,
    commits INTEGER
);

CREATE TABLE IF NOT EXISTS ng_version (
    sid INTEGER NOT NULL,
    version TEXT
);

CREATE TABLE IF NOT EXISTS ng_entities (
    sid INTEGER NOT NULL,
    kind TEXT,
    path TEXT NOT NULL,
    url TEXT,
    standalone INTEGER
);

CREATE TABLE IF NOT EXISTS warnings (
    sid INTEGER NOT NULL,
    path TEXT NOT NULL,
    message TEXT NOT NULL,
    url TEXT
);

CREATE TABLE IF NOT EXISTS packages (
    sid INTEGER NOT NULL,
    path TEXT NOT NULL,
    url TEXT
);

CREATE TABLE IF NOT EXISTS dependencies (
    sid INTEGER NOT NULL,
    package_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    dev INTEGER
);

CREATE TABLE IF NOT EXISTS file_types (
    sid INTEGER NOT NULL,
    name TEXT NOT NULL,
    count INTEGER
);

CREATE TABLE IF NOT EXISTS tests (
    sid INTEGER NOT NULL,
    path TEXT NOT NULL,
    url TEXT,
    kind TEXT
);

CREATE TABLE IF NOT EXISTS test_cases (
    test_id INTEGER NOT NULL,
    name TEXT NOT NULL
);

COMMIT;
