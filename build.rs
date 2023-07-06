use std::{env, io};

use clap::{CommandFactory, ValueEnum};
use clap_complete::{
    generate_to,
    Shell,
};

include!("src/cli.rs");

mod config {
    use std::path::PathBuf;
    pub struct Config {
        pub ca: CaConfig,
    }

    pub struct CaConfig {
        pub default_ca_path: PathBuf,

        pub vality_time_days: i64,
    }
}

fn main() -> Result<(), io::Error> {
    println!("hi");
    let out_dir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(out_dir) => out_dir,
    };
    let mut cli_cmd = Cli::command();
    for &shell in Shell::value_variants() {
        let path = generate_to(shell, &mut cli_cmd, "certman", &out_dir)?;
        println!("{path:?}");
    }

    Ok(())
}
