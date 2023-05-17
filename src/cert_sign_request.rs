use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::{Context, Ok};
use rcgen::{
    CertificateParams, DistinguishedName, IsCa, KeyUsagePurpose, SanType,
};
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use toml::Table;

use crate::{utils::exit_with_msg, RunPlan};

pub fn create_csr(
    plan: &RunPlan,
    rl: &mut DefaultEditor,
) -> anyhow::Result<CertificateParams> {
    let mut csr = CertificateParams::default();

    let dn = get_dn(rl, plan)?;
    if plan.ca_or_cert {
        csr.is_ca = IsCa::Ca(rcgen::BasicConstraints::Constrained(1));
        csr.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        let expiry =
            OffsetDateTime::now_utc() + Duration::days(plan.expiry_days);
        csr.not_after = expiry;
    } else {
        let san = get_sans(rl, plan)?;
        csr.subject_alt_names = san;
        csr.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
    }

    csr.distinguished_name = dn;

    Ok(csr)
}

#[derive(Deserialize, Serialize)]
pub struct DN {
    pub country: String,
    pub state_or_province: String,
    pub locality: String,
    pub orgiazation: String,
    pub common_name: String,
}

impl From<DN> for DistinguishedName {
    fn from(value: DN) -> Self {
        let mut dn = Self::new();

        dn.push(rcgen::DnType::CountryName, value.country);
        dn.push(rcgen::DnType::StateOrProvinceName, value.state_or_province);
        dn.push(rcgen::DnType::LocalityName, value.locality);
        dn.push(rcgen::DnType::OrganizationName, value.orgiazation);
        dn.push(rcgen::DnType::CommonName, value.common_name);

        dn
    }
}

fn get_dn(
    rl: &mut DefaultEditor,
    plan: &RunPlan,
) -> anyhow::Result<DistinguishedName> {
    if let Some(dn_file_path) = &plan.dn_file {
        if dn_file_path.exists() {
            let mut string = String::new();
            File::open(dn_file_path)?.read_to_string(&mut string)?;
            toml::from_str::<DN>(&string)
                .with_context(|| {
                    format!("can't Deserialize file {dn_file_path:?}")
                })
                .map(|dn| dn.into())
        } else if plan.user_read_dn {
            let dn = get_dn_interactive(rl)?;
            if plan.write_dn {
                let data = toml::to_string_pretty(&dn)?;
                File::create(dn_file_path)
                    .with_context(|| {
                        format!("can't open file {dn_file_path:?}")
                    })?
                    .write(data.as_bytes())
                    .with_context(|| {
                        format!("can't wirte file {dn_file_path:?}")
                    })?;
            }
            Ok(dn.into())
        } else {
            exit_with_msg("dn file not found and user input disabled")
        }
    } else {
        exit_with_msg("dn file not found and user input disabled")
    }
}

pub fn get_dn_interactive(rl: &mut DefaultEditor) -> anyhow::Result<DN> {
    let country = rl.readline("Country > ")?;
    let state_or_province = rl.readline("State or Province > ")?;
    let locality = rl.readline("Locality > ")?;
    let orgiazation = rl.readline("Orgiazation > ")?;
    let common_name = rl.readline("Common Name > ")?;

    Ok(DN {
        country,
        state_or_province,
        locality,
        orgiazation,
        common_name,
    })
}

pub fn get_sans(
    rl: &mut DefaultEditor,
    plan: &RunPlan,
) -> anyhow::Result<Vec<SanType>> {
    if let Some(san_file_path) = &plan.san_file {
        if san_file_path.exists() {
            let mut data = String::new();
            File::open(san_file_path)
                .with_context(|| format!("Can't open file {san_file_path:?}"))?
                .read_to_string(&mut data)
                .with_context(|| {
                    format!("Can't read file {san_file_path:?}")
                })?;
            let table: Table = toml::from_str(&data).with_context(|| {
                format!("{san_file_path:?} is not valid toml")
            })?;
            println!("{table:#?}");
            return parse_san_toml(&table);
        } else {
            todo!()
        }
    } else {
        todo!()
    }
    todo!()
}

fn parse_san_toml(data: &Table) -> anyhow::Result<Vec<SanType>> {
    let san = Vec::new();
    for (san_type, san_list) in data.iter() {
        match san_type.as_str() {
            "DNS" => {
                san.extend(
                    san_list
                        .as_table()
                        .unwrap()
                        .values()
                        .map(|value| value.as_str().unwrap())
                        .map(SanType::DnsName),
                );
            }
            _ => (),
        }
    }
    Ok(san)
}

pub fn get_subject_alt_names_interactive(
    rl: &mut DefaultEditor,
) -> anyhow::Result<Vec<SanType>> {
    let subject_alt_name = rl
        .readline("Subject alt names (comma seperated) > ")?
        .split(',')
        .map(|dns| SanType::DnsName(dns.to_owned()))
        .collect::<Vec<_>>();

    Ok(subject_alt_name)
}
