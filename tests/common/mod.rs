#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use ntix_rs::package_manager::command_runner::{CommandRunner, LineCallback};

type RunHandler = Box<dyn Fn(&str) -> i32 + Send + Sync>;

/// Returns a tag string that is unique across the whole process.
pub fn unique_tag(prefix: &str) -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "ntix_{}_pid{}_{}",
        prefix,
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

/// Hand-rolled mock of `CommandRunner`.
pub struct MockCommandRunner {
    pub captured_commands: Mutex<Vec<String>>,
    pub run_handler: Mutex<Option<RunHandler>>,
    pub output_responses: HashMap<String, String>,
}

impl MockCommandRunner {
    pub fn new() -> Self {
        Self {
            captured_commands: Mutex::new(Vec::new()),
            run_handler: Mutex::new(None),
            output_responses: HashMap::new(),
        }
    }

    pub fn commands(&self) -> Vec<String> {
        self.captured_commands.lock().unwrap().clone()
    }
}

impl Default for MockCommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandRunner for MockCommandRunner {
    async fn run(
        &self,
        command: &str,
        _on_output: Option<LineCallback<'_>>,
        _on_error: Option<LineCallback<'_>>,
    ) -> i32 {
        self.captured_commands
            .lock()
            .unwrap()
            .push(command.to_string());

        if let Some(handler) = self.run_handler.lock().unwrap().as_ref() {
            return handler(command);
        }

        0
    }

    async fn run_output(&self, command: &str, _combine_stderr: bool) -> String {
        self.captured_commands
            .lock()
            .unwrap()
            .push(command.to_string());

        if let Some(output) = self.output_responses.get(command) {
            return output.clone();
        }

        for (key, output) in &self.output_responses {
            if command.contains(key) {
                return output.clone();
            }
        }

        String::new()
    }
}

pub fn winget_list_table(rows: &[(&str, &str, Option<&str>)]) -> String {
    let name_w = rows.iter().map(|r| r.0.len()).chain([4]).max().unwrap();
    let id_w = rows.iter().map(|r| r.0.len()).chain([2]).max().unwrap();
    let ver_w = rows.iter().map(|r| r.1.len()).chain([7]).max().unwrap();
    let avail_w = rows
        .iter()
        .map(|r| r.2.map(|a| a.len()).unwrap_or(0))
        .chain([9])
        .max()
        .unwrap();
    let source_w = 6;

    let header = format!(
        "{:<name_w$}  {:<id_w$}  {:<ver_w$}  {:<avail_w$}  {:<source_w$}",
        "Name", "Id", "Version", "Available", "Source"
    );
    let separator = format!(
        "{}  {}  {}  {}  {}",
        "-".repeat(name_w),
        "-".repeat(id_w),
        "-".repeat(ver_w),
        "-".repeat(avail_w),
        "-".repeat(source_w)
    );

    let mut lines = vec![header, separator];
    for (id, ver, avail) in rows {
        lines.push(format!(
            "{:<name_w$}  {:<id_w$}  {:<ver_w$}  {:<avail_w$}  {:<source_w$}",
            id,
            id,
            ver,
            avail.unwrap_or(""),
            "winget"
        ));
    }
    lines.join("\r\n") + "\r\n"
}

/// The exact command string `winget_ops` runs to discover installed packages.
pub fn winget_list_command() -> String {
    "winget list --accept-source-agreements".to_string()
}

/// The exact command string `winget_ops` runs to check whether a package
/// exists in the winget source.
pub fn winget_search_command(id: &str) -> String {
    format!("winget search --id {id} --exact --accept-source-agreements")
}
