extern crate clap;
extern crate dirs;
extern crate handlebars;
#[macro_use]
extern crate serde_json;
use clap::{App, SubCommand, ArgMatches, Arg};
use std::fs;
use std::fs::File;
use handlebars::Handlebars;
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
                    .about("Start a new new tmux session"))
        .get_matches();

    match matches.subcommand() {
        ("new", Some(matches)) => new(matches),
        ("start", Some(matches)) => start(matches),
        _ => println!("No subcommand"),
    }
}

fn tmux() -> ExitStatus {
    let mut command = Command::new("tmux");

    command.status().expect("Unable to start the tmux session")
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

    f.write(config_content.as_bytes());
}

fn start(matches: &ArgMatches) {
    // TODO Find the configuration file from the configuration directory
    // TODO parse configuration file
    // TODO start session using configuration values
    tmux();
}
