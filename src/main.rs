extern crate cargo;
#[macro_use]
extern crate clap;
extern crate itertools;
extern crate rustc_serialize;

use std::process;

mod cli;
mod util;
mod features;

use cargo::core::Workspace;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::process_builder::process;
use cargo::util::{human, CliError, CliResult, Config};
use itertools::Itertools;

fn main() {
    let matches = cli::Options::app(false).get_matches();
    let options = cli::Options::from_matches(&matches);
    let mut config = Config::default().expect("No idea why this would fail");
    let result = real_main(options, &mut config);
    if let Err(err) = result {
        config.shell().error(err).expect("Can't do much");
        process::exit(1);
    }
}

fn base_args(options: cli::Options) -> Vec<String> {
    let mut base_args = options.arg_args;
    for _ in 0..options.verbose {
        base_args.push("--verbose".to_owned());
    }
    if options.quiet == true {
        base_args.push("--quiet".to_owned());
    }
    if let Some(ref manifest_path) = options.manifest_path {
        base_args.push("--manifest-path".to_owned());
        base_args.push(manifest_path.clone());
    }
    if let Some(ref color) = options.color {
        base_args.push("--color".to_owned());
        base_args.push(color.clone());
    }
    if options.frozen {
        base_args.push("--frozen".to_owned());
    }
    if options.locked {
        base_args.push("--locked".to_owned());
    }
    base_args.push("--no-default-features".to_owned());
    base_args
}

fn real_main(options: cli::Options, config: &Config) -> CliResult<Option<()>> {
    config.configure(
        options.verbose,
        Some(options.quiet),
        &options.color,
        options.frozen,
        options.locked,
    )?;

    // debug!("options {:#?}", options);

    if options.version {
        config
            .shell()
            .say(format!("cargo-featomatic {}", env!("CARGO_PKG_VERSION")), 0)?;
        return Ok(None);
    }


    let root = find_root_manifest_for_wd(options.manifest_path.clone(), config.cwd())?;
    let workspace = Workspace::new(&root, config).map_err(|e| {
        println!("{}", e);
        e
    })?;
    let current = workspace.current()?;

    let _summary = current
            .summary();
    println!("{:#?}", _summary);


    let features = features::discriminate_features(current.summary());

    let feature_names = current
            .summary()
            .features()
            .keys()
            .cloned()
            .collect::<Vec<_>>();

    println!("features: {:#?}", feature_names);
    println!("features discriminated: {:#?}", features);

   check_combinations(&feature_names, &options, config)
}

fn check_combinations(feature_names: &[String], options: &cli::Options, config: &Config) -> CliResult<Option<()>> {
    let base_args = base_args(options.to_owned());

    let set_to_process = |set| {
        let mut process = process("cargo");
        process.arg("check");
        process.args(&base_args);
        if set != "" {
            process.arg("--features").arg(set);
        }
        process
    };

    let feature_sets = (1..feature_names.len()).flat_map(|n| {
        feature_names
            .iter()
            .combinations(n)
            .map(|combination| combination.iter().join(" "))
    });

    let sets = feature_sets.collect::<Vec<_>>();

    let set_count = sets.len();
    let mut failed = false;


    if set_count < 10 || options.yes || util::really(&format!("There are {} possible feature combinations, checking could take very long! Proceed?", set_count)) {
        // debug!("feature sets {:#?} {}", sets, set_count);

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
    }

    if failed {
        Err(CliError::new(human("at least one subcommand failed"), 7))
    } else {
        Ok(None)
    }
}
