extern crate handlebars;
extern crate regex;
extern crate run_script;
extern crate serde;

use handlebars::Handlebars;
use regex::Regex;
use run_script::ScriptOptions;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::env;
use std::process::Command;

fn get_shell() -> String {
    env::var("SHELL").unwrap()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "get_shell")]
    shell: String,
    name: String,
    root: String,
    #[serde(default = "Vec::new")]
    windows: Vec<Window>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Window {
    name: String,
    #[serde(default = "Vec::new")]
    commands: Vec<String>,
}

// TODO bring back in pane control
//#[derive(Serialize, Deserialize, Debug)]
//pub struct Pane {
//    name: String,
//    #[serde(default = "Vec::new")]
//    commands: Vec<String>,
//}

pub fn start(config: Config) -> i32 {
    let reg = Handlebars::new();

    let template_str = include_str!("start.sh.hbs");
    let script = reg.render_template(&template_str, &json!(config)).unwrap();

    let mut options = ScriptOptions::new();
    options.capture_output = false;

    let args = vec![];

    let (code, _, _) = run_script::run(&script, &args, &options).unwrap();

    code
}

pub fn attach(project: String) -> i32 {
    let status = Command::new("tmux")
        .args(&["attach", "-t", &project])
        .status()
        .unwrap();

    status.code().unwrap_or(1)
}

pub fn has_session(project: String) -> bool {
    let output = Command::new("tmux")
        .arg("list-sessions")
        .output()
        .unwrap();

    let r = Regex::new(&format!("^{}:", project).to_string()).unwrap();

    r.is_match(&String::from_utf8(output.stdout).unwrap())
}
