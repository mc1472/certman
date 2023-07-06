use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::Context;
use rcgen::DistinguishedName;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};

use crate::cli::CertPlan;

#[derive(Deserialize, Serialize, PartialEq, Eq)]
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

pub fn get_dn_file(plan: &CertPlan) -> anyhow::Result<DN> {
    let path = plan.output_path.join(&plan.name).with_extension("dn.toml");
    if path.exists() {
        let mut data = String::new();
        File::open(&path)?.read_to_string(&mut data)?;
        toml::from_str(&data)
            .with_context(|| format!("Can't parse dn from file {path:?}"))
    } else {
        anyhow::bail!("dn doesn't exist")
    }
}

pub fn write_dn_file(plan: &CertPlan, dn: &DN) -> anyhow::Result<()> {
    let path = plan.output_path.join(&plan.name).with_extension("dn.toml");
    if let Some(file_dn) = get_dn_file(plan).ok() {
        if dn == &file_dn {
            return Ok(());
        }
    }
    if path.exists() && !plan.force {
        return Ok(());
    }
    let data = toml::to_string_pretty(dn)?;
    File::create(&path)?.write_all(data.as_bytes())?;
    Ok(())
}

pub fn get_dn_interactive(
    plan: &CertPlan,
    rl: &mut DefaultEditor,
) -> anyhow::Result<DN> {
    if !plan.interactive {
        anyhow::bail!("interaction disabled");
    }
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
