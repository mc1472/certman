use std::{env::current_dir, path::PathBuf};

use clap::{arg, Args, Parser, Subcommand};

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// the config file.
    #[arg(long, short)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Ca(CaArgs),
    Certs(CertsArgs),
}

#[derive(Args, Debug)]
struct CaArgs {
    #[arg(long)]
    gen: bool,
    #[arg(long)]
    days: Option<i64>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    non_interactive: bool,
    #[arg(long, short)]
    force: bool,
    ca_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct CertsArgs {
    #[arg(long)]
    gen: bool,
    #[arg(long)]
    days: Option<i64>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    self_signed: bool,
    #[arg(long)]
    non_interactive: bool,
    #[arg(long, short)]
    force: bool,
    name: String,
    ca_dir: Option<PathBuf>,
}


#[derive(Debug)]
pub enum CaOrCert {
    Ca,
    Cert { ca_dir: PathBuf },
}

#[derive(Debug)]
pub struct CertPlan {
    pub ca_or_cert: CaOrCert,
    pub self_signed: bool,
    pub generate_or_show: bool,
    pub force: bool,
    pub expiry_days: i64,
    pub output_path: PathBuf,
    pub name: String,
    pub interactive: bool,
}

pub fn create_plan(cli: Cli, config: &Config) -> CertPlan {
    match cli.command {
        Commands::Ca(cert) => CertPlan {
            ca_or_cert: CaOrCert::Ca,
            self_signed: true,
            generate_or_show: cert.gen,
            force: cert.force,
            expiry_days: cert.days.unwrap_or(config.ca.vality_time_days),
            output_path: cert.out.unwrap_or(config.ca.default_ca_path.clone()),
            name: "ca_cert".into(),
            interactive: !cert.non_interactive,
        },
        Commands::Certs(cert) => CertPlan {
            ca_or_cert: CaOrCert::Cert {
                ca_dir: cert.ca_dir.unwrap_or(config.ca.default_ca_path.clone()),
            },
            self_signed: cert.self_signed,
            generate_or_show: cert.gen,
            force: cert.force,
            expiry_days: cert.days.unwrap_or(30),
            output_path: cert.out.unwrap_or(current_dir().unwrap()),
            name: cert.name,
            interactive: !cert.non_interactive,
        },
    }
}

