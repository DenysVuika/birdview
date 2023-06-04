use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct FileTypeInspector {
    html: i64,
    markdown: i64,
    scss: i64,
    css: i64,
    ts: i64,
    js: i64,
    json: i64,
}

impl FileTypeInspector {
    pub fn new() -> Self {
        FileTypeInspector {
            html: 0,
            markdown: 0,
            scss: 0,
            css: 0,
            ts: 0,
            js: 0,
            json: 0,
        }
    }
}

impl Default for FileTypeInspector {
    fn default() -> Self {
        FileTypeInspector::new()
    }
}

impl FileInspector for FileTypeInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn inspect_file(
        &mut self,
        workspace: &Workspace,
        path: &Path,
        _output: &mut Map<String, Value>,
    ) {
        let workspace_path = path
            .strip_prefix(&workspace.working_dir)
            .unwrap()
            .display()
            .to_string();

        if workspace_path.ends_with(".html") {
            self.html += 1;
        } else if workspace_path.ends_with(".md") {
            self.markdown += 1;
        } else if workspace_path.ends_with(".scss") {
            self.scss += 1;
        } else if workspace_path.ends_with(".css") {
            self.css += 1;
        } else if workspace_path.ends_with(".ts") {
            self.ts += 1;
        } else if workspace_path.ends_with(".js") {
            self.js += 1;
        } else if workspace_path.ends_with(".json") {
            self.json += 1;
        }
    }

    fn finalize(&mut self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        let types = stats
            .entry("types")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        types.entry("html").or_insert(json!(self.html));
        types.entry("scss").or_insert(json!(self.scss));
        types.entry("css").or_insert(json!(self.css));
        types.entry("ts").or_insert(json!(self.ts));
        types.entry("js").or_insert(json!(self.js));
        types.entry("md").or_insert(json!(self.markdown));
        types.entry("json").or_insert(json!(self.json));

        println!("Project Files");
        println!(" ├── HTML: {}", self.html);
        println!(" ├── SCSS: {}", self.scss);
        println!(" ├── CSS: {}", self.css);
        println!(" ├── TypeScript: {}", self.ts);
        println!(" ├── JavaScript: {}", self.js);
        println!(" ├── JSON: {}", self.json);
        println!(" └── Markdown: {}", self.markdown);
    }
}
