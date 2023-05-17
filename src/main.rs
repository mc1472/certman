use std::{
    env::current_dir,
    fs::{create_dir, create_dir_all, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process,
};

use anyhow::Context;
use clap::{arg, Args, Parser, Subcommand};
use directories::ProjectDirs;
use rcgen::{Certificate, CertificateParams, KeyPair};
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use utils::prompt_question;

mod cert_sign_request;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// the config file.
    #[arg(long, short)]
    config: Option<PathBuf>,
    #[arg(long)]
    non_interactive: bool,
    #[arg(long, short)]
    force: bool,
    #[arg(long)]
    dn_file: Option<PathBuf>,
    #[arg(long)]
    san_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Ca(CaArgs),
    Certs(Certsargs),
}

#[derive(Args, Debug)]
struct CaArgs {
    #[arg(long)]
    ca_dir: Option<PathBuf>,
    #[arg(long)]
    gen: bool,
}

#[derive(Args, Debug)]
struct Certsargs {
    /// Output dir
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    self_signed: bool,
    name: String,
    ca_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub ca: CaConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CaConfig {
    default_ca_path: PathBuf,

    vality_time_days: i64,
}

fn create_default_config(dirs: &ProjectDirs) -> Config {
    Config {
        ca: CaConfig {
            default_ca_path: dirs.data_dir().join("ca"),
            vality_time_days: 365,
        },
    }
}

fn read_config(app: &ProjectDirs, cli: &Cli) -> anyhow::Result<Config> {
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

pub struct RunPlan {
    pub ca_or_cert: bool,
    pub generate_or_show: bool,
    pub user_read_dn: bool,
    pub dn_file: Option<PathBuf>,
    pub write_dn: bool,
    pub user_read_san: bool,
    pub san_file: Option<PathBuf>,
    pub write_san: bool,
    pub overwrite: bool,
    pub expiry_days: i64,
}

impl Default for RunPlan {
    fn default() -> Self {
        Self {
            ca_or_cert: false,
            generate_or_show: false,
            user_read_dn: true,
            dn_file: None,
            write_dn: true,
            user_read_san: true,
            san_file: None,
            write_san: true,
            overwrite: false,
            expiry_days: 7300,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut plan = RunPlan::default();
    let app = ProjectDirs::from("com", "mc1472", "certman").unwrap();
    let cli = Cli::parse();


    let config = read_config(&app, &cli)?;

    plan.user_read_dn = !cli.non_interactive;
    plan.user_read_san = !cli.non_interactive;
    plan.overwrite = cli.force;
    plan.dn_file = cli.dn_file;
    plan.san_file = cli.san_file;

    plan.expiry_days = config.ca.vality_time_days;
    let mut rl = DefaultEditor::new()?;

    match cli.command {
        Commands::Ca(args) => {
            let ca_dir = args.ca_dir.unwrap_or(config.ca.default_ca_path);
            plan.generate_or_show = args.gen;
            plan.ca_or_cert = true;
            if args.gen {
                create_dir_all(&ca_dir)?;
                if ca_dir.join("ca_cert.pem").exists() && !prompt_question(
                        &mut rl,
                        &format!(
                        "A Certificate Autority already exists at {ca_dir:?}. Overwrite? (y/N) "
                    ),
                        "y",
                    )? {
                        eprintln!(
                            "not Overwriting existing Certificate Autority"
                        );
                        process::exit(1);
                    
                }
                let cert = create_cert(&plan, &mut rl)?;
                save_cert(&ca_dir, "ca_cert", cert, None)?;
            } else {
                process::Command::new("openssl")
                    .args(["x509", "-text", "-in"])
                    .arg(ca_dir.join("ca_cert.pem"))
                    .arg("-noout")
                    .spawn()?
                    .wait()?;
            }
        }
        Commands::Certs(args) => {
            plan.generate_or_show = true;
            let ca = if args.self_signed {
                None
            } else if let Some(ca_dir) = args.ca_dir {
                Some(load_ca(ca_dir)?)
            } else {
                Some(load_ca(&config.ca.default_ca_path)?)
            };
            let cert = create_cert(&plan, &mut rl)?;
            if let Some(out_dir) = args.out {
                create_dir(&out_dir)?;
                save_cert(out_dir, &args.name, cert, ca)?;
            } else {
                save_cert(current_dir()?, &args.name, cert, ca)?;
            }
        }
    }

    Ok(())
}

fn save_cert<P: AsRef<Path>>(
    dir: P,
    name: &str,
    cert: Certificate,
    ca: Option<Certificate>,
) -> anyhow::Result<()> {
    let pem = if let Some(ca) = ca {
        cert.serialize_pem_with_signer(&ca)?
    } else {
        cert.serialize_pem()?
    };
    let key = cert.serialize_private_key_pem();
    let mut cert_file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(dir.as_ref().join(format!("{}.pem", name)))
        .with_context(|| format!("can't open file {}.pem", name))?;
    cert_file
        .write(pem.as_bytes())
        .with_context(|| format!("can't write file {}.pem", name))?;
    let mut key_file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(dir.as_ref().join(format!("{}.key", name)))
        .with_context(|| format!("can't write file {}.key", name))?;
    key_file
        .write(key.as_bytes())
        .with_context(|| format!("can't write file {}.pem", name))?;
    Ok(())
}

fn create_cert(
    config: &RunPlan,
    rl: &mut DefaultEditor,
) -> anyhow::Result<Certificate> {
    let csr = cert_sign_request::create_csr(config, rl)
        .context("can't create csr")?;
    let cert = Certificate::from_params(csr).context("can't create cert")?;
    Ok(cert)
}

fn load_ca<P: AsRef<Path>>(ca_dir: P) -> anyhow::Result<Certificate> {
    let path = ca_dir.as_ref();

    let mut key = String::new();
    let key_path = path.join("ca_cert.key");
    File::options()
        .read(true)
        .open(&key_path)
        .with_context(|| format!("can't open {:?}", &key_path))?
        .read_to_string(&mut key)
        .with_context(|| format!("can't read {:?}", &key_path))?;

    let key_pair = KeyPair::from_pem(&key)?;

    let mut cert = String::new();
    let cert_path = path.join("ca_cert.pem");
    File::options()
        .read(true)
        .open(&cert_path)
        .with_context(|| format!("can't open {:?}", &cert_path))?
        .read_to_string(&mut cert)
        .with_context(|| format!("can't read {:?}", &cert_path))?;
    let cert_prams = CertificateParams::from_ca_cert_pem(&cert, key_pair)?;
    Ok(Certificate::from_params(cert_prams)?)
}
