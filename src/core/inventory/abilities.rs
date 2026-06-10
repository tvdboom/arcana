use crate::core::inventory::effects::Effect;
use crate::core::inventory::equipment::Kind;
use crate::core::inventory::modifiers::Modifier;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Ability {
    /// Name of the ability (matches the english name)
    /// Lowercase with space -> underscore matches the language key for name
    /// Lowercase with space -> underscore and ends with _desc matches the description key
    pub name: String,

    /// Name of the image the ability corresponds to
    pub image: String,

    /// Kind of ability
    pub kind: Kind,

    /// Level of the ability
    pub level: u32,

    /// How much mana this ability costs
    pub mana_cost: u32,

    /// Flat heal/damage stat before modifiers
    pub base: u32,

    /// How heavily the scaling attribute affects base (e.g., 1.5x INT)
    pub scaling_factor: f32,

    /// The ability cooldown (in seconds)
    pub cooldown: f32,

    /// Whether this ability applies to only the player or also his pet
    pub is_aoe: bool,

    /// Modifiers applied when activated
    pub modifiers: Vec<Modifier>,

    /// Effects applied when hitting
    pub effects: Vec<Effect>,
}

impl Ability {
    pub fn description(&self) -> String {
        format!("This is a test description for {}", self.name)
    }
}
