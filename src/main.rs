mod rgl;

use anyhow::{Context, Result};
use clap::{crate_version, Arg, ArgAction, Command};
use log::LevelFilter;
use paris::log;
use simplelog::{error, ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

fn main() {
    let config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Debug)
        .build();

    TermLogger::init(
        LevelFilter::Info,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    if let Err(e) = run_command() {
        error!("{}", e);
        e.chain().skip(1).for_each(|e| log!("<red>[+]</> {e}"));
        std::process::exit(1);
    }
}

fn run_command() -> Result<()> {
    let matches = Command::new("rgl")
        .bin_name("rgl")
        .about("Not Regolith")
        .author("ink0rr")
        .version(crate_version!())
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("init")
                .about("Initializes a new Regolith project in the current directory"),
        )
        .subcommand(
            Command::new("install")
                .alias("i")
                .about("Downloads and installs Regolith filters from the internet, and adds them to the \"filterDefinitions\" list of the project's \"config.json\" file.")
                .arg(Arg::new("filters").num_args(0..).action(ArgAction::Set))
                .arg(Arg::new("force").short('f').long("force").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("run")
                .about("Runs Regolith with specified profile")
                .arg(Arg::new("profile").action(ArgAction::Set)),
        )
        .subcommand(
            Command::new("watch")
                .about("Watches project files and automatically runs Regolith when they change")
                .arg(Arg::new("profile").action(ArgAction::Set)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            rgl::init().context("Error initializing project")?;
        }
        Some(("install", matches)) => {
            let filters: Option<Vec<&String>> = match matches.get_many::<String>("filters") {
                Some(filters) => Some(filters.collect::<Vec<&String>>()),
                None => None,
            };
            let force = matches.get_flag("force");
            match filters {
                Some(filters) => {
                    rgl::install_add(filters, force).context("Error installing filter")?;
                }
                None => {
                    rgl::install_filters(force).context("Error installing filters")?;
                }
            };
        }
        Some(("run", matches)) => {
            let profile = match matches.get_one::<String>("profile") {
                Some(profile) => profile,
                None => "default",
            };
            rgl::run_or_watch(profile, false)
                .context(format!("Error running <b>{profile}</> profile"))?;
        }
        Some(("watch", matches)) => {
            let profile = match matches.get_one::<String>("profile") {
                Some(profile) => profile,
                None => "default",
            };
            rgl::run_or_watch(profile, true)
                .context(format!("Error running <b>{profile}</> profile"))?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
