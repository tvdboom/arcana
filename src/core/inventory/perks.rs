use crate::core::inventory::effects::Effect;
use crate::core::inventory::equipment::Kind;
use crate::core::inventory::modifiers::Modifier;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Perk {
    /// Name of the perk (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the perk corresponds to
    pub image: String,

    /// Kind of perk (determines the theme or unlock requirements, if applicable)
    pub kind: Kind,

    /// Level of the perk
    pub level: u32,

    /// Passive modifiers that are always applied
    pub modifiers: Vec<Modifier>,

    /// Passive effects that are always applied
    pub effects: Vec<Effect>,
}

impl Perk {
    pub fn description(&self) -> String {
        format!("This is a test description for {}", self.name)
    }
}
