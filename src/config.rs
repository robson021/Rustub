use crate::file_utils;
use log::LevelFilter;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde::Deserialize;
use std::error::Error;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
struct Config {
    server: ServerConfig,
}

pub(crate) fn read_config(path: &str) -> Result<ServerConfig, Box<dyn Error>> {
    let file = file_utils::open_file(path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_yaml::from_reader(reader)?;
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
