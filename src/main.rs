extern crate clap;
extern crate dirs;
extern crate handlebars;
extern crate run_script;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
use clap::{App, SubCommand, ArgMatches, Arg};
use handlebars::Handlebars;
use run_script::ScriptOptions;
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;

fn main() {
    let matches = App::new("tmust")
        .version("0.1")
        .about("Automate your tmux envionrments")
        .author("Chris Muthig")
        .subcommand(SubCommand::with_name("new")
                    .about("Add a new tmux project")
                    .arg(Arg::with_name("project")
                         .help("The name of the new tmux project")
                         .index(1)
                         .required(true)))
        .subcommand(SubCommand::with_name("start")
                    .about("Start a new new tmux session")
                    .arg(Arg::with_name("project")
                         .help("The name of the new tmux project")
                         .index(1)
                         .required(true)))
        .get_matches();

    match matches.subcommand() {
        ("new", Some(matches)) => new(matches),
        ("start", Some(matches)) => start(matches),
        _ => println!("No subcommand"),
    }
}

fn tmux(config: Config) -> i32 {
    let mut reg = Handlebars::new();

    let template_str = fs::read_to_string("start.sh.hbs").unwrap();
    let script = reg.render_template(&template_str, &json!(config)).unwrap();

    println!("{:?}", script);

    let mut options = ScriptOptions::new();
    options.capture_output = false;
    let args = vec![];

    let (code, _, _) = run_script::run(&script, &args, &options).unwrap();

    code
}

fn config_path() -> PathBuf {
    let mut config_dir = dirs::home_dir().expect("Unable to determine home directory");

    config_dir.push(Path::new(".tmust"));

    let path = config_dir.as_path();

    if !path.exists() {
        fs::create_dir(path).expect("Unable to create configuration directory");
    }

    if !path.is_dir() {
        panic!("Configuration path exists but is not a directory")
    }

    config_dir
}

fn config_file(project: &str) -> File {
    let mut config_dir = config_path();

    config_dir.push(project.to_owned() + ".yaml");

    if !config_dir.as_path().exists() {
        panic!("Config file does not exist for project. Try use \"start\" instead");
    }

    let f = File::open(config_dir.as_path()).expect("Unable to read configuration file");

    f
}


#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(default = "get_shell")]
    shell: String,
    name: String,
    root: String,
}

fn get_shell() -> String {
    env::var("SHELL").unwrap()
}

fn get_config(project: &str) -> Config {
    let config_file = config_file(project);

    let config: Config = serde_yaml::from_reader(config_file).expect("Unable to parse config file");

    config
}


fn new(matches: &ArgMatches) {
    let mut config_path = config_path();
    let project = matches.value_of("project").unwrap();

    println!("Starting new project: {:?}", matches.value_of("project").unwrap());

    config_path.push(project.to_owned() + ".yaml");

    if config_path.as_path().exists() {
        println!("The {:?} project already exists.", config_path.as_path());
        return;
    }

    let mut reg = Handlebars::new();

    let template_str = fs::read_to_string("project.yaml.hbs").expect("Unable to read template file");
    let config_content = reg.render_template(&template_str, &json!({"project": project})).expect("Unable to render template");

    let mut f = File::create(config_path.as_path()).expect("Unable to create new project file");

    f.write(config_content.as_bytes()).expect("Unable to write configuration template");
}

fn start(matches: &ArgMatches) {
    // TODO Find the configuration file from the configuration directory
    let project = matches.value_of("project").unwrap();
    let config = get_config(project);

    // TODO parse configuration file

    // TODO start session using configuration values
    println!("Starting {}...", project);
    tmux(config);
}
