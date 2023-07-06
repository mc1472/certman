use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::PathBuf,
    process,
};

use crate::cli::Cli;
use anyhow::Context;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub ca: CaConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CaConfig {
    pub default_ca_path: PathBuf,

    pub vality_time_days: i64,
}

pub fn create_default_config(dirs: &ProjectDirs) -> Config {
    Config {
        ca: CaConfig {
            default_ca_path: dirs.data_dir().join("ca"),
            vality_time_days: 365,
        },
    }
}

pub fn read_config(app: &ProjectDirs, cli: &Cli) -> anyhow::Result<Config> {
    if let Some(user_config_path) = &cli.config {
        if user_config_path.exists() {
            let mut str = String::new();
            File::open(user_config_path)
                .with_context(|| format!("can't open {user_config_path:?}"))?
                .read_to_string(&mut str)
                .with_context(|| format!("Can't read {user_config_path:?}"))?;
            toml::from_str::<Config>(&str).with_context(|| {
                format!("Can't parse file {user_config_path:?}")
            })
        } else {
            eprintln!("can't find file {user_config_path:?}");
            process::exit(1)
        }
    } else {
        let config_path = app.config_dir().join("config.toml");
        if config_path.exists() {
            let mut str = String::new();
            File::open(&config_path)
                .with_context(|| format!("can't open {config_path:?}"))?
                .read_to_string(&mut str)
                .with_context(|| format!("Can't read {config_path:?}"))?;
            toml::from_str::<Config>(&str)
                .with_context(|| format!("Can't parse file {config_path:?}"))
        } else {
            let config = create_default_config(app);
            create_dir_all(app.config_dir())
                .context("Can't create config dir")?;
            File::create(&config_path)
                .with_context(|| format!("Can't create file {config_path:?}"))?
                .write_all(toml::to_string_pretty(&config)?.as_bytes())
                .with_context(|| format!("Can't write file {config_path:?}"))?;
            Ok(config)
        }
    }
}
