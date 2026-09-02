use std::sync::Mutex;

use ntix_rs::package_manager::command_runner::CommandRunner;
use ntix_rs::package_manager::process_command_runner::ProcessCommandRunner;

#[tokio::test]
async fn run_output_captures_stdout() {
    let runner = ProcessCommandRunner;
    let out = runner.run_output("echo hello", false).await;
    assert_eq!(out, "hello");
}

#[tokio::test]
async fn run_output_with_combine_stderr() {
    let runner = ProcessCommandRunner;
    let out = runner.run_output("echo both", true).await;
    assert_eq!(out, "both");
}

#[tokio::test]
async fn run_streams_stdout_and_returns_zero() {
    let runner = ProcessCommandRunner;
    let captured = Mutex::new(Vec::new());
    let code = runner
        .run(
            "echo line1 & echo line2",
            Some(&|line: &str| captured.lock().unwrap().push(line.to_string())),
            None,
        )
        .await;
    assert_eq!(code, 0);
    let lines = captured.lock().unwrap().clone();
    assert!(lines.iter().any(|l| l.contains("line1")), "got {lines:?}");
    assert!(lines.iter().any(|l| l.contains("line2")), "got {lines:?}");
}

#[tokio::test]
async fn run_captures_stderr_via_callback() {
    let runner = ProcessCommandRunner;
    let captured = Mutex::new(Vec::new());
    let code = runner
        .run(
            "echo boom 1>&2",
            None,
            Some(&|line: &str| captured.lock().unwrap().push(line.to_string())),
        )
        .await;
    assert_eq!(code, 0);
    let lines = captured.lock().unwrap().clone();
    assert!(lines.iter().any(|l| l.contains("boom")), "got {lines:?}");
}

#[tokio::test]
async fn run_propagates_exit_code() {
    let runner = ProcessCommandRunner;
    let code = runner.run("exit 5", None, None).await;
    assert_eq!(code, 5);
}
