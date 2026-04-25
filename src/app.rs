use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::diff::DiffLine;

#[derive(Debug)]
pub struct AppState {
    pub command: String,
    pub interval: f64,
    pub hostname: String,
    pub run_count: u64,
    pub last_run: Option<SystemTime>,
    pub exit_code: Option<i32>,
    pub current_output: Vec<String>,
    pub diff_lines: Vec<DiffLine>,
    pub error: Option<String>,
    pub scroll_offset: u16,
}

impl AppState {
    pub fn new(command: String, interval: f64, hostname: String) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            command,
            interval,
            hostname,
            run_count: 0,
            last_run: None,
            exit_code: None,
            current_output: Vec::new(),
            diff_lines: Vec::new(),
            error: None,
            scroll_offset: 0,
        }))
    }

    pub fn update(
        &mut self,
        output: Vec<String>,
        diff_lines: Vec<DiffLine>,
        exit_code: Option<i32>,
        error: Option<String>,
    ) {
        self.current_output = output;
        self.diff_lines = diff_lines;
        self.exit_code = exit_code;
        self.error = error;
        self.run_count += 1;
        self.last_run = Some(SystemTime::now());
    }
}
