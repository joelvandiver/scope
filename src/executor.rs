use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;

use crate::cli::Args;

pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

pub async fn run_command(args: &Args) -> ExecResult {
    let mut cmd = if args.exec {
        let mut c = Command::new(&args.command[0]);
        c.args(&args.command[1..]);
        c
    } else {
        let shell_cmd = args.command.join(" ");
        let mut c = Command::new("sh");
        c.args(["-c", &shell_cmd]);
        c
    };

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    match cmd.output().await {
        Ok(output) => ExecResult {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code(),
        },
        Err(e) => ExecResult {
            stdout: String::new(),
            stderr: format!("scope: error running command: {e}"),
            exit_code: None,
        },
    }
}

pub async fn run_loop(args: Args, mut on_result: impl FnMut(ExecResult)) {
    let interval = Duration::from_secs_f64(args.interval);

    loop {
        let start = Instant::now();
        let result = run_command(&args).await;
        on_result(result);

        let sleep_duration = if args.precise {
            interval.saturating_sub(start.elapsed())
        } else {
            interval
        };

        tokio::time::sleep(sleep_duration).await;
    }
}
