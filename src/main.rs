extern crate toml;

extern crate rustyline;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate colored;
extern crate env_logger;
extern crate heck;
extern crate regex;
extern crate reqwest;
extern crate zip;

mod core;
mod util;

use colored::*;
use rustyline::Editor;

use std::{
    env,
    io::IsTerminal,
    path::{Path, PathBuf},
};

use crate::core::{Param, Reframe};

fn print_usage(args: &[String]) {
    let path = Path::new(&args[0]);
    let exe_name = path.file_name().unwrap().to_str().unwrap();
    println!("Usage: ");
    println!("       ");
    println!("       $ {} [SOURCE] [OPTIONS]", exe_name);
    println!("       $ {} apply [SOURCE] [OPTIONS]", exe_name);
    println!();
    println!("OPTIONS:");
    println!();
    println!("       -L,--list          List available sources.");
    println!("       --dry-run          Test only, don't touch disk.");
    println!("       -Pkey=value        Preset parameters (short form).");
    println!("       --param key=value  Preset parameters (long form).");
    println!("       -b,--branch        Select branch to use. Default: master");
    println!("       ");
    println!(
        "       --out              Custom output dir name (default: project name in kebab case)."
    );
    println!("       --quiet            Don't ask anything, just do it.");
    println!("       --yaml <path>      Load parameters from YAML file.");
    println!();
    println!("ENVIRONMENT:");
    println!();
    println!("       REFRAME_PARAM_<key>=<value>   Set parameters via env vars.");
    println!();
    println!("SUBCOMMANDS:");
    println!();
    println!(
        "       apply             Apply template to current directory (template must have mode = \"apply\")."
    );
    println!();
    println!("Examples:");
    println!();
    println!("       $ {} anvie/basic-rust", exe_name);
    println!("       $ {} anvie/basic-rust --dry-run", exe_name);
    println!("       $ {} apply anvie/git-init", exe_name);
    println!();
}

// extract user's arguments to params
fn extract_params(args: &[String]) -> Vec<Param> {
    let mut params = Vec::new();

    // 1. Load from environment variables: REFRAME_PARAM_key=value
    for (key, val) in env::vars() {
        if let Some(k) = key.strip_prefix("REFRAME_PARAM_") {
            params.push(Param::new(k.to_lowercase(), val));
        }
    }

    // 2. Load from CLI args: -Pkey=value or --param key=value
    let mut i = 0;
    while i < args.len() {
        let a = args[i].trim();
        if a.starts_with("-P") {
            if let Some(eq) = a.find('=') {
                let key = a[2..eq].to_string();
                let val = a[eq + 1..].to_string();
                params.push(Param::new(key, val));
            }
        } else if a == "--param" {
            if let Some(next) = args.get(i + 1) {
                if let Some(eq) = next.find('=') {
                    let key = next[..eq].to_string();
                    let val = next[eq + 1..].to_string();
                    params.push(Param::new(key, val));
                }
                i += 1; // skip the value
            }
        }
        i += 1;
    }

    params
}

