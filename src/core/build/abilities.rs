use crate::core::build::effects::Effect;
use crate::core::build::equipment::Kind;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Ability {
    /// Name of the ability (matches the english name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the ability corresponds to
    pub image: String,

    /// Level of the ability
    pub level: u32,

    /// Kind of ability
    pub kind: Kind,

    /// How much mana this ability costs
    pub mana_cost: u32,

    /// The ability cooldown (in seconds)
    pub cooldown: f32,

    /// Whether the ability applies to self or the enemy
    pub on_self: bool,

    /// Whether this ability applies only to the player or also his pet
    pub is_aoe: bool,

    /// Effects applied when hitting
    pub effects: Vec<Effect>,
}

impl Ability {
    pub fn description(
        &self,
        language: Language,
        localization: &Localization,
    ) -> String {
        let value_label = if self.kind == Kind::Holy {
            "Heal"
        } else {
            "Dmg"
        };
        let mut line =
            format!("Type: {} | Cost: {} MP: {}", self.kind, value_label, self.mana_cost,);
        if self.cooldown > 0.0 {
            line.push_str(&format!(" | CD: {}s", self.cooldown));
        }

        let mut sub_parts = Vec::new();
        for e in &self.effects {
            sub_parts.push(e.description(language, localization));
        }
        if !sub_parts.is_empty() {
            line.push_str(&format!("\n{}", sub_parts.join(" | ")));
        }
        line
    }
}
