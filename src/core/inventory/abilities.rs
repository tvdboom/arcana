use crate::core::inventory::effects::Effect;
use crate::core::inventory::equipment::Kind;
use crate::core::inventory::modifiers::Modifier;
use crate::core::player::Player;
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
    pub fn actual_value(&self, player: &Player) -> u32 {
        let scaling_stat = if self.kind.is_magic() {
            if self.kind == Kind::Holy || self.kind == Kind::Nature {
                player.wisdom() as f32
            } else {
                player.intelligence() as f32
            }
        } else {
            if self.kind == Kind::Assassination || self.kind == Kind::Skirmish {
                player.dexterity() as f32
            } else {
                player.strength() as f32
            }
        };
        (self.base as f32 + scaling_stat * self.scaling_factor).round() as u32
    }

    pub fn description(&self, player: &Player) -> String {
        let value_label = if self.kind == Kind::Holy || self.kind == Kind::Nature {
            "Heal"
        } else {
            "Dmg"
        };
        let mut line = format!(
            "Type: {} | {}: {} | Cost: {} MP",
            self.kind,
            value_label,
            self.actual_value(player),
            self.mana_cost,
        );
        if self.cooldown > 0.0 {
            line.push_str(&format!(" | CD: {}s", self.cooldown));
        }
        
        let mut sub_parts = Vec::new();
        for m in &self.modifiers {
            sub_parts.push(m.to_short_string());
        }
        for e in &self.effects {
            sub_parts.push(e.to_short_string());
        }
        if !sub_parts.is_empty() {
            line.push_str(&format!("\n{}", sub_parts.join(" | ")));
        }
        line
    }
}
