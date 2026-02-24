use std::{
    collections::VecDeque,
    io::{Read, Write},
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize};

const OUTPUT_LIMIT: usize = 2000;
const PENDING_CHAR_LIMIT: usize = 8192;

pub(super) struct PaneProcess {
    master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    _child: Box<dyn portable_pty::Child + Send>,
    output: Arc<Mutex<VecDeque<String>>>,
}

impl PaneProcess {
    pub(super) fn spawn(command: String, size: PtySize) -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system.openpty(size)?;
        let mut cmd = CommandBuilder::new(&command);
        cmd.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let output = Arc::new(Mutex::new(VecDeque::new()));
        let output_clone = output.clone();

        thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            let mut pending = String::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]);
                        let stripped = strip_ansi_escapes::strip(chunk.as_bytes());
                        let text = String::from_utf8_lossy(&stripped).replace('\r', "");
                        let mut combined = String::new();
                        std::mem::swap(&mut combined, &mut pending);
                        combined.push_str(&text);
                        let mut lines: Vec<&str> = combined.split('\n').collect();
                        let last = lines.pop().unwrap_or("");
                        pending = last.to_string();
                        if pending.chars().count() > PENDING_CHAR_LIMIT {
                            pending = pending
                                .chars()
                                .rev()
                                .take(PENDING_CHAR_LIMIT)
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .collect();
                        }
                        let mut guard = lock_output(&output_clone);
                        for line in lines {
                            guard.push_back(line.to_string());
                            if guard.len() > OUTPUT_LIMIT {
                                guard.pop_front();
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            master: pair.master,
            writer,
            _child: child,
            output,
        })
    }

    pub(super) fn resize(&mut self, rows: u16, cols: u16) {
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }

    pub(super) fn write_bytes(&mut self, bytes: &[u8]) {
        let _ = self.writer.write_all(bytes);
        let _ = self.writer.flush();
    }

    pub(super) fn lines_for_height(&self, height: usize) -> Vec<String> {
        let guard = lock_output(&self.output);
        let total = guard.len();
        let start = total.saturating_sub(height);
        guard.iter().skip(start).cloned().collect()
    }
}

fn lock_output(output: &Mutex<VecDeque<String>>) -> MutexGuard<'_, VecDeque<String>> {
    match output.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
