use crate::core::localization::Localization;
use crate::core::player::Attribute;
use crate::core::settings::Language;
use crate::utils::NameFromEnum;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Debug, Clone, Display, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    /// Enrages your pet, boosting its stats.
    BeastFrenzy {
        attack_pct: f32,
        attack_speed_pct: f32,
        duration: f32,
    },

    /// Increases the attack of the target.
    Berserk {
        attack_pct: f32,
        duration: f32,
    },

    /// Increases the damage of the next basic attack.
    Bleed {
        damage_pct: f32,
    },

    /// Causes the target to have an increased chance to miss basic attacks.
    Blind {
        miss_pct: f32,
        duration: f32,
    },

    /// Deals damage over time.
    Burn {
        damage: u32,
        duration: f32,
    },

    /// Grants a brief window where abilities cost less mana to cast.
    Clearcasting {
        reduction_pct: f32,
        duration: f32,
    },

    /// Deals a percentage of the damage to a second target.
    Cleave {
        damage_pct: f32,
        duration: f32,
    },

    /// Places a mark that deals damage after some time.
    Curse {
        damage: u32,
        timer: u32,
    },

    /// Instant damage
    Pierce {
        damage: u32,
    },

    /// Grants a low percentage chance on hitting to instantly reset all active ability cooldowns.
    EchoStruck {
        reset_pct: f32,
    },

    /// Increases overall damage.
    Empower {
        damage_pct: f32,
        duration: f32,
    },

    /// Increases critical strike chance.
    Focus {
        crit_chance_pct: f32,
        duration: f32,
    },

    /// Increases defense.
    Fortify {
        defense_pct: f32,
        duration: f32,
    },

    /// Reduces attack speed.
    Freeze {
        attack_speed_pct: f32,
        duration: f32,
    },

    /// Increases initiative.
    Haste {
        initiative_pct: f32,
        duration: f32,
    },

    /// Instantly heal a percentage of missing health.
    Heal {
        heal_pct: u32,
    },

    /// Prevents the target from dodging attacks or abilities.
    Immobilize {
        duration: f32,
    },

    /// Instantly restores a flat amount of mana.
    InstantMana {
        amount: u32,
    },

    /// Restores a flat percentage of health on inflicted damage.
    Lifesteal {
        percentage: f32,
        duration: f32,
    },

    /// Destroys a portion of the target's current mana pool.
    ManaBurn {
        amount: u32,
    },

    /// Recover a flat amount of mana per sec.
    ManaFlow {
        amount: u32,
        duration: f32,
    },

    /// Siphons a percentage of the target's current mana pool into your own.
    Manasteal {
        percentage: f32,
    },

    /// Grants a brief window of total invulnerability where the target cannot take damage or be debuffed.
    /// The target cannot cast any ability nor attack in this period.
    MonarchShield {
        duration: f32,
    },

    /// Deals damage over time.
    Poison {
        damage: u32,
        duration: f32,
    },

    /// Reduces initiative.
    Paranoia {
        initiative_pct: f32,
        duration: f32,
    },

    /// Removes all negative effects on the target.
    Purge,

    /// Recover a flat amount of health per sec.
    Regen {
        heal: u32,
        duration: f32,
    },

    /// Preventing the target from casting any abilities.
    Silence {
        duration: f32,
    },

    /// Redirects a portion of all incoming damage taken by the player directly into their pet.
    SoulLink {
        damage_transfer_pct: f32,
        duration: f32,
    },

    /// Temporarily boosts a specific attribute.
    StatBoost {
        attribute: Attribute,
        amount: u32,
        duration: f32,
    },

    /// Freezes the target's ability cooldown recovery.
    Stun {
        duration: f32,
    },

    /// Forces enemies to prioritize attacking the target's pet instead of the player.
    Taunt {
        duration: f32,
    },

    /// Spikes out thorny roots that reflect a flat amount of physical damage back to anyone who strikes you.
    Thorns {
        damage_reflected_pct: f32,
        duration: f32,
    },

    /// Amplifies all incoming damage on the target.
    Vulnerability {
        damage_pct: f32,
        duration: f32,
    },
}

