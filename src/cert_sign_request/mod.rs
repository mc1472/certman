use std::io;

use rcgen::{CertificateParams, IsCa, KeyUsagePurpose};
use rustyline::DefaultEditor;
use thiserror::Error;
use time::{Duration, OffsetDateTime};

use crate::RunPlan;

mod dn;
mod san;

#[derive(Debug, Error)]
pub enum CertError {
    #[error("An io error {0:?}")]
    Ioerror(#[from] io::Error),
    #[error("Can't parse toml file {0:?}")]
    CantParseTomlFile(#[from] toml::de::Error),
    #[error("Can't serialize toml file {0:?}")]
    CantSerializeTomlFile(#[from] toml::ser::Error),
    #[error("Can't read from stdin {0:?}")]
    FailedToReadFromStdin(#[from] rustyline::error::ReadlineError),
    #[error("{0}")]
    InvalidPlan(&'static str),
    #[error("Other errors {0:?}")]
    Other(#[from] anyhow::Error),
}

pub fn create_csr(
    plan: &RunPlan,
    rl: &mut DefaultEditor,
) -> Result<CertificateParams, CertError> {
    let mut csr = CertificateParams::default();

    let dn = dn::get_dn(rl, plan)?;
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
        let san = san::get_sans(rl, plan)?;
        csr.subject_alt_names = san;
        csr.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
    }

    csr.distinguished_name = dn;

    Ok(csr)
}
