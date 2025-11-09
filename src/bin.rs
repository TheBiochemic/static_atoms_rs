mod dist;
#[cfg(test)]
mod tests;

use std::{env, path::PathBuf};

use crate::dist::run_dist;

static DEFAULT_MAX_DEPTH: u8 = 8;

pub struct Configuration {
    root: PathBuf,
    out: Option<PathBuf>,
    clean: bool,
    write: bool,
    max_depth: u8,
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            [
                format!("  root: `{}`", self.root.to_string_lossy()),
                format!(
                    "  out: `{}`",
                    match &self.out {
                        Some(out) => out.to_string_lossy().to_string(),
                        None => "<root>/dist".to_owned(),
                    }
                ),
                format!("  clean: `{}`", self.clean),
                format!("  write: `{}`", self.write),
                format!("  max_depth: `{}`", self.max_depth),
            ]
            .join("\n")
        )
    }
}

pub enum Action {
    RunHelp,
    RunDist,
    ParamsHelp,
    ShowConfig,
}

fn main() {
    let mut unrecognized_params: Vec<String> = vec![];
    let config_root: PathBuf = env::current_dir()
        .expect("The current directory either doesn't exist, or is inaccessible!");
    let mut config = Configuration {
        root: config_root.clone(),
        out: None,
        clean: false,
        write: true,
        max_depth: DEFAULT_MAX_DEPTH,
    };
    let mut action = Action::RunHelp;

    for (index, arg) in env::args().enumerate() {
        // Ignore first index
        if index == 0 {
            continue;
        }

        // Parameters
        if let Some(param) = arg.strip_prefix("--") {
            if param.eq_ignore_ascii_case("clean") {
                config.clean = true;
                continue;
            }

            if param.eq_ignore_ascii_case("dry") {
                config.write = false;
                continue;
            }

            if let Some(out_param) = param.strip_prefix("out=") {
                let path: PathBuf = PathBuf::from(out_param);
                config.out = Some(path);
                continue;
            }

            if let Some(root_param) = param.strip_prefix("root=") {
                let path: PathBuf = PathBuf::from(root_param);
                config.root = path;
                continue;
            }

            if let Some(depth_param) = param.strip_prefix("depth=") {
                let depth: u8 = depth_param
                    .parse()
                    .expect("The depth parameter does not contain a valid number");
                config.max_depth = depth;
                continue;
            }
        }

        // Parameter shortcuts
        if let Some(param) = arg.strip_prefix('-') {
            let mut unrecognized_letters = String::default();
            for letter in param.chars() {
                match letter {
                    'c' => {
                        config.clean = true;
                        continue;
                    }

                    'd' => {
                        config.write = false;
                        continue;
                    }

                    letter => {
                        unrecognized_letters.push(letter);
                    }
                }
            }
            if !unrecognized_letters.is_empty() {
                unrecognized_params.push("-".to_string() + &unrecognized_letters);
            } else {
                continue;
            }
        }

        // Actions
        if arg.eq_ignore_ascii_case("dist") {
            action = Action::RunDist;
            continue;
        }

        if arg.eq_ignore_ascii_case("help") {
            action = Action::RunHelp;
            continue;
        }

        if arg.eq_ignore_ascii_case("config") {
            action = Action::ShowConfig;
            continue;
        }

        // Unrecognized params
        unrecognized_params.push(arg);
    }

    if !unrecognized_params.is_empty() {
        action = Action::ParamsHelp;
    }

    match action {
        Action::ShowConfig => show_config(&config),
        Action::RunHelp => show_help(),
        Action::RunDist => run_dist(&config),
        Action::ParamsHelp => show_params_help(&unrecognized_params),
    }
}

fn show_config(config: &Configuration) {
    println!("Current configuration:\n\n{config}");
}

fn show_help() {
    println!(
        "\
        Usage: static_atoms [ACTION]... [PARAMS]...\n\
        Transforms a templated website into a static website, that can be hosted\n\
        by any webserver. Runs this help, if no action has been specified.\n\n\
        Available actions are:\n\
        \tdist\t\tbuilds the dist in the specified roots /dist directory\n\
        \tconfig\t\tdumps the config into stdout\n\
        \thelp\t\tshows this help\n\n\
        Additional Parameters are:\n\
        \t--out=<path>\toverrides the default output directory; default\n\
        \t\t\tis <root_dir>/dist\n\
        \t--root=<path>\toverrides the project root; the default is\n\
        \t\t\tthe current folder the command is executed in\n\
        \t--clean\t\tdeletes the contents of the output folder before building\n\
        \t\t\tthe new static website\n\
        \t--dry\t\tdo not write any files, to validate if the process runs\n\
        \t\t\tsuccessfully without errors\n\
        \t--depth\t\tsets the maximum recursion depth. Default is {DEFAULT_MAX_DEPTH}\n\
        \t-c\t\tsame as --clean\n\
        \t-d\t\tsame as --dry\n\
    "
    )
}

fn show_params_help(unrecognized_params: &[String]) {
    for param in unrecognized_params {
        println!("Unrecognized parameter: {param}");
    }
    println!("Use `static_atoms help` to get more info. No action has been performed");
}