impl Effect {
    pub fn description(&self, language: Language, localization: &Localization) -> String {
        let template =
            localization.get(format!("effect.{}", self.to_string().to_lowercase()), language);

        match self {
            Self::BeastFrenzy {
                attack_pct,
                attack_speed_pct,
                duration,
            } => template
                .replace("{attack_pct}", &format!("{attack_pct:+.0}"))
                .replace("{attack_speed_pct}", &format!("{attack_speed_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Berserk {
                attack_pct,
                duration,
            } => template
                .replace("{attack_pct}", &format!("{attack_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Bleed {
                damage_pct,
            } => template.replace("{damage_pct}", &format!("{damage_pct:+.0}")),
            Self::Blind {
                miss_pct,
                duration,
            } => template
                .replace("{miss_pct}", &format!("{miss_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Burn {
                damage,
                duration,
            } => template
                .replace("{damage}", &format!("{damage:+}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Clearcasting {
                reduction_pct,
                duration,
            } => template
                .replace("{reduction_pct}", &format!("{reduction_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Cleave {
                damage_pct,
                duration,
            } => template
                .replace("{damage_pct}", &format!("{damage_pct:.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Curse {
                damage,
                timer,
            } => template
                .replace("{damage}", &format!("{damage}"))
                .replace("{timer}", &format!("{timer}")),
            Self::Pierce {
                damage,
            } => template.replace("{damage}", &format!("{damage}")),
            Self::EchoStruck {
                reset_pct,
            } => template.replace("{reset_pct}", &format!("{reset_pct:+.0}")),
            Self::Empower {
                damage_pct,
                duration,
            } => template
                .replace("{damage_pct}", &format!("{damage_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Focus {
                crit_chance_pct,
                duration,
            } => template
                .replace("{crit_chance_pct}", &format!("{crit_chance_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Fortify {
                defense_pct: armor_pct,
                duration,
            } => template
                .replace("{armor_pct}", &format!("{armor_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Freeze {
                attack_speed_pct,
                duration,
            } => template
                .replace("{attack_speed_pct}", &format!("{attack_speed_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Haste {
                initiative_pct,
                duration,
            } => template
                .replace("{initiative_pct}", &format!("{initiative_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Heal {
                heal_pct,
            } => template.replace("{heal_pct}", &format!("{heal_pct}")),
            Self::Immobilize {
                duration,
            } => template.replace("{duration}", &format!("{duration:.1}")),
            Self::Lifesteal {
                percentage,
                duration,
            } => template
                .replace("{percentage}", &format!("{percentage:.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::ManaBurn {
                amount,
            } => template.replace("{amount}", &format!("{amount}")),
            Self::InstantMana {
                amount,
            } => template.replace("{amount}", &format!("{amount}")),
            Self::StatBoost {
                attribute,
                amount,
                duration,
            } => {
                let key = format!("attribute.{}", attribute.to_lowername());
                let stat_localized = localization.get(&key, language);
                template
                    .replace("{stat}", &stat_localized)
                    .replace("{amount}", &format!("{amount}"))
                    .replace("{duration}", &format!("{duration:.1}"))
            },
            Self::ManaFlow {
                amount,
                duration,
            } => template
                .replace("{amount}", &format!("{amount:+}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Manasteal {
                percentage,
            } => template.replace("{percentage}", &format!("{percentage:.0}")),
            Self::MonarchShield {
                duration,
            } => template.replace("{duration}", &format!("{duration:.1}")),
            Self::Poison {
                damage,
                duration,
            } => template
                .replace("{damage}", &format!("{damage:+}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Paranoia {
                initiative_pct,
                duration,
            } => template
                .replace("{initiative_pct}", &format!("{initiative_pct:+.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Purge => template,
            Self::Regen {
                heal,
                duration,
            } => template
                .replace("{heal}", &format!("{heal:+}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Silence {
                duration,
            } => template.replace("{duration}", &format!("{duration:.1}")),
            Self::SoulLink {
                damage_transfer_pct,
                duration,
            } => template
                .replace("{damage_transfer_pct}", &format!("{damage_transfer_pct:.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Stun {
                duration,
            } => template.replace("{duration}", &format!("{duration:.1}")),
            Self::Taunt {
                duration,
            } => template.replace("{duration}", &format!("{duration:.1}")),
            Self::Thorns {
                damage_reflected_pct,
                duration,
            } => template
                .replace("{damage_reflected_pct}", &format!("{damage_reflected_pct:.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
            Self::Vulnerability {
                damage_pct,
                duration,
            } => template
                .replace("{damage_pct}", &format!("{damage_pct:.0}"))
                .replace("{duration}", &format!("{duration:.1}")),
        }
    }
}
