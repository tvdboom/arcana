use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Effect {
    /// Reduces the target's physical armor value.
    ArmorShred {
        reduction: u32,
        duration: f32,
    },

    /// Enrages your pet, boosting its attack speed, movement, and physical scale.
    BeastFrenzy {
        attack_speed_bonus: f32,
        duration: f32,
    },

    /// Causes the target's next X attacks to have a guaranteed chance to miss.
    Blind {
        miss_chance: f32,
        duration: f32,
    },

    /// Deals physical damage over time as the target bleeds out.
    Bleed {
        damage_per_sec: u32,
        duration: f32,
    },

    /// Deals elemental fire damage over time; can ignite nearby flammable elements.
    Burn {
        damage_per_sec: u32,
        duration: f32,
    },

    /// Strikes the primary target and arcs a chain of lightning out to X nearby enemies.
    ChainArc {
        jumps: u32,
        damage_decay_pct: f32,
    },

    /// Converts a portion of raw elemental damage dealt into a freezing explosion.
    ChillBlast {
        radius: f32,
        frost_damage: u32,
    },

    /// Grants a brief window where active magic abilities cost 0 mana to cast.
    Clearcasting {
        duration: f32,
    },

    /// Deals a percentage of the auto-attack's damage to all targets within a short radius.
    Cleave {
        radius: f32,
        damage_pct: f32,
    },

    /// Forces a defeated enemy unit to detonate, dealing damage based on their maximum HP.
    CorpseExplosion {
        radius: f32,
        scaling_pct_of_max_hp: f32,
    },

    /// Places a ticking mark that explodes for massive damage once the target is struck 3 times.
    DoomCurse {
        stacks_required: u32,
        explosion_damage: u32,
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

    /// Instantly executes an enemy below a specific health threshold, dealing massive true damage.
    Execute {
        health_threshold_pct: f32,
        base_true_damage: u32,
    },

    /// Increases the user's critical hit chance for a short combat window.
    Focus {
        crit_chance_bonus: f32,
        duration: f32,
    },

    /// Multiplies the user's base armor, reinforcing them against heavy incoming blows.
    Fortify {
        armor_bonus_pct: f32,
        duration: f32,
    },

    /// Locks the target in place and drastically reduces their auto-attack interval speed.
    Freeze {
        attack_speed_reduction: f32,
        duration: f32,
    },

    /// Accelerates the internal initiative clock ticker of the player or their active pet.
    Haste {
        initiative_bonus: f32,
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

    /// Destroys a portion of the target's current mana pool, dealing true damage equal to the mana lost.
    ManaBurn {
        amount: u32,
    },

    /// Siphons a percentage of the target's current mana pool directly back into your own.
    Manasteal {
        percentage: f32,
    },

    /// Grants a brief window of total invulnerability where the user cannot take damage or be debuffed.
    MonarchShield {
        duration: f32,
    },

    /// Deals nature damage over time; slows down target healing received while active.
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

    /// Redirects a portion of all incoming damage taken by the player directly into their active pet.
    SoulLink {
        damage_transfer_pct: f32,
        duration: f32,
    },

    /// Interrupts the enemy loop and completely freezes their attack timers.
    Stun {
        duration: f32,
    },

    /// Forces enemies to prioritize attacking the User's active pet instead of the player.
    Taunt {
        duration: f32,
    },

    /// Spikes out thorny roots that reflect a flat amount of physical damage back to anyone who strikes you.
    Thorns {
        damage_reflected: u32,
        duration: f32,
    },

    /// Intentionally warps time to reduce the target's clock tick/initiative progression speed.
    TimeWarp {
        initiative_reduction: f32,
        duration: f32,
    },

    /// Amplifies all incoming damage the target receives by reducing their structural resistances.
    Vulnerability {
        damage_taken_multiplier: f32,
        duration: f32,
    },

    /// Weakens the target's posture, lowering their base attack power rating.
    Weaken {
        attack_power_reduction: u32,
        duration: f32,
    },

    /// Reduces damage taken from fire-based effects or spells.
    FireResistance {
        percentage: f32,
    },

    /// Reduces damage taken from frost-based effects or slows.
    FrostResistance {
        percentage: f32,
    },

    /// Reduces damage taken from lightning-based effects or stun durations.
    LightningResistance {
        percentage: f32,
    },

    /// Reduces damage taken from poison-based effects or toxins.
    PoisonResistance {
        percentage: f32,
    },

    /// Reduces damage taken from shadow-based effects or curses.
    ShadowResistance {
        percentage: f32,
    },

    /// Reduces damage taken from holy-based effects or radiant damage.
    HolyResistance {
        percentage: f32,
    },

    /// Reduces damage taken from bleed-based effects or physical lacerations.
    BleedResistance {
        percentage: f32,
    },
}

impl Effect {
    pub fn to_short_string(&self) -> String {
        match self {
            Effect::ArmorShred { reduction, duration } => format!("Shred: {} ({}s)", reduction, duration),
            Effect::BeastFrenzy { attack_speed_bonus, duration } => format!("Frenzy: +{:.0}% AS ({}s)", attack_speed_bonus * 100.0, duration),
            Effect::Blind { miss_chance, duration } => format!("Blind: {:.0}% ({}s)", miss_chance * 100.0, duration),
            Effect::Bleed { damage_per_sec, duration } => format!("Bleed: {}/s ({}s)", damage_per_sec, duration),
            Effect::Burn { damage_per_sec, duration } => format!("Burn: {}/s ({}s)", damage_per_sec, duration),
            Effect::ChainArc { jumps, damage_decay_pct } => format!("Chain: {} jumps (-{:.0}%)", jumps, damage_decay_pct * 100.0),
            Effect::ChillBlast { radius, frost_damage } => format!("Chill: {} dmg (r:{})", frost_damage, radius),
            Effect::Clearcasting { duration } => format!("Clearcast ({}s)", duration),
            Effect::Cleave { radius, damage_pct } => format!("Cleave: {:.0}% (r:{})", damage_pct * 100.0, radius),
            Effect::CorpseExplosion { radius, scaling_pct_of_max_hp } => format!("Corpse Boom: {:.0}% HP (r:{})", scaling_pct_of_max_hp * 100.0, radius),
            Effect::DoomCurse { stacks_required, explosion_damage } => format!("Doom: {} dmg ({} stacks)", explosion_damage, stacks_required),
            Effect::EchoStruck { proc_chance } => format!("Echo: {:.0}%", proc_chance * 100.0),
            Effect::Empower { damage_bonus_pct, duration } => format!("Empower: +{:.0}% ({}s)", damage_bonus_pct * 100.0, duration),
            Effect::Execute { health_threshold_pct, base_true_damage } => format!("Execute: <{:.0}% ({} dmg)", health_threshold_pct * 100.0, base_true_damage),
            Effect::Focus { crit_chance_bonus, duration } => format!("Focus: +{:.0}% Crit ({}s)", crit_chance_bonus * 100.0, duration),
            Effect::Fortify { armor_bonus_pct, duration } => format!("Fortify: +{:.0}% Armor ({}s)", armor_bonus_pct * 100.0, duration),
            Effect::Freeze { attack_speed_reduction, duration } => format!("Freeze: -{:.0}% AS ({}s)", attack_speed_reduction * 100.0, duration),
            Effect::Haste { initiative_bonus, duration } => format!("Haste: +{:.0}% Init ({}s)", initiative_bonus * 100.0, duration),
            Effect::Immobilize { duration } => format!("Immobilize ({}s)", duration),
            Effect::Lifesteal { percentage } => format!("Lifesteal: {:.0}%", percentage * 100.0),
            Effect::ManaBurn { amount } => format!("Mana Burn: {}", amount),
            Effect::Manasteal { percentage } => format!("Manasteal: {:.0}%", percentage * 100.0),
            Effect::MonarchShield { duration } => format!("Shield ({}s)", duration),
            Effect::Poison { damage_per_sec, duration } => format!("Poison: {}/s ({}s)", damage_per_sec, duration),
            Effect::Purge => "Purge".to_string(),
            Effect::Regen { heal_per_sec, duration } => format!("Regen: {}/s ({}s)", heal_per_sec, duration),
            Effect::Silence { duration } => format!("Silence ({}s)", duration),
            Effect::SoulLink { damage_transfer_pct, duration } => format!("Soul Link: {:.0}% ({}s)", damage_transfer_pct * 100.0, duration),
            Effect::Stun { duration } => format!("Stun ({}s)", duration),
            Effect::Taunt { duration } => format!("Taunt ({}s)", duration),
            Effect::Thorns { damage_reflected, duration } => format!("Thorns: {} dmg ({}s)", damage_reflected, duration),
            Effect::TimeWarp { initiative_reduction, duration } => format!("Time Warp: -{:.0}% Init ({}s)", initiative_reduction * 100.0, duration),
            Effect::Vulnerability { damage_taken_multiplier, duration } => format!("Vulnerable: +{:.0}% ({}s)", (damage_taken_multiplier - 1.0) * 100.0, duration),
            Effect::Weaken { attack_power_reduction, duration } => format!("Weaken: -{} Atk ({}s)", attack_power_reduction, duration),
            Effect::FireResistance { percentage } => format!("Fire Res: {:.0}%", percentage * 100.0),
            Effect::FrostResistance { percentage } => format!("Frost Res: {:.0}%", percentage * 100.0),
            Effect::LightningResistance { percentage } => format!("Lightning Res: {:.0}%", percentage * 100.0),
            Effect::PoisonResistance { percentage } => format!("Poison Res: {:.0}%", percentage * 100.0),
            Effect::ShadowResistance { percentage } => format!("Shadow Res: {:.0}%", percentage * 100.0),
            Effect::HolyResistance { percentage } => format!("Holy Res: {:.0}%", percentage * 100.0),
            Effect::BleedResistance { percentage } => format!("Bleed Res: {:.0}%", percentage * 100.0),
        }
    }
}
