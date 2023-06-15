BEGIN;
CREATE TABLE IF NOT EXISTS projects (id BLOB PRIMARY KEY NOT NULL, name TEXT, version TEXT, created_on TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS angular (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, version TEXT);
CREATE TABLE IF NOT EXISTS ng_modules (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS ng_components (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL, standalone INTEGER);
CREATE TABLE IF NOT EXISTS ng_directives (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL, standalone INTEGER);
CREATE TABLE IF NOT EXISTS ng_services (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS ng_pipes (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL, standalone INTEGER);
CREATE TABLE IF NOT EXISTS ng_dialogs (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL, standalone INTEGER);
CREATE TABLE IF NOT EXISTS warnings (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL, message TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS packages (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS dependencies (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, package_id BLOB NOT NULL, name TEXT NOT NULL, version TEXT NOT NULL, dev INTEGER);
CREATE TABLE IF NOT EXISTS file_types (project_id BLOB NOT NULL, name TEXT NOT NULL, count INTEGER);
CREATE TABLE IF NOT EXISTS unit_tests (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS e2e_tests (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL, path TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS test_cases (test_id BLOB NOT NULL, name TEXT NOT NULL);
COMMIT;
