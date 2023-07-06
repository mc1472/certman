use std::{
    fs::File,
    io::{Read, Write},
    net::IpAddr,
};

use anyhow::Context;
use rcgen::SanType;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};

use crate::cli::CertPlan;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SanFile {
    dns: Option<Vec<String>>,
    ip: Option<Vec<IpAddr>>,
    email: Option<Vec<String>>,
    uri: Option<Vec<String>>,
}

impl From<SanFile> for Vec<SanType> {
    fn from(value: SanFile) -> Self {
        let mut san = Vec::new();
        value
            .dns
            .map(|dns| dns.into_iter().map(SanType::DnsName))
            .into_iter()
            .for_each(|val| san.extend(val));
        value
            .ip
            .map(|ip| ip.into_iter().map(SanType::IpAddress))
            .into_iter()
            .for_each(|val| san.extend(val));
        value
            .email
            .map(|dns| dns.into_iter().map(SanType::Rfc822Name))
            .into_iter()
            .for_each(|val| san.extend(val));
        value
            .uri
            .map(|dns| dns.into_iter().map(SanType::URI))
            .into_iter()
            .for_each(|val| san.extend(val));
        san
    }
}

pub fn get_san_file(plan: &CertPlan) -> anyhow::Result<SanFile> {
    let path = plan.output_path.join(&plan.name).with_extension("san.toml");
    if path.exists() {
        let mut data = String::new();
        File::open(&path)?.read_to_string(&mut data)?;
        toml::from_str(&data)
            .with_context(|| format!("Can't parse san file from file {path:?}"))
    } else {
        anyhow::bail!("san file doesn't exist")
    }
}

pub fn write_san_file(plan: &CertPlan, san: &SanFile) -> anyhow::Result<()> {
    let path = plan.output_path.join(&plan.name).with_extension("san.toml");
    if let Some(file_dn) = get_san_file(plan).ok() {
        if san == &file_dn {
            return Ok(());
        }
    }
    if path.exists() && !plan.force {
        return Ok(());
    }
    let data = toml::to_string_pretty(san)?;
    File::create(&path)?.write_all(data.as_bytes())?;
    Ok(())
}

pub fn get_san_interactive(
    plan: &CertPlan,
    rl: &mut DefaultEditor,
) -> anyhow::Result<SanFile> {
    if !plan.interactive {
        anyhow::bail!("interaction disabled");
    }
    let subject_alt_name = rl
        .readline("Subject alt names (comma seperated) > ")?
        .split(',')
        .map(|dns| dns.to_owned())
        .collect::<Vec<_>>();

    Ok(SanFile {
        dns: Some(subject_alt_name),
        ..Default::default()
    })
}
