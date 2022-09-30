#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected},
  executable_path::executable_path,
  regex::Regex,
  std::{
    fs,
    process::{Command, Stdio},
    str,
  },
  tempfile::TempDir,
  unindent::Unindent,
};

mod command_builder;
mod epochs;
mod expected;
mod find;
mod index;
mod info;
mod list;
mod parse;
mod range;
mod supply;
mod traits;
mod version;
