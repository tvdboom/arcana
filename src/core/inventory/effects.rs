use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Effect {
    /// Reduces the target's physical armor value.
    ArmorShred { reduction: u32, duration: f32 },

    /// Enrages your pet, boosting its attack speed, movement, and physical scale.
    BeastFrenzy { attack_speed_bonus: f32, duration: f32 },

    /// Causes the target's next X attacks to have a guaranteed chance to miss.
    Blind { miss_chance: f32, duration: f32 },

    /// Deals physical damage over time as the target bleeds out.
    Bleed { damage_per_sec: u32, duration: f32 },

    /// Deals elemental fire damage over time; can ignite nearby flammable elements.
    Burn { damage_per_sec: u32, duration: f32 },

    /// Strikes the primary target and arcs a chain of lightning out to X nearby enemies.
    ChainArc { jumps: u32, damage_decay_pct: f32 },

    /// Converts a portion of raw elemental damage dealt into a freezing explosion.
    ChillBlast { radius: f32, frost_damage: u32 },

    /// Grants a brief window where active magic abilities cost 0 mana to cast.
    Clearcasting { duration: f32 },

    /// Deals a percentage of the auto-attack's damage to all targets within a short radius.
    Cleave { radius: f32, damage_pct: f32 },

    /// Forces a defeated enemy unit to detonate, dealing damage based on their maximum HP.
    CorpseExplosion { radius: f32, scaling_pct_of_max_hp: f32 },

    /// Places a ticking mark that explodes for massive damage once the target is struck 3 times.
    DoomCurse { stacks_required: u32, explosion_damage: u32 },

    /// Grants a low percentage chance on hitting to instantly reset all active ability cooldowns.
    EchoStruck { proc_chance: f32 },

    /// Increases overall physical and magical damage output by a fixed percentage.
    Empower { damage_bonus_pct: f32, duration: f32 },

    /// Instantly executes an enemy below a specific health threshold, dealing massive true damage.
    Execute { health_threshold_pct: f32, base_true_damage: u32 },

    /// Increases the user's critical hit chance for a short combat window.
    Focus { crit_chance_bonus: f32, duration: f32 },

    /// Multiplies the user's base armor, reinforcing them against heavy incoming blows.
    Fortify { armor_bonus_pct: f32, duration: f32 },

    /// Locks the target in place and drastically reduces their auto-attack interval speed.
    Freeze { attack_speed_reduction: f32, duration: f32 },

    /// Accelerates the internal initiative clock ticker of the player or their active pet.
    Haste { initiative_bonus: f32, duration: f32 },

    /// Completely binds the target to their coordinate position, preventing dodge steps.
    Immobilize { duration: f32 },

    /// Restores a flat percentage of health on every auto-attack swing that connects.
    Lifesteal { percentage: f32 },

    /// Destroys a portion of the target's current mana pool, dealing true damage equal to the mana lost.
    ManaBurn { amount: u32 },

    /// Siphons a percentage of the target's current mana pool directly back into your own.
    Manasteal { percentage: f32 },

    /// Grants a brief window of total invulnerability where the user cannot take damage or be debuffed.
    MonarchShield { duration: f32 },

    /// Deals nature damage over time; slows down target healing received while active.
    Poison { damage_per_sec: u32, duration: f32 },

    /// Instantly cleanses and removes all negative status debuffs currently acting on the unit.
    Purge,

    /// Slowly regenerates health over time; ideal for natural recovery spells or recovery items.
    Regen { heal_per_sec: u32, duration: f32 },

    /// Silences the target, completely preventing them from firing off active abilities.
    Silence { duration: f32 },

    /// Redirects a portion of all incoming damage taken by the player directly into their active pet.
    SoulLink { damage_transfer_pct: f32, duration: f32 },

    /// Interrupts the enemy loop and completely freezes their attack timers.
    Stun { duration: f32 },

    /// Forces enemies to prioritize attacking the User's active pet instead of the player.
    Taunt { duration: f32 },

    /// Spikes out thorny roots that reflect a flat amount of physical damage back to anyone who strikes you.
    Thorns { damage_reflected: u32, duration: f32 },

    /// Intentionally warps time to reduce the target's clock tick/initiative progression speed.
    TimeWarp { initiative_reduction: f32, duration: f32 },

    /// Amplifies all incoming damage the target receives by reducing their structural resistances.
    Vulnerability { damage_taken_multiplier: f32, duration: f32 },

    /// Weakens the target's posture, lowering their base attack power rating.
    Weaken { attack_power_reduction: u32, duration: f32 },
}
