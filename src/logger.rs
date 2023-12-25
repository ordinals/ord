use anyhow::Context;
use log4rs::{
  append::{
    console::ConsoleAppender,
    rolling_file::{
      policy::compound::{
        roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
      },
      RollingFileAppender,
    },
  },
  config::{Appender, Logger, Root},
  encode::pattern::PatternEncoder,
  Config,
};
use std::fs;
use std::path::Path;

pub fn init<P: AsRef<Path>>(level: log::LevelFilter, log_dir: P) -> anyhow::Result<log4rs::Handle> {
  fs::create_dir_all(&log_dir)?;
  let log_file = log_dir.as_ref().join("ord.log");

  let stdout = ConsoleAppender::builder().build();

  // using default encoder for now, change it as needed.
  let encoder = PatternEncoder::default();
  let trigger = SizeTrigger::new(1024 * 1024 * 20);
  let roller = FixedWindowRoller::builder()
    .build(
      log_dir
        .as_ref()
        .join("ord-{}.log.gz")
        .to_string_lossy()
        .as_ref(),
      50,
    )
    .map_err(|e| anyhow::format_err!("build FixedWindowRoller error: {}", e))?;
  let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));
  let rfile = RollingFileAppender::builder()
    .append(true)
    .encoder(Box::new(encoder))
    .build(&log_file, Box::new(policy))
    .with_context(|| format!("Failed to create rolling file {}", log_file.display()))?;

  let cfg = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .appender(Appender::builder().build("rfile", Box::new(rfile)))
    .logger(Logger::builder().build("mio", log::LevelFilter::Error))
    .build(
      Root::builder()
        .appender("stdout")
        .appender("rfile")
        .build(level),
    )
    .context("build log config failed")?;

  log4rs::init_config(cfg).context("log4rs init config error")
}
