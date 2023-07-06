use rcgen::CertificateParams;
use time::{ext::NumericalDuration, OffsetDateTime};

use crate::cli::CertPlan;

use self::{dn::DN, san::SanFile};

pub mod dn;
pub mod san;

pub fn create_csr(plan: &CertPlan, dn: DN, san: SanFile) -> CertificateParams {
    let mut csr = CertificateParams::default();
    csr.distinguished_name = dn.into();
    csr.subject_alt_names = san.into();
    csr.not_before = OffsetDateTime::now_utc();
    let date = csr.not_before;
    csr.not_after = date + plan.expiry_days.days();
    csr
}
