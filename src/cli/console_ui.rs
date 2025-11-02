use std::collections::VecDeque;
use std::io::{self, IsTerminal, Write};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use anyhow::{Result, anyhow};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use flume::{Receiver, Sender};
use parking_lot::Mutex;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

use crate::engine::{ConsoleSummarySink, ConsoleSummaryUpdate};

const MAX_BUFFERED_LOG_LINES: usize = 500;

enum ConsoleUiEvent {
    LogLine(String),
    Summary(ConsoleSummaryUpdate),
    Shutdown,
}

pub struct ConsoleUi {
    event_tx: Sender<ConsoleUiEvent>,
    join_handle: Mutex<Option<JoinHandle<()>>>,
}

impl ConsoleUi {
    pub fn start() -> Result<Self> {
        if !io::stdout().is_terminal() {
            return Err(anyhow!("终端不支持控制台摘要面板"));
        }

        let (event_tx, event_rx) = flume::unbounded::<ConsoleUiEvent>();
        let join_handle = thread::Builder::new()
            .name("console-ui".to_string())
            .spawn(move || {
                if let Err(err) = run_ui_loop(event_rx) {
                    eprintln!("console ui terminated: {err}");
                }
            })
            .map_err(|err| anyhow!("启动控制台面板线程失败: {err}"))?;

        Ok(Self {
            event_tx,
            join_handle: Mutex::new(Some(join_handle)),
        })
    }

    pub fn make_writer(&self) -> BoxMakeWriter {
        let tx = self.event_tx.clone();
        BoxMakeWriter::new(move || ConsoleUiLogWriter::new(tx.clone()))
    }

    pub fn summary_sink(&self) -> Arc<dyn ConsoleSummarySink> {
        Arc::new(ConsoleUiSummarySink {
            tx: self.event_tx.clone(),
        })
    }
}

impl Drop for ConsoleUi {
    fn drop(&mut self) {
        if self.event_tx.send(ConsoleUiEvent::Shutdown).is_ok() {
            if let Some(handle) = self.join_handle.lock().take() {
                let _ = handle.join();
            }
        }
    }
}

struct ConsoleUiSummarySink {
    tx: Sender<ConsoleUiEvent>,
}

impl ConsoleSummarySink for ConsoleUiSummarySink {
    fn publish(&self, update: ConsoleSummaryUpdate) {
        let _ = self.tx.send(ConsoleUiEvent::Summary(update));
    }
}

struct ConsoleUiLogWriter {
    tx: Sender<ConsoleUiEvent>,
    pending: String,
}

impl ConsoleUiLogWriter {
    fn new(tx: Sender<ConsoleUiEvent>) -> Self {
        Self {
            tx,
            pending: String::new(),
        }
    }

    fn flush_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let message = self.pending.drain(..).collect();
        let _ = self.tx.send(ConsoleUiEvent::LogLine(message));
    }
}

impl Write for ConsoleUiLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let chunk = String::from_utf8_lossy(buf);
        self.pending.push_str(&chunk);
        while let Some(pos) = self.pending.find('\n') {
            let mut line = self.pending.drain(..=pos).collect::<String>();
            if line.ends_with('\n') {
                line.pop();
            }
            if line.ends_with('\r') {
                line.pop();
            }
            let _ = self.tx.send(ConsoleUiEvent::LogLine(line));
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_pending();
        Ok(())
    }
}

impl Drop for ConsoleUiLogWriter {
    fn drop(&mut self) {
        self.flush_pending();
    }
}

fn run_ui_loop(event_rx: Receiver<ConsoleUiEvent>) -> Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    if let Err(err) = execute_initialise(&mut stdout) {
        let _ = terminal::disable_raw_mode();
        return Err(err);
    }

    let mut logs = VecDeque::with_capacity(MAX_BUFFERED_LOG_LINES);
    let mut summary_line = String::from("等待交易批次…");
    let mut loop_result: Result<()> = Ok(());

    if let Err(err) = render(&mut stdout, &logs, &summary_line) {
        let _ = cleanup_terminal(&mut stdout);
        return Err(err);
    }

    while let Ok(event) = event_rx.recv() {
        match event {
            ConsoleUiEvent::LogLine(line) => {
                if !line.is_empty() {
                    logs.push_back(line);
                    if logs.len() > MAX_BUFFERED_LOG_LINES {
                        logs.pop_front();
                    }
                }
            }
            ConsoleUiEvent::Summary(update) => {
                summary_line = update.line;
            }
            ConsoleUiEvent::Shutdown => break,
        }

        if let Err(err) = render(&mut stdout, &logs, &summary_line) {
            loop_result = Err(err);
            break;
        }
    }

    let cleanup_result = cleanup_terminal(&mut stdout);
    match (loop_result, cleanup_result) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(err), Ok(_)) => Err(err),
        (Ok(_), Err(err)) => Err(err),
        (Err(primary), Err(cleanup_err)) => {
            Err(primary.context(format!("cleanup failed: {cleanup_err}")))
        }
    }
}

fn execute_initialise(stdout: &mut io::Stdout) -> Result<()> {
    use crossterm::execute;
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        Clear(ClearType::All),
        MoveTo(0, 0)
    )?;
    stdout.flush()?;
    Ok(())
}

fn cleanup_terminal(stdout: &mut io::Stdout) -> Result<()> {
    use crossterm::execute;
    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    stdout.flush()?;
    Ok(())
}

fn render(stdout: &mut io::Stdout, logs: &VecDeque<String>, summary: &str) -> Result<()> {
    let (width, height) = terminal::size()?;
    if height == 0 || width == 0 {
        return Ok(());
    }
    let available_rows = height.saturating_sub(1) as usize;

    queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

    let start_index = logs.len().saturating_sub(available_rows);
    for (idx, line) in logs.iter().skip(start_index).enumerate() {
        queue!(stdout, MoveTo(0, idx as u16), Print(line))?;
    }

    let summary_row = height - 1;
    queue!(
        stdout,
        MoveTo(0, summary_row),
        Clear(ClearType::CurrentLine),
        Print(summary)
    )?;

    stdout.flush()?;
    Ok(())
}
