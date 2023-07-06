use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::Context;
use clap::Parser;
use cli::{create_plan, CertPlan, Cli};
use config::read_config;
use directories::ProjectDirs;
use rcgen::{Certificate, CertificateParams, KeyPair};

use crate::{
    cert_sign_request::{create_csr, dn, san},
    cli::CaOrCert,
};

mod cert_sign_request;
mod cli;
mod config;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let dirs = ProjectDirs::from("com", "mc1472", "cerman").unwrap();
    let config = read_config(&dirs, &cli)?;
    let cert_plan = create_plan(cli, &config);

    let mut rl = rustyline::DefaultEditor::new()?;
    if cert_plan.generate_or_show {
        if !cert_plan.output_path.exists() {
            fs::create_dir(&cert_plan.output_path)?;
        }
        let dn = dn::get_dn_file(&cert_plan)
            .or_else(|_| dn::get_dn_interactive(&cert_plan, &mut rl))?;
        dn::write_dn_file(&cert_plan, &dn)?;
        let san = san::get_san_file(&cert_plan)
            .or_else(|_| san::get_san_interactive(&cert_plan, &mut rl))?;
        san::write_san_file(&cert_plan, &san)?;
        let csr = create_csr(&cert_plan, dn, san);
        let cert = Certificate::from_params(csr)?;
        match &cert_plan.ca_or_cert {
            CaOrCert::Ca => save_cert(&cert_plan, cert, None)?,
            CaOrCert::Cert { ca_dir } => {
                if cert_plan.self_signed {
                    save_cert(&cert_plan, cert, None)?
                } else {
                    let ca = load_ca(ca_dir)?;
                    save_cert(&cert_plan, cert, Some(ca))?
                }
            }
        }
    }

    Ok(())
}

fn save_cert(
    plan: &CertPlan,
    cert: Certificate,
    ca: Option<Certificate>,
) -> anyhow::Result<()> {
    let pem = if let Some(ca) = ca {
        cert.serialize_pem_with_signer(&ca)?
    } else {
        cert.serialize_pem()?
    };
    let basepath = plan.output_path.join(&plan.name);
    let key = cert.serialize_private_key_pem();
    let cert_path = basepath.with_extension("pem");
    let key_path = basepath.with_extension("key");
    if (cert_path.exists() || key_path.exists()) && !plan.force {
        anyhow::bail!("file exists");
    }
    let mut cert_file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&cert_path)
        .with_context(|| format!("can't open file {:?}", cert_path))?;
    cert_file
        .write(pem.as_bytes())
        .with_context(|| format!("can't write file {:?}", cert_path))?;
    let mut key_file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&key_path)
        .with_context(|| format!("can't write file {:?}", key_path))?;
    key_file
        .write(key.as_bytes())
        .with_context(|| format!("can't write file {:?}", key_path))?;
    Ok(())
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
