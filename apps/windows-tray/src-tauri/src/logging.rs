use std::{fs, path::Path};

use anyhow::Context;
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};

pub fn init_logging(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let config = ConfigBuilder::new().set_time_format_rfc3339().build();
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![SimpleLogger::new(LevelFilter::Info, config.clone())];
    if let Some(file) = open_log_file(path)? {
        loggers.push(WriteLogger::new(LevelFilter::Info, config, file));
    }
    let _ = CombinedLogger::init(loggers);
    Ok(())
}

fn open_log_file(path: &Path) -> anyhow::Result<Option<fs::File>> {
    match fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(file) => Ok(Some(file)),
        Err(primary_error) => {
            let fallback = path.with_file_name(format!("app-{}.log", std::process::id()));
            match fs::OpenOptions::new().create(true).append(true).open(&fallback) {
                Ok(file) => Ok(Some(file)),
                Err(fallback_error) => {
                    eprintln!(
                        "failed to open log files {} ({}) and {} ({})",
                        path.display(),
                        primary_error,
                        fallback.display(),
                        fallback_error
                    );
                    Ok(None)
                }
            }
        }
    }
}
