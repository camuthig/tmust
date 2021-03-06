extern crate clap;
extern crate dirs;
#[macro_use]
extern crate handlebars;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use handlebars::Handlebars;
use quicli::prelude::*;
use serde_json::json;
use std::env;
use std::fs;
use std::fs::{File, ReadDir};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use structopt::StructOpt;

mod tmux;
use tmux::{Config, Project};

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
struct Stop {
    #[structopt(index = 1, help = "The name of the tmux project")]
    project: String,
}

#[derive(Debug, StructOpt)]
struct Edit {
    #[structopt(index = 1, help = "The name of the tmux project")]
    project: String,
}

#[derive(Debug, StructOpt)]
struct Delete {
    #[structopt(index = 1, help = "The name of the tmux project")]
    project: String,
}

#[derive(Debug, StructOpt)]
struct Rename {
    #[structopt(index = 1, help = "The existing project to rename")]
    existing: String,
    #[structopt(index = 2, help = "The new name for the project")]
    new: String,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "new", about = "Add a new tmux project")]
    New(New),

    #[structopt(name = "edit", about = "Edit an existing tmux project")]
    Edit(Edit),

    #[structopt(name = "start", about = "Start a new tmux session")]
    Start(Start),

    #[structopt(name = "stop", about = "Stop a running tmux session")]
    Stop(Stop),

    #[structopt(name = "delete", about = "Delete an existing tmux project")]
    Delete(Delete),

    #[structopt(name = "rename", about = "Rename an existing tmux project")]
    Rename(Rename),

    #[structopt(name = "list", about = "List the configured projects")]
    List {},
}

fn main() -> CliResult {
    let tmust = Tmust::from_args();

    match &tmust.cmd {
        Command::New(c) => new(&c)?,
        Command::Edit(c) => edit(&c)?,
        Command::Start(c) => start(&c)?,
        Command::Stop(c) => stop(&c)?,
        Command::Delete(c) => delete(&c)?,
        Command::Rename(c) => rename(&c)?,
        Command::List { .. } => list()?,
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

    let config_content =
        reg.render_template(&project_template, &json!({"project": cmd.project}))?;

    let mut f = File::create(config_path.as_path()).expect("Unable to create new project file");

    f.write(config_content.as_bytes())
        .expect("Unable to write configuration template");

    edit(&Edit {
        project: cmd.project.to_owned(),
    })?;

    Ok(())
}

fn edit(cmd: &Edit) -> Result<(), Error> {
    let mut config_path = config_path();

    config_path.push(cmd.project.to_owned() + ".yaml");

    if !config_path.as_path().exists() {
        println!("The {} project does not yet exist.", cmd.project);
        return Ok(());
    }

    let editor = env::var("EDITOR")?;

    StdCommand::new(editor)
        .args(&[config_path.as_path()])
        .status()?;

    Ok(())
}

fn start(cmd: &Start) -> Result<(), Error> {
    let project = Project::new(get_config(&cmd.project));

    println!("Starting {}...", cmd.project);

    let status = project.start();

    if status != 0 {
        // TODO Clean up this handling
        error!("Unable to start the session");
    }

    Ok(())
}

fn stop(cmd: &Stop) -> Result<(), Error> {
    let project = Project::new(get_config(&cmd.project));

    println!("Stopping {}...", cmd.project);

    let status = project.stop();

    if status != 0 {
        // TODO Clean up this handling
        error!("Unable to stop the session");
    }

    Ok(())
}

fn delete(cmd: &Delete) -> Result<(), Error> {
    let mut config_path = config_path();

    config_path.push(cmd.project.to_owned() + ".yaml");

    if !config_path.as_path().exists() {
        println!("The {} project does not yet exist.", cmd.project);
        return Ok(());
    }

    fs::remove_file(config_path.as_path())?;

    println!("Deleted {} project", cmd.project);

    Ok(())
}

fn rename(cmd: &Rename) -> Result<(), Error> {
    let mut config = get_config(&cmd.existing);
    config.name = cmd.new.to_owned();

    let mut existing_path = config_path();

    existing_path.push(cmd.existing.to_owned() + ".yaml");

    let mut new_path = config_path();

    new_path.push(cmd.new.to_owned() + ".yaml");

    if new_path.as_path().exists() {
        println!("The {} project already exists.", cmd.new);
        return Ok(());
    }

    let new_content = serde_yaml::to_string(&config)?;

    fs::remove_file(existing_path.as_path())?;

    fs::write(new_path.as_path(), new_content)?;

    Ok(())
}

fn list() -> Result<(), Error> {
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
