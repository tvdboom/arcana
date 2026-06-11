use crate::core::inventory::abilities::AbilityKind;
use crate::core::inventory::equipment::Debuff;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Effect {
    /// Enrages your pet, boosting its stats.
    BeastFrenzy {
        attack_modifier: f32,
        attack_speed_modifier: f32,
        duration: f32,
    },

    /// Causes the target's to have a guaranteed chance to miss.
    Blind {
        miss_chance: f32,
        duration: f32,
    },

    /// Deals damage over time.
    Burn {
        damage_per_sec: u32,
        duration: f32,
    },

    /// Grants a brief window where abilities cost X% less mana to cast.
    Clearcasting {
        reduction: f32,
        duration: f32,
    },

    /// Deals a percentage of the auto-attack's damage to the enemy pet as well.
    Cleave {
        damage_pct: f32,
        duration: f32,
    },

    /// Places a mark that explodes for damage after some time.
    Curse {
        damage: u32,
        timer: u32,
    },

    /// Instant damage
    Damage {
        amount: u32,
    },

    /// Grants a low percentage chance on hitting to instantly reset all active ability cooldowns.
    EchoStruck {
        proc_chance: f32,
    },

    /// Increases overall physical and magical damage output by a fixed percentage.
    Empower {
        damage_bonus_pct: f32,
        duration: f32,
    },

    /// Increases the user's critical hit chance for a short combat window.
    Focus {
        crit_chance_bonus: f32,
        duration: f32,
    },

    /// Increases armor.
    Fortify {
        armor_modifier: f32,
        duration: f32,
    },

    /// Reduces the target's attack speed.
    Freeze {
        attack_speed_reduction: f32,
        duration: f32,
    },

    /// Increases initiative.
    Haste {
        initiative_modifier: f32,
        duration: f32,
    },

    /// Completely binds the target to their coordinate position, preventing dodge steps.
    Immobilize {
        duration: f32,
    },

    /// Restores a flat percentage of health on every auto-attack swing that connects.
    Lifesteal {
        percentage: f32,
    },

    /// Destroys a portion of the target's current mana pool.
    ManaBurn {
        amount: u32,
    },

    /// Recover mana.
    ManaFlow {
        amount_per_sec: u32,
        duration: f32,
    },

    /// Siphons a percentage of the target's current mana pool directly back into your own.
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
        damage_per_sec: u32,
        duration: f32,
    },

    /// Instantly cleanses and removes all negative status debuffs currently acting on the unit.
    Purge,

    /// Slowly regenerates health over time; ideal for natural recovery spells or recovery items.
    Regen {
        heal_per_sec: u32,
        duration: f32,
    },

    /// Silences the target, completely preventing them from firing off active abilities.
    Silence {
        duration: f32,
    },

    /// Redirects a portion of all incoming damage taken by the player directly into their pet.
    SoulLink {
        damage_transfer_pct: f32,
        duration: f32,
    },

    /// Interrupts the enemy loop and completely freezes their attack timers.
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

    /// Intentionally warps time to reduce the target's initiative.
    TimeWarp {
        initiative_modifier: f32,
        duration: f32,
    },

    /// Amplifies all incoming damage the target receives by reducing their structural resistances.
    Vulnerability {
        damage_taken_multiplier: f32,
        duration: f32,
    },

    /// Increase/decrease the target's attack.
    AttackModifier {
        amount: i32,
        duration: f32,
    },

    /// Increase/decrease the target's defense.
    DefenseModifier {
        amount: i32,
        duration: f32,
    },

    /// Increase/decrease the target's initiative.
    InitiativeModifier {
        amount: i32,
        duration: f32,
    },

    /// Increase/decrease the target's attack speed.
    AttackSpeedModifier {
        percentage: f32,
        duration: f32,
    },

    /// Increase/decrease the target's critical chance.
    CritChanceModifier {
        percentage: f32,
        duration: f32,
    },

    /// Increase/decrease magical effects of a specific ability kind.
    AbilityKindModifier {
        kind: AbilityKind,
        reduction: f32,
        duration: f32,
    },

    /// Increase/decrease effects of a debuff.
    DebuffModifier {
        debuff: Debuff,
        reduction: f32,
        duration: f32,
    },
}

impl Effect {
    pub fn to_short_string(&self) -> String {
        format!("+1 px")
    }
}
