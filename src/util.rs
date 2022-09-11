use std::{
    ffi::OsStr,
    path::PathBuf,
};

use chrono::{Duration, Local};
use crossterm::execute;

use crate::CrossTerminal;

pub fn fail<T, S: AsRef<str>>(msg: S) -> T {
    eprintln!("{}", msg.as_ref());
    std::process::exit(1);
}

pub fn init_logging() -> std::io::Result<()> {
    if let Ok(log_file) = std::env::var("LOG_FILE") {
        let path = PathBuf::from(log_file.clone());
        if path.exists() {
            if !path.is_file() {
                fail::<(), &str>("log file is not a file");
            }
            std::fs::remove_file(path)?;
        }
        let config = simple_log::LogConfigBuilder::builder()
            .path(log_file)
            .time_format("")
            .output_file()
            .build();
        simple_log::new(config).unwrap_or_else(|_| fail("couldn't set up log file"));
    }
    Ok(())
}

pub fn open_editor<I, S>(
    editor: &OsStr,
    args: I,
    terminal: &mut CrossTerminal,
) -> std::io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    std::process::Command::new(editor).args(args).status()?;
    execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}

pub fn format_date(offset: i64) -> String {
    let now = Local::now() + Duration::days(offset);
    format!("{}", now.format("%Y-%m-%d"))
}