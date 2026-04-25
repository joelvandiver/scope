use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::app::AppState;
use crate::cli::Args;
use crate::diff;

async fn run_command(args: &Args) -> (String, Option<i32>, Option<String>) {
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
        Ok(output) => {
            let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if !stderr.is_empty() {
                combined.push_str(&stderr);
            }
            (combined, output.status.code(), None)
        }
        Err(e) => (
            String::new(),
            None,
            Some(format!("scope: error running command: {e}")),
        ),
    }
}

pub async fn run_loop(args: Args, state: Arc<Mutex<AppState>>, cancel: CancellationToken) {
    let interval = Duration::from_secs_f64(args.interval);
    let mut previous_output = String::new();

    loop {
        let start = Instant::now();
        let (output, exit_code, error) = run_command(&args).await;

        let diff_lines = diff::compute(&previous_output, &output);
        let lines: Vec<String> = output.lines().map(str::to_string).collect();

        {
            let mut s = state.lock().unwrap();
            s.update(lines, diff_lines, exit_code, error);
        }

        previous_output = output;

        let sleep_duration = if args.precise {
            interval.saturating_sub(start.elapsed())
        } else {
            interval
        };

        tokio::select! {
            _ = tokio::time::sleep(sleep_duration) => {}
            _ = cancel.cancelled() => return,
        }
    }
}
