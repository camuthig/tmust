extern crate clap;
extern crate dirs;
extern crate handlebars;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use handlebars::Handlebars;
use quicli::prelude::*;
use serde_json::json;
use std::fs;
use std::fs::{File, ReadDir};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

mod tmux;
use tmux::Config;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "tmust",
    version = "0.1",
    about = "Automate your tmux environments",
    author = "Chris Muthig",
    raw(setting = "structopt::clap::AppSettings::SubcommandRequiredElseHelp")
)]
struct Tmust {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
struct New {
    #[structopt(index = 1, help = "The name of the tmux project")]
    project: String,
}

#[derive(Debug, StructOpt)]
struct Start {
    #[structopt(index = 1, help = "The name of the tmux project")]
    project: String,
}


#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "new", about = "Add a new tmux project")]
    New(New),

    #[structopt(name = "start", about = "Start a new tmux session")]
    Start(Start),

    #[structopt(name = "list", about = "List the configured projects")]
    List {
    },
}

fn main() -> CliResult {
    let tmust = Tmust::from_args();

    match &tmust.cmd {
        Command::New(c)=> new(&c)?,
        Command::Start(c) => start(&c)?,
        Command::List{..} => list()?,
    }

    Ok(())
}

fn config_path() -> PathBuf {
    let mut config_dir = dirs::home_dir().unwrap();

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

fn config_file(project: &String) -> File {
    let mut config_dir = config_path();

    config_dir.push(project.to_owned() + ".yaml");

    if !config_dir.as_path().exists() {
        panic!("Config file does not exist for project. Try use \"start\" instead");
    }

    let f = File::open(config_dir.as_path()).expect("Unable to read configuration file");

    f
}

fn get_config(project: &String) -> Config {
    let config_file = config_file(project);

    let config: Config = serde_yaml::from_reader(config_file).expect("Unable to parse config file");

    config
}


fn new(cmd: &New) -> Result<(), Error> {
    let mut config_path = config_path();

    println!("Creating new project: {}", cmd.project);

    config_path.push(cmd.project.to_owned() + ".yaml");

    if config_path.as_path().exists() {
        println!("The {} project already exists.", cmd.project);
        return Ok(());
    }

    let reg = Handlebars::new();

    let project_template = include_str!("project.yaml.hbs");

    let config_content = reg.render_template(&project_template, &json!({"project": cmd.project}))?;

    let mut f = File::create(config_path.as_path()).expect("Unable to create new project file");

    f.write(config_content.as_bytes()).expect("Unable to write configuration template");

    Ok(())
}

fn start(cmd: &Start) -> Result<(), Error> {
    let config = get_config(&cmd.project);

    println!("Starting {}...", cmd.project);

    if !tmux::has_session(&cmd.project) {
        let status = tmux::start(config);

        if status != 0 {
            // TODO Clean up this handling
            error!("Unable to start the session");
        }
    }

    tmux::attach(&cmd.project);

    Ok(())
}

fn list() -> Result<(), Error> {
    println!("Projects:\n");
    let config_dir_path = config_path();

    let des: ReadDir = fs::read_dir(config_dir_path)?;

    for de in des {
        let de = de?;

        if de.file_name().into_string().unwrap().ends_with(".yaml") {
            let f = File::open(de.path())?;
            let c: Config = serde_yaml::from_reader(f)?;
            println!("{}", c.name);
        }
    }

    Ok(())
}
