//! Artifact catalog records and their localized names and descriptions.

use crate::core::catalog::equipment::Kind;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub image: String,
    pub kind: Kind,
    pub level: u32,
    pub price: u32,
}

impl Artifact {
    /// Performs the description operation.
    pub fn description(&self, _language: Language, _localization: &Localization) -> String {
        format!("{}", self.kind)
    }

    /// Performs the full description operation.
    pub fn full_description(
        &self,
        _language: Language,
        _localization: &Localization,
    ) -> Vec<String> {
        vec![format!("[{}] {}", self.kind.to_string().to_lowercase(), self.kind)]
    }
}
