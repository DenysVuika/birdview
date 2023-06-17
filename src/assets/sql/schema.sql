BEGIN;
CREATE TABLE IF NOT EXISTS projects (name TEXT, version TEXT, created_on TEXT NOT NULL, origin TEXT);
CREATE TABLE IF NOT EXISTS ng_version (project_id INTEGER NOT NULL, version TEXT);
CREATE TABLE IF NOT EXISTS ng_entities (project_id INTEGER NOT NULL, kind TEXT, path TEXT NOT NULL, url TEXT, standalone INTEGER);
CREATE TABLE IF NOT EXISTS warnings (project_id INTEGER NOT NULL, path TEXT NOT NULL, message TEXT NOT NULL, url TEXT);
CREATE TABLE IF NOT EXISTS packages (project_id INTEGER NOT NULL, path TEXT NOT NULL, url TEXT);
CREATE TABLE IF NOT EXISTS dependencies (project_id INTEGER NOT NULL, package_id INTEGER NOT NULL, name TEXT NOT NULL, version TEXT NOT NULL, dev INTEGER);
CREATE TABLE IF NOT EXISTS file_types (project_id INTEGER NOT NULL, name TEXT NOT NULL, count INTEGER);
CREATE TABLE IF NOT EXISTS tests (project_id INTEGER NOT NULL, path TEXT NOT NULL, url TEXT, kind TEXT);
CREATE TABLE IF NOT EXISTS test_cases (test_id INTEGER NOT NULL, name TEXT NOT NULL);
COMMIT;
