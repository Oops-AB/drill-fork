use async_trait::async_trait;
use colored::*;
use serde_json::json;
use std::io::{Seek, SeekFrom, Write};
use std::process::{Command, Stdio};
use tempfile::tempfile;
use yaml_rust::Yaml;

use crate::actions::Runnable;
use crate::actions::{extract, extract_optional};
use crate::benchmark::{Context, Pool, Reports};
use crate::config::Config;
use crate::interpolator;

#[derive(Clone)]
pub struct Exec {
  name: String,
  command: String,
  pub assign: Option<String>,
  pub stdin: Option<String>,
}

impl Exec {
  pub fn is_that_you(item: &Yaml) -> bool {
    item["exec"].as_hash().is_some()
  }

  pub fn new(item: &Yaml, _with_item: Option<Yaml>) -> Exec {
    let name = extract(item, "name");
    let command = extract(&item["exec"], "command");
    let assign = extract_optional(item, "assign");
    let stdin = extract_optional(item, "stdin");

    Exec {
      name,
      command,
      assign,
      stdin,
    }
  }
}

#[async_trait]
impl Runnable for Exec {
  async fn execute(&self, context: &mut Context, _reports: &mut Reports, _pool: &Pool, config: &Config) {
    if !config.quiet {
      println!("{:width$} {}", self.name.green(), self.command.cyan().bold(), width = 25);
    }

    let final_command = interpolator::Interpolator::new(context).resolve(&self.command, !config.relaxed_interpolations);

    let args = vec!["bash", "-c", "--", final_command.as_str()];

    let mut file_for_stdin = Stdio::null();

    if let Some(ref key) = self.stdin {
      let interpolator = interpolator::Interpolator::new(context);
      let eval = format!("{{{{ {} }}}}", key);
      let cmd_input = interpolator.resolve(&eval, true);
      let mut f = tempfile().unwrap();
      if let Err(why) = f.write_all(cmd_input.as_bytes()) {
        panic!("couldn't write {}: {}", key, why);
      }
      if let Err(why) = f.seek(SeekFrom::Start(0)) {
        panic!("couldn't rewind file: {}: {}", key, why);
      }
      file_for_stdin = Stdio::from(f);
    }

    let execution = Command::new(args[0]).args(&args[1..]).stdin(file_for_stdin).output().expect("Couldn't run it");

    let output: String = String::from_utf8_lossy(&execution.stdout).into();
    let output = output.trim_end().to_string();

    if let Some(ref key) = self.assign {
      context.insert(key.to_owned(), json!(output));
    }
  }
}
