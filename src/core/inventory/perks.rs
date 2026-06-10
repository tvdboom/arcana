use crate::core::inventory::abilities::AbilityKind;
use crate::core::inventory::effects::Effect;
use crate::core::inventory::modifiers::Modifier;

#[derive(Debug, Clone)]
pub struct Perk {
    /// Name of the perk (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the perk corresponds to
    pub image: String,

    /// Description key in the language files
    pub desc_key: String,

    /// Kind of perk (determines the theme or unlock requirements, if applicable)
    pub kind: AbilityKind,

    /// Level of the perk
    pub level: u32,

    /// Passive modifiers that are always applied
    pub modifiers: Vec<Modifier>,
    
    /// Passive effects that are always applied
    pub effect: Vec<Effect>,
}
