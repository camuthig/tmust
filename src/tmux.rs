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
    pub name: String,
    root: String,
    startup_window: Option<String>,
    on_project_start: Option<String>,
    on_project_first_start: Option<String>,
    on_project_exit: Option<String>,
    on_project_stop: Option<String>,
    on_project_restart: Option<String>,
    pre_window: Option<String>,
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
    let reg = handlebars_registry();

    let template_str = include_str!("start.sh.hbs");
    let script = reg.render_template(&template_str, &json!(config)).unwrap();

    let mut options = ScriptOptions::new();
    options.capture_output = false;

    let args = vec![];

    let (code, _, _) = run_script::run(&script, &args, &options).unwrap();

    code
}

pub fn stop(config: Config) -> i32 {
    let reg = handlebars_registry();

    let template_str = include_str!("stop.sh.hbs");
    let script = reg.render_template(&template_str, &json!(config)).unwrap();

    let mut options = ScriptOptions::new();
    options.capture_output = false;

    let args = vec![];

    let (code, _, _) = run_script::run(&script, &args, &options).unwrap();

    code
}

pub fn has_session(project: &String) -> bool {
    let output = Command::new("tmux")
        .arg("list-sessions")
        .output()
        .unwrap();

    let r = Regex::new(&format!("^{}:", project).to_string()).unwrap();

    r.is_match(&String::from_utf8(output.stdout).unwrap())
}

pub fn handlebars_registry() -> Handlebars {
    let mut h = Handlebars::new();

    h.register_helper("has_session", Box::new(has_session_helper));

    return h;
}

handlebars_helper!(has_session_helper: |name: str| has_session(&name.to_string()));
