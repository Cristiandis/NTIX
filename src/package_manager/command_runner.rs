use async_trait::async_trait;

pub type LineCallback<'a> = &'a (dyn Fn(&str) + Sync);

#[async_trait]
pub trait CommandRunner {
    async fn run(
        &self,
        command: &str,
        on_output: Option<LineCallback<'_>>,
        on_error: Option<LineCallback<'_>>,
    ) -> i32;

    async fn run_output(&self, command: &str, combine_stderr: bool) -> String;
}