/// Load parameters from a YAML file (flat key-value mapping).
fn load_yaml_params(yaml_path: &str) -> Vec<Param> {
    use std::fs;

    let content = fs::read_to_string(yaml_path)
        .unwrap_or_else(|e| panic!("Cannot read YAML file `{}`: {}", yaml_path, e));
    let value: serde_yaml::Value = serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Cannot parse YAML file `{}`: {}", yaml_path, e));

    let mut params = Vec::new();
    if let serde_yaml::Value::Mapping(map) = value {
        for (k, v) in map {
            let key = match k.as_str() {
                Some(k) => k.to_string(),
                None => continue,
            };
            let val = match v {
                serde_yaml::Value::String(s) => s,
                serde_yaml::Value::Number(n) => n.to_string(),
                serde_yaml::Value::Bool(b) => b.to_string(),
                _ => continue,
            };
            params.push(Param::new(key, val));
        }
    }
    params
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let non_interactive = !std::io::stdin().is_terminal();
    let quiet = args.contains(&"--quiet".to_string()) || non_interactive;

    if args.contains(&"--version".to_string()) {
        println!(" Reframe {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if !quiet {
        println!();
        println!(" Reframe {}", env!("CARGO_PKG_VERSION"));
        println!(" project generator tool");
        println!(" by: Robin Syihab <r@ansvia.com>");
        println!(" Twitter: @anvie");
        println!(" ---------------------------");
        println!();
    }

    if args.len() < 2 || args[1] == "--help" {
        print_usage(&args);
        return;
    }

    let list_sources = args.contains(&"-L".to_string()) || args.contains(&"--list".to_string());

    if list_sources {
        println!(" Available sources:");
        println!();
        for (name, description) in util::get_available_sources().await.unwrap() {
            println!(" * {0: <30} - {1: <10}", name, description);
        }
        println!();
        return;
    }

    let dry_run = args.contains(&"--dry-run".to_string());

    if dry_run {
        debug!("DRY RUN MODE");
    }

    let (source, apply_mode) = if args[1] == "apply" {
        if args.len() < 3 {
            eprintln!("Usage: reframe apply <source> [OPTIONS]");
            std::process::exit(1);
        }
        (&args[2], true)
    } else {
        (&args[1], false)
    };
    let branch = get_param_value(&args, "--branch", "-b").unwrap_or_else(|| "master".to_string());

    let reframe_work_path = env::temp_dir().join("reframe_work");
    let source_path = if !Path::new(&source).exists() {
        debug!("source not found in local: {}", source);
        debug!("trying get from github.com/{} ...", source);
        if !quiet {
            if branch != "master" {
                println!(" Downloading from repo `{}` branch `{}`...", source, branch);
            } else {
                println!(" Downloading from repo `{}`...", source);
            }
        }
        let url = format!(
            "https://github.com/{}.rf/archive/{}.zip?nocache={}",
            source,
            branch,
            util::get_current_time_millis()
        );
        debug!("output: {}", env::temp_dir().display());
        if let Err(e) = util::download(&url, &reframe_work_path, &format!("{}.zip", branch)).await {
            eprintln!(
                "😭 {} {}, while pulling from repo for `{}`",
                "FAILED:".red(),
                e,
                source.bright_blue()
            );
            eprintln!();
            return;
        }
        let _path = reframe_work_path.join(format!(
            "{}.rf-master",
            source.split('/').skip(1).collect::<String>()
        ));

        if _path.exists() {
            _path
        } else {
            // change -master with -main as the default branch
            reframe_work_path.join(format!(
                "{}.rf-main",
                source.split('/').skip(1).collect::<String>()
            ))
        }
    } else {
        PathBuf::from(&source)
    };

    let mut rl = Editor::<()>::new()
        .unwrap_or_else(|_| panic!("Unable to create editor: {}", "Rustyline".red()));

    let history_path = env::temp_dir().join(".reframe~");

    if rl.load_history(&history_path).is_err() {
        debug!("no history");
    }

    // 1. Load from env vars and CLI flags (highest priority)
    let mut params = extract_params(&args);

    // 2. Load from YAML file (lowest priority — env/CLI override YAML)
    {
        let mut i = 0;
        while i < args.len() {
            let a = args[i].trim();
            if a == "--yaml" {
                if let Some(next) = args.get(i + 1) {
                    params.extend(load_yaml_params(next));
                    i += 1;
                }
            } else if a.starts_with("--yaml=") {
                let path = &a[7..];
                params.extend(load_yaml_params(path));
            }
            i += 1;
        }
    }

    let mut rf = match Reframe::open(&source_path, &mut rl, dry_run, params, apply_mode) {
        Ok(rf) => rf,
        Err(e) => {
            eprintln!(
                "Cannot open reframe source in tmp dir. {}",
                format!("{}", e).red()
            );
            std::process::exit(2);
        }
    };

    // get custom pre-out-name if any
    let pre_out_name: Option<String> = get_param_value(&args, "--out", "");

    match rf.generate(".", pre_out_name, quiet) {
        Ok(Some(out_name)) => {
            println!();
            if apply_mode {
                println!("  ✨ template applied to `{}`", out_name);
            } else {
                println!("  ✨ project generated at `{}`", out_name);
            }
            println!("{}", "     Ready to roll! 😎".green());

            if let Some(text) = rf.config.project.finish_text {
                println!(
                    "________________________________________________\n\n{}",
                    text
                );
            }
        }
        Ok(None) => {
            println!("aborted.");
        }
        Err(e) => {
            eprintln!("{}: {}", "ERROR".red(), e);
        }
    }
    rl.save_history(&history_path).expect("cannot save history");
}

fn get_param_value(args: &Vec<String>, name: &str, short_name: &str) -> Option<String> {
    args.iter()
        .map(|a| a.trim())
        .filter(|a| a.starts_with(name) || (short_name != "" && a.starts_with(short_name)))
        .map(|a| {
            let s = a.split("=").collect::<Vec<&str>>();
            if s.len() == 2 {
                Some(s[1].to_string())
            } else {
                None
            }
        })
        .collect::<Vec<Option<String>>>()
        .pop()
        .unwrap_or(None)
}
