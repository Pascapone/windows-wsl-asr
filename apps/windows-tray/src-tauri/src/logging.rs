use std::{fs, path::Path};

use anyhow::Context;
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};

pub fn init_logging(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let config = ConfigBuilder::new().set_time_format_rfc3339().build();
    let file = fs::File::create(path).with_context(|| format!("failed to open {}", path.display()))?;
    let loggers: Vec<Box<dyn SharedLogger>> = vec![
        SimpleLogger::new(LevelFilter::Info, config.clone()),
        WriteLogger::new(LevelFilter::Info, config, file),
    ];
    let _ = CombinedLogger::init(loggers);
    Ok(())
}
