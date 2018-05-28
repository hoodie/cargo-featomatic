extern crate cargo;
#[macro_use]
extern crate clap;
extern crate rustc_serialize;
extern crate itertools;

use std::iter::FromIterator;
use std::process;

use cargo::core::Workspace;
use cargo::util::{ human, CliResult, CliError, Config };
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::process_builder::process;
use itertools::Itertools;

#[derive(RustcDecodable)]
struct Options {
    arg_args: Vec<String>,
    flag_version: bool,
    flag_verbose: u32,
    flag_quiet: bool,
    flag_manifest_path: Option<String>,
    flag_color: Option<String>,
    flag_frozen: bool,
    flag_locked: bool,
}

use clap::{ App, Arg, SubCommand, AppSettings, ArgMatches };
impl Options {
    pub fn app(subcommand_required: bool) -> App<'static, 'static> {
        App::new("cargo")
            .bin_name("cargo")
            .subcommand(Options::subapp(subcommand_required))
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .global_settings(&[
                             AppSettings::ColorAuto,
                             AppSettings::ColoredHelp,
                             AppSettings::VersionlessSubcommands,
                             AppSettings::DeriveDisplayOrder,
                             AppSettings::UnifiedHelpMessage,
            ])
    }
    pub fn subapp(subcommand_required: bool) -> App<'static, 'static> {
        let mut app = SubCommand::with_name("featomatic")
            .author(crate_authors!())
            .version(crate_version!())
            .about(crate_description!())
            .args(&Options::args());
        if subcommand_required {
            app = app.setting(AppSettings::SubcommandRequiredElseHelp);
        }
        app
    }
    pub fn args() -> Vec<Arg<'static, 'static>> {
        vec![
            Arg::with_name("verbose")
                .short("v").long("verbose")
                .multiple(true)
                .help("Use verbose output (-vv very verbose output)"),
            Arg::with_name("quiet")
                .short("q").long("quiet")
                .help("Use quiet output"),
            Arg::with_name("manifest-path")
                .long("manifest-path")
                .takes_value(true).value_name("featomatic")
                .help("Path to the manifest to analyze"),
            Arg::with_name("color")
                .long("color")
                .takes_value(true).value_name("COLOR")
                .possible_values(&["auto", "always", "never"])
                .help("Coloring"),
            Arg::with_name("frozen")
                .long("frozen")
                .help("Require Cargo.lock and cache are up to date"),
            Arg::with_name("locked")
                .long("locked")
                .help("Require Cargo.lock is up to date"),
        ]
    }
    fn from_matches(matches: &ArgMatches) -> Options {
        Options {
            arg_args: vec![],
            flag_version: matches.is_present("version"),
            flag_verbose: matches.occurrences_of("verbose") as u32,
            flag_quiet: matches.is_present("quite"),
            flag_manifest_path: None,
            flag_color: None,
            flag_frozen: false,
            flag_locked: false,
        }
    }
}

fn main() {
    let matches = Options::app(false).get_matches();
    let options = Options::from_matches(&matches);
    let mut config = Config::default().expect("No idea why this would fail");
    let result = real_main(options, &mut config);
    if let Err(err) = result {
        config.shell().error(err).expect("Can't do much");
        process::exit(1);
}}

fn real_main(options: Options, config: &Config) -> CliResult<Option<()>> {
    config.configure(
        options.flag_verbose,
        Some(options.flag_quiet),
        &options.flag_color,
        options.flag_frozen,
        options.flag_locked)?;

    if options.flag_version {
        config.shell().say(format!("cargo-featomatic {}", env!("CARGO_PKG_VERSION")), 0)?;
        return Ok(None);
    }

    let base_args = {
        let mut base_args = options.arg_args;
        for _ in 0..options.flag_verbose {
            base_args.push("--verbose".to_owned());
        }
        if options.flag_quiet == true {
            base_args.push("--quiet".to_owned());
        }
        if let Some(ref manifest_path) = options.flag_manifest_path {
            base_args.push("--manifest-path".to_owned());
            base_args.push(manifest_path.clone());
        }
        if let Some(ref color) = options.flag_color {
            base_args.push("--color".to_owned());
            base_args.push(color.clone());
        }
        if options.flag_frozen {
            base_args.push("--frozen".to_owned());
        }
        if options.flag_locked {
            base_args.push("--locked".to_owned());
        }
        base_args.push("--no-default-features".to_owned());
        base_args
    };

    let root = find_root_manifest_for_wd(options.flag_manifest_path, config.cwd())?;
    let workspace = Workspace::new(&root, config).map_err(|e| {println!("{:#?}", e);e})?;
    let current = workspace.current()?;
    let features = Vec::from_iter(current.summary().features().keys().map(|s| s as &str).filter(|s| s != &"default"));

    let set_to_process = |set| {
        let mut process = process("cargo");
        process.arg("check");
        process.args(&base_args);
        if set != "" {
            process.arg("--features").arg(set);
        }
        process
    };

    let feature_sets = (1..features.len()).flat_map(|n| features.iter().combinations(n).map(|combination| combination.iter().join(" ")));
    let sets = feature_sets.collect::<Vec<_>>();
    println!("feature sets {:#?} {}", sets, sets.len());

    let mut failed = false;
    for process in sets.into_iter().map(|set| set_to_process(set)) {
        config.shell().status("Running", process.to_string())?;
        match process.exec() {
            Ok(()) => (),
            Err(err) => {
                config.shell().error(err)?;
                failed = true;
            }
        }
    }

    if failed {
        Err(CliError::new(human("at least one subcommand failed"), 7))
    } else {
        Ok(None)
    }
}
