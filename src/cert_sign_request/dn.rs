use std::{
    fs::File,
    io::{Read, Write},
};

use rcgen::DistinguishedName;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};

use super::CertError;
use crate::RunPlan;

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

pub fn get_dn(
    rl: &mut DefaultEditor,
    plan: &RunPlan,
) -> Result<DistinguishedName, CertError> {
    if let Some(dn_file_path) = &plan.dn_file {
        if dn_file_path.exists() {
            let mut string = String::new();
            File::open(dn_file_path)?.read_to_string(&mut string)?;
            toml::from_str::<DN>(&string)
                .map_err(|err| err.into())
                .map(|dn| dn.into())
        } else if plan.user_read_dn {
            let dn = get_dn_interactive(rl)?;
            if plan.write_dn {
                let data = toml::to_string_pretty(&dn)?;
                File::create(dn_file_path)?.write_all(data.as_bytes())?;
            }
            Ok(dn.into())
        } else {
            Err(CertError::InvalidPlan(
                "dn file not found and user input disabled",
            ))
        }
    } else if plan.user_read_dn {
        let dn = get_dn_interactive(rl)?;
        Ok(dn.into())
    } else {
        Err(CertError::InvalidPlan(
            "dn file not found and user input disabled",
        ))
    }
}

pub fn get_dn_interactive(rl: &mut DefaultEditor) -> Result<DN, CertError> {
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
