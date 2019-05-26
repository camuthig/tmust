extern crate clap;
extern crate dirs;
extern crate handlebars;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
use clap::{App, AppSettings, SubCommand, ArgMatches, Arg};
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::fs::{File, ReadDir};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

mod tmux;
use tmux::Config;


fn main() {
    let matches = App::new("tmust")
        .setting(AppSettings::SubcommandRequiredElseHelp)
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
        .subcommand(SubCommand::with_name("list")
                    .about("List the configured projects"))
        .get_matches();

    match matches.subcommand() {
        ("new", Some(matches)) => new(matches),
        ("start", Some(matches)) => start(matches),
        ("list", Some(matches)) => list(matches),
        _ => println!("No subcommand"),
    }
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

fn config_file(project: &str) -> File {
    let mut config_dir = config_path();

    config_dir.push(project.to_owned() + ".yaml");

    if !config_dir.as_path().exists() {
        panic!("Config file does not exist for project. Try use \"start\" instead");
    }

    let f = File::open(config_dir.as_path()).expect("Unable to read configuration file");

    f
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

    let reg = Handlebars::new();

    let project_template = include_str!("project.yaml.hbs");

    let config_content = reg.render_template(&project_template, &json!({"project": project})).expect("Unable to render template");

    let mut f = File::create(config_path.as_path()).expect("Unable to create new project file");

    f.write(config_content.as_bytes()).expect("Unable to write configuration template");
}

fn start(matches: &ArgMatches) {
    let project = matches.value_of("project").unwrap();
    let config = get_config(project);

    println!("Starting {}...", project);

    if !tmux::has_session(project.to_string()) {
        let status = tmux::start(config);

        if status != 0 {
            // TODO Clean up this handling
            panic!("Unable to start the session");
        }
    }

    tmux::attach(project.to_string());
}

fn list(matches: &ArgMatches) {
    println!("Projects:\n");
    let config_dir_path = config_path();

    let des: ReadDir = fs::read_dir(config_dir_path).unwrap();

    for de in des {
        let de = de.unwrap();
        //println!("{:?}", de.path());

        if de.file_name().into_string().unwrap().ends_with(".yaml") {
            let f = File::open(de.path()).expect("Unable to read configuration file");
            let c: Config = serde_yaml::from_reader(f).expect("Unable to parse config file");
            println!("{}", c.name);
        }
    }


}
