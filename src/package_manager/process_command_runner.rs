use std::process::Stdio;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::package_manager::command_runner::{CommandRunner, LineCallback};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct ProcessCommandRunner;

#[async_trait]
impl CommandRunner for ProcessCommandRunner {
    async fn run(
        &self,
        command: &str,
        on_output: Option<LineCallback<'_>>,
        on_error: Option<LineCallback<'_>>,
    ) -> i32 {
        let mut child = match Command::new("cmd.exe")
            .arg("/c")
            .raw_arg(command)
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return -1,
        };

        let stdout = child.stdout.take().expect("stdout was piped");
        let stderr = child.stderr.take().expect("stderr was piped");

        tokio::join!(
            stream_lines(stdout, on_output),
            stream_lines(stderr, on_error)
        );

        match child.wait().await {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        }
    }

    async fn run_output(&self, command: &str, combine_stderr: bool) -> String {
        let redirected = if combine_stderr {
            format!("{command} 2>&1")
        } else {
            command.to_string()
        };

        let mut child = match Command::new("cmd.exe")
            .arg("/c")
            .raw_arg(&redirected)
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return String::new(),
        };

        let stdout = child.stdout.take().expect("stdout was piped");
        let mut output = String::new();
        let mut reader = BufReader::new(stdout);
        use tokio::io::AsyncReadExt;
        let _ = reader.read_to_string(&mut output).await;

        let _ = child.wait().await;

        output.trim().to_string()
    }
}

async fn stream_lines<R>(reader: R, callback: Option<&(dyn Fn(&str) + Sync)>)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(cb) = callback {
            cb(&line);
        }
    }
}
