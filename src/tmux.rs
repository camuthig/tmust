extern crate handlebars;
extern crate run_script;
extern crate serde;

use handlebars::Handlebars;
use run_script::ScriptOptions;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::env;
use std::fs;

fn get_shell() -> String {
    env::var("SHELL").unwrap()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "get_shell")]
    shell: String,
    name: String,
    root: String,
}

pub fn run(config: Config) -> i32 {
    let reg = Handlebars::new();

    let template_str = fs::read_to_string("start.sh.hbs").unwrap();
    let script = reg.render_template(&template_str, &json!(config)).unwrap();

    println!("{:?}", script);

    let mut options = ScriptOptions::new();
    options.capture_output = false;
    let args = vec![];

    let (code, _, _) = run_script::run(&script, &args, &options).unwrap();

    code
}
