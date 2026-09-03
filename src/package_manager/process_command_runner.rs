use std::process::Stdio;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::package_manager::command_runner::{CREATE_NO_WINDOW, CommandRunner, LineCallback};

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
            stream_lines(Some(stdout), on_output),
            stream_lines(Some(stderr), on_error)
        );

        match child.wait().await {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        }
    }

    async fn run_output(&self, command: &str, combine_stderr: bool) -> String {
        let mut child = match Command::new("cmd.exe")
            .arg("/c")
            .raw_arg(command)
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return String::new(),
        };

        let stdout = child.stdout.take().expect("stdout was piped");
        let stderr = child.stderr.take().expect("stderr was piped");

        let (mut output, mut errout) = (String::new(), String::new());
        use tokio::io::AsyncReadExt;
        tokio::join!(
            async {
                let mut reader = BufReader::new(stdout);
                let _ = reader.read_to_string(&mut output).await;
            },
            async {
                let mut reader = BufReader::new(stderr);
                let _ = reader.read_to_string(&mut errout).await;
            },
        );

        let _ = child.wait().await;

        if combine_stderr {
            output.push_str(&errout);
        }
        output.trim().to_string()
    }
}

pub(crate) async fn stream_lines<R>(reader: Option<R>, callback: Option<LineCallback<'_>>)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let Some(reader) = reader else {
        return;
    };
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(cb) = callback {
            cb(&line);
        }
    }
}
