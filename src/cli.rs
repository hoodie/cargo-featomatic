use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

#[derive(Clone, Debug, Default, RustcDecodable)]
pub struct Options {
    pub arg_args: Vec<String>,
    pub version: bool,
    pub verbose: u32,
    pub quiet: bool,
    pub manifest_path: Option<String>,
    pub color: Option<String>,
    pub frozen: bool,
    pub locked: bool,
    pub yes: bool,
}

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
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Use verbose output (-vv very verbose output)"),

            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Use quiet output"),

            Arg::with_name("yes")
                .short("y")
                .long("yes")
                .help("Don't ask for confirmation"),

            Arg::with_name("manifest-path")
                .long("manifest-path")
                .takes_value(true)
                .value_name("featomatic")
                .help("Path to the manifest to analyze"),

            Arg::with_name("color")
                .long("color")
                .takes_value(true)
                .value_name("COLOR")
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

    pub fn from_matches(matches: &ArgMatches) -> Options {
        if let Some(matches) = matches.subcommand_matches("featomatic") {
            Options {
                arg_args: vec![],
                version: matches.is_present("version"),
                verbose: matches.occurrences_of("verbose") as u32,
                quiet: matches.is_present("quite"),
                manifest_path: matches.value_of("manifest-path").map(ToOwned::to_owned),
                color: matches.value_of("color").map(ToOwned::to_owned),
                frozen: matches.is_present("frozen"),
                yes: matches.is_present("yes"),
                locked: matches.is_present("locked"),
            }
        } else {
            Options::default()
        }
    }
}
