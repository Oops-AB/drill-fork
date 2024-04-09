use async_trait::async_trait;
use colored::*;
use yaml_rust::Yaml;

use crate::actions::extract;
use crate::actions::Runnable;
use crate::benchmark::{Context, Pool, Reports};
use crate::config::Config;
use crate::interpolator;

#[derive(Clone)]
pub struct Debug {
  name: String,
  msg: String,
}

impl Debug {
  pub fn is_that_you(item: &Yaml) -> bool {
    item["debug"].as_hash().is_some()
  }

  pub fn new(item: &Yaml, _with_item: Option<Yaml>) -> Debug {
    let name = extract(item, "name");
    let msg = extract(&item["debug"], "msg");

    Debug {
      name,
      msg,
    }
  }
}

#[async_trait]
impl Runnable for Debug {
  async fn execute(&self, context: &mut Context, _reports: &mut Reports, _pool: &Pool, config: &Config) {
    if !config.quiet {
      println!("{:width$} {}", self.name.green(), self.msg.cyan().bold(), width = 25);
    }

    let interpolator = interpolator::Interpolator::new(context);
    let eval = format!("{{{{ {} }}}}", &self.msg);
    let stored = interpolator.resolve(&eval, false);

    println!("{}", stored.as_str());
  }
}
