use crate::file_utils;
use crate::model::server_config::ServerConfig;
use crate::model::stub_response::*;
use anyhow::Result;
use log::{LevelFilter, debug, info, warn};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
}

pub(crate) fn read_stubs_config(path: &str) -> Result<Vec<StubResponse>> {
    let files = file_utils::list_all_files_with_suffix(path, "-stub.json")?;
    info!("Found {} stub files: {:?}", files.len(), files);
    let mut stubs = vec![];
    for path in files {
        let path_buf = path.to_str();
        if path_buf.is_none() {
            warn!("Skipping invalid stub file path: {:?}", path);
            continue;
        }
        let path = path_buf.unwrap();
        debug!("Reading stub file: {}", path);
        let file = file_utils::open_file(path)?;
        let stub: Vec<StubResponse> = serde_json::from_reader(file)?;
        stubs.push(stub);
    }
    let stubs = stubs.into_iter().flatten().collect();
    Ok(stubs)
}

pub(crate) fn read_server_config(path: &str) -> Result<ServerConfig> {
    let file = file_utils::open_file(path)?;
    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config.server)
}

pub(crate) fn setup_logger() {
    const LOG_PATTERN: &str = "{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n})}";
    let stderr = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
        .target(Target::Stderr)
        .build();

    let error_trace_level = Root::builder().appender("stderr").build(LevelFilter::Trace);

    let console_cfg = log4rs::Config::builder()
        .appender(Appender::builder().build("stderr", Box::new(stderr)))
        .build(error_trace_level)
        .unwrap();

    log4rs::init_config(console_cfg).unwrap();
    log::set_max_level(if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    });
}
