use std::{
    env::current_dir,
    fs::{create_dir, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{arg, Args, Command, Parser, Subcommand};
use rcgen::{Certificate, CertificateParams, DistinguishedName, IsCa, KeyPair, SanType};
use rustyline::DefaultEditor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Ca(CaArgs),
    Server(ServerArgs),
}

#[derive(Args, Debug)]
struct CaArgs {
    ca_dir: PathBuf,
    #[arg(long)]
    gen: bool,
}

#[derive(Args, Debug)]
struct ServerArgs {
    ca_dir: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
    name: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ca(args) => {
            if args.gen {
                create_dir(&args.ca_dir)?;
                let cert = create_cert(true)?;
                save_cert(args.ca_dir, "ca_cert", cert, None)?;
            }
        }
        Commands::Server(args) => {
            let ca = load_ca(args.ca_dir).context("can't load ca")?;
            let cert = create_cert(false)?;
            if let Some(out_dir) = args.out {
                create_dir(&out_dir)?;
                save_cert(out_dir, &args.name, cert, Some(ca))?;
            } else {
                save_cert(current_dir()?, &args.name, cert, Some(ca))?;
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

fn create_cert(is_ca: bool) -> anyhow::Result<Certificate> {
    let csr = create_csr(is_ca).context("can't create csr")?;
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

// println!("Hello, world!");
// let csr = create_csr()?;
// let cert = Certificate::from_params(csr)?;
// let pem = cert.serialize_pem()?;
// let mut file = File::options()
//     .create(true)
//     .write(true)
//     .truncate(true)
//     .open("./ca_cert.pem")?;
// file.write(pem.as_bytes())?;

fn create_csr(is_ca: bool) -> anyhow::Result<CertificateParams> {
    let mut rl = DefaultEditor::new()?;
    let mut csr = CertificateParams::default();

    let dn = get_dn(&mut rl)?;
    if is_ca {
        csr.is_ca = IsCa::Ca(rcgen::BasicConstraints::Constrained(1));
    } else {
        let san = get_subject_alt_names(&mut rl)?;
        csr.subject_alt_names = san;
    }

    csr.distinguished_name = dn;

    Ok(csr)
}

fn get_dn(rl: &mut DefaultEditor) -> anyhow::Result<DistinguishedName> {
    let mut dn = DistinguishedName::new();

    let country = rl.readline("Country > ")?;
    dn.push(rcgen::DnType::CountryName, country);
    let state_or_province = rl.readline("State or Province > ")?;
    dn.push(rcgen::DnType::StateOrProvinceName, state_or_province);
    let locality = rl.readline("Locality > ")?;
    dn.push(rcgen::DnType::LocalityName, locality);
    let org = rl.readline("Orgiazation > ")?;
    dn.push(rcgen::DnType::OrganizationName, org);
    let common_name = rl.readline("Common Name > ")?;
    dn.push(rcgen::DnType::CommonName, common_name);

    Ok(dn)
}

fn get_subject_alt_names(rl: &mut DefaultEditor) -> anyhow::Result<Vec<SanType>> {
    let subject_alt_name = rl
        .readline("Subject alt names (comma seperated) > ")?
        .split(",")
        .map(|dns| SanType::DnsName(dns.to_owned()))
        .collect::<Vec<_>>();

    Ok(subject_alt_name)
}
