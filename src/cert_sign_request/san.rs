use std::{
    fs::File,
    io::{Read, Write},
    net::IpAddr,
};

use rcgen::SanType;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};

use super::CertError;
use crate::RunPlan;

#[derive(Debug, Serialize, Deserialize, Default)]
struct SanFile {
    dns: Option<Vec<String>>,
    ip: Option<Vec<IpAddr>>,
    email: Option<Vec<String>>,
    uri: Option<Vec<String>>,
}

impl SanFile {
    fn into_san_type(self) -> Vec<SanType> {
        let mut san = Vec::new();
        self.dns
            .map(|dns| dns.into_iter().map(SanType::DnsName))
            .into_iter()
            .for_each(|val| san.extend(val));
        self.ip
            .map(|ip| ip.into_iter().map(SanType::IpAddress))
            .into_iter()
            .for_each(|val| san.extend(val));
        self.email
            .map(|dns| dns.into_iter().map(SanType::Rfc822Name))
            .into_iter()
            .for_each(|val| san.extend(val));
        self.uri
            .map(|dns| dns.into_iter().map(SanType::URI))
            .into_iter()
            .for_each(|val| san.extend(val));
        san
    }
}

pub fn get_sans(
    rl: &mut DefaultEditor,
    plan: &RunPlan,
) -> Result<Vec<SanType>, CertError> {
    if let Some(san_file_path) = &plan.san_file {
        if san_file_path.exists() {
            let mut data = String::new();
            File::open(san_file_path)?.read_to_string(&mut data)?;
            let table: SanFile = toml::from_str(&data)?;
            Ok(table.into_san_type())
        } else if plan.user_read_san {
            let dns_san = get_subject_alt_names_interactive(rl)?;
            let san = SanFile {
                dns: Some(dns_san),
                ..Default::default()
            };
            if plan.write_san {
                let data = toml::to_string_pretty(&san)?;
                File::create(san_file_path)?.write_all(data.as_bytes())?;
            }
            Ok(san.into_san_type())
        } else {
            Err(CertError::InvalidPlan(
                "san file not found and user input disabled",
            ))
        }
    } else if plan.user_read_san {
        let dns_san = get_subject_alt_names_interactive(rl)?;
        let san = SanFile {
            dns: Some(dns_san),
            ..Default::default()
        };
        Ok(san.into_san_type())
    } else {
        Err(CertError::InvalidPlan(
            "san file not found and user input disabled",
        ))
    }
}

fn get_subject_alt_names_interactive(
    rl: &mut DefaultEditor,
) -> Result<Vec<String>, CertError> {
    let subject_alt_name = rl
        .readline("Subject alt names (comma seperated) > ")?
        .split(',')
        .map(|dns| dns.to_owned())
        .collect::<Vec<_>>();

    Ok(subject_alt_name)
}
