use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::core::audio::SoundEffect;
use crate::core::player::{Ability, AbilityEffect, Character, ConsumableType, Item, Pet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Combatant {
    pub name: String,
    pub level: u32,
    pub is_player: bool,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mana: i32,
    pub armor: i32,
    pub evasion: f32,
    pub accuracy: f32,
    pub crit_chance: f32,
    
    // Stats for scaling
    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,

    // Auto-attack
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    pub base_damage: i32,

    // Abilities & Consumables
    pub abilities: Vec<AbilityState>,
    pub consumables: Vec<ConsumableState>,

    // Buffs and Debuffs
    pub shield_value: i32,
    pub shield_timer: f32,
    pub poison_timer: f32,
    pub poison_damage: i32,
    pub burn_timer: f32,
    pub burn_damage: i32,
    pub buff_timer: f32,
    pub strength_bonus: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbilityState {
    pub ability: Ability,
    pub cooldown_timer: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsumableState {
    pub item: Item,
    pub used: bool,
}

#[derive(Resource, Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CombatSession {
    pub active: bool,
    pub is_pvp: bool,
    pub player: Option<Combatant>,
    pub player_pet: Option<Combatant>,
    pub opponent: Option<Combatant>,
    pub opponent_pet: Option<Combatant>,
    pub logs: Vec<String>,
    pub victory_state: Option<VictoryState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VictoryState {
    PlayerWins,
    OpponentWins,
}

impl CombatSession {
    pub fn init_hunt(character: &Character, npc_type: &str) -> Self {
        let player = Combatant::from_character(character);
        let player_pet = character.pet.as_ref().map(Combatant::from_pet);

        // Generate NPC
        let mut rng = rand::rng();
        let level = character.level + rng.random_range(0..=1);
        let opponent = Combatant::generate_npc(npc_type, level);

        // NPCs have a 20% chance to have a basic pet (e.g. wild beast) at higher levels
        let opponent_pet = if level >= 3 && rng.random_bool(0.2) {
            Some(Combatant {
                name: "Feral Minion".to_string(),
                level,
                is_player: false,
                hp: (level * 30) as i32 + 20,
                max_hp: (level * 30) as i32 + 20,
                mp: 0,
                max_mana: 0,
                armor: (level * 2) as i32,
                evasion: 0.10,
                accuracy: 1.0,
                crit_chance: 0.05,
                strength: level * 3,
                dexterity: level * 2,
                intelligence: 0,
                attack_cooldown: 2.5,
                attack_timer: 2.5,
                base_damage: (level * 3) as i32 + 2,
                abilities: Vec::new(),
                consumables: Vec::new(),
                shield_value: 0,
                shield_timer: 0.0,
                poison_timer: 0.0,
                poison_damage: 0,
                burn_timer: 0.0,
                burn_damage: 0,
                buff_timer: 0.0,
                strength_bonus: 0,
            })
        } else {
            None
        };

        Self {
            active: true,
            is_pvp: false,
            player: Some(player),
            player_pet,
            opponent: Some(opponent),
            opponent_pet,
            logs: vec!["A dangerous wild encounter begins!".to_string()],
            victory_state: None,
        }
    }

    pub fn tick(&mut self, delta: f32, sfx_writer: &mut MessageWriter<SoundEffect>) {
        if !self.active || self.victory_state.is_some() {
            return;
        }

        // Ticks for primary combatants
        if let (Some(ref mut p), Some(ref mut o)) = (&mut self.player, &mut self.opponent) {
            // Tick abilities cooldowns
            p.tick_cooldowns(delta);
            o.tick_cooldowns(delta);

            // Tick active status effects
            p.tick_statuses(delta, &mut self.logs);
            o.tick_statuses(delta, &mut self.logs);

            // Tick auto attacks
            if p.tick_attack(delta) {
                p.perform_attack(o, &mut self.logs, sfx_writer);
            }
            if o.tick_attack(delta) {
                o.perform_attack(p, &mut self.logs, sfx_writer);
            }

            // Tick pet combat
            if let Some(ref mut p_pet) = &mut self.player_pet {
                p_pet.tick_cooldowns(delta);
                p_pet.tick_statuses(delta, &mut self.logs);
                if p_pet.tick_attack(delta) {
                    p_pet.perform_attack(o, &mut self.logs, sfx_writer);
                }
            }

            if let Some(ref mut o_pet) = &mut self.opponent_pet {
                o_pet.tick_cooldowns(delta);
                o_pet.tick_statuses(delta, &mut self.logs);
                if o_pet.tick_attack(delta) {
                    o_pet.perform_attack(p, &mut self.logs, sfx_writer);
                }
            }

            // Check death
            if p.hp <= 0 {
                self.victory_state = Some(VictoryState::OpponentWins);
                self.logs.push(format!("{} has fallen in battle!", p.name));
                sfx_writer.write(SoundEffect::Defeat);
            } else if o.hp <= 0 {
                self.victory_state = Some(VictoryState::PlayerWins);
                self.logs.push(format!("{} is victorious!", p.name));
                sfx_writer.write(SoundEffect::Victory);
            }
        }
    }

    pub fn cast_ability(
        &mut self,
        by_player: bool,
        index: usize,
        sfx_writer: &mut MessageWriter<SoundEffect>,
    ) -> Result<(), String> {
        let (caster, target, logs) = if by_player {
            (&mut self.player, &mut self.opponent, &mut self.logs)
        } else {
            (&mut self.opponent, &mut self.player, &mut self.logs)
        };

        let c = caster.as_mut().ok_or("Caster not found.")?;
        let t = target.as_mut().ok_or("Target not found.")?;

        if index >= c.abilities.len() {
            return Err("Invalid ability index.".to_string());
        }

        let state = &mut c.abilities[index];
        if state.cooldown_timer > 0.0 {
            return Err("Ability is on cooldown.".to_string());
        }

        let actual_cost = if c.is_player {
            // Apply mana reduction formula
            (state.ability.mana_cost as f32 * (1.0 - c.intelligence as f32 * 0.005)) as i32
        } else {
            state.ability.mana_cost as i32
        };

        if c.mp < actual_cost {
            return Err("Insufficient Mana to activate ability.".to_string());
        }

        c.mp -= actual_cost;
        state.cooldown_timer = state.ability.cooldown_seconds;

        logs.push(format!("{} casted {}!", c.name, state.ability.name));

        match &state.ability.effect {
            AbilityEffect::Damage { base, strength_scale, intel_scale } => {
                let scaling = (c.strength as f32 * strength_scale) + (c.intelligence as f32 * intel_scale);
                let raw_dmg = *base + scaling as i32;
                
                // Roll defense
                let final_dmg = (raw_dmg - t.armor).max(1);
                
                // Shield absorb check
                if t.shield_value > 0 {
                    let absorbed = final_dmg.min(t.shield_value);
                    t.shield_value -= absorbed;
                    logs.push(format!("{}'s shield absorbed {} damage ({} shield left).", t.name, absorbed, t.shield_value));
                    let excess = final_dmg - absorbed;
                    if excess > 0 {
                        t.hp -= excess;
                        logs.push(format!("{} took {} overflow damage.", t.name, excess));
                    }
                } else {
                    t.hp -= final_dmg;
                    logs.push(format!("{} took {} magical/skill damage.", t.name, final_dmg));
                }
                sfx_writer.write(SoundEffect::MagicCast);
            }
            AbilityEffect::Heal { base, intel_scale } => {
                let scaling = c.intelligence as f32 * intel_scale;
                let heal = *base + scaling as i32;
                c.hp = (c.hp + heal).min(c.max_hp);
                logs.push(format!("{} healed for {} HP.", c.name, heal));
                sfx_writer.write(SoundEffect::Heal);
            }
            AbilityEffect::Shield { base, intel_scale, duration_seconds } => {
                let scaling = c.intelligence as f32 * intel_scale;
                c.shield_value = *base + scaling as i32;
                c.shield_timer = *duration_seconds;
                logs.push(format!("{} barriers for {} absorption (lasts {}s).", c.name, c.shield_value, duration_seconds));
                sfx_writer.write(SoundEffect::Shield);
            }
            AbilityEffect::Debuff { duration_seconds, damage_per_tick } => {
                t.poison_timer = *duration_seconds;
                t.poison_damage = *damage_per_tick;
                logs.push(format!("{} afflicted {} with damage over time status effect.", c.name, t.name));
                sfx_writer.write(SoundEffect::MagicCast);
            }
            AbilityEffect::Buff { duration_seconds, strength_bonus } => {
                c.buff_timer = *duration_seconds;
                c.strength_bonus = *strength_bonus;
                logs.push(format!("{} gained physical enhancement (+{} Strength for {}s).", c.name, strength_bonus, duration_seconds));
                sfx_writer.write(SoundEffect::MagicCast);
            }
        }

        Ok(())
    }

    pub fn use_consumable(
        &mut self,
        by_player: bool,
        index: usize,
        sfx_writer: &mut MessageWriter<SoundEffect>,
    ) -> Result<(), String> {
        let caster = if by_player { &mut self.player } else { &mut self.opponent };
        let c = caster.as_mut().ok_or("User not found.")?;

        if index >= c.consumables.len() {
            return Err("Invalid consumable index.".to_string());
        }

        if c.consumables[index].used {
            return Err("Potion is already consumed.".to_string());
        }

        c.consumables[index].used = true;
        let item = &c.consumables[index].item;

        if let Some(t) = item.consumable_type {
            match t {
                ConsumableType::HealthPotion => {
                    let heal = 50;
                    c.hp = (c.hp + heal).min(c.max_hp);
                    self.logs.push(format!("{} consumed Health Potion and healed for {} HP.", c.name, heal));
                    sfx_writer.write(SoundEffect::Heal);
                }
                ConsumableType::ManaPotion => {
                    let mana = 30;
                    c.mp = (c.mp + mana).min(c.max_mana);
                    self.logs.push(format!("{} consumed Mana Potion and restored {} Mana.", c.name, mana));
                    sfx_writer.write(SoundEffect::Heal);
                }
                ConsumableType::ElixirOfStrength => {
                    c.buff_timer = 10.0;
                    c.strength_bonus = 15;
                    self.logs.push(format!("{} drank Elixir (+15 Strength for 10s).", c.name));
                    sfx_writer.write(SoundEffect::Heal);
                }
            }
        }

        Ok(())
    }
}

impl Combatant {
    pub fn from_character(c: &Character) -> Self {
        let mut abilities = Vec::new();
        for eq_name in &c.equipped_abilities {
            if let Some(abi) = c.abilities.iter().find(|a| a.name == *eq_name) {
                abilities.push(AbilityState {
                    ability: abi.clone(),
                    cooldown_timer: 0.0,
                });
            }
        }

        let mut consumables = Vec::new();
        for item in &c.equipped_consumables {
            consumables.push(ConsumableState {
                item: item.clone(),
                used: false,
            });
        }

        // Calculate attack speed: standard speed 3.0s, reduced by Dexterity scaling
        let speed = (3.0 - (c.dexterity() as f32 * 0.02)).max(1.0);
        let base_damage = (c.strength() as f32 * 0.5) as i32 + 5;

        Self {
            name: c.name.clone(),
            level: c.level,
            is_player: true,
            hp: c.current_health,
            max_hp: c.max_health(),
            mp: c.current_mana,
            max_mana: c.max_mana(),
            armor: c.armor(),
            evasion: c.evasion_rate(),
            accuracy: c.accuracy_rate(),
            crit_chance: c.critical_rate(),
            strength: c.strength(),
            dexterity: c.dexterity(),
            intelligence: c.intelligence(),
            attack_cooldown: speed,
            attack_timer: speed,
            base_damage,
            abilities,
            consumables,
            shield_value: 0,
            shield_timer: 0.0,
            poison_timer: 0.0,
            poison_damage: 0,
            burn_timer: 0.0,
            burn_damage: 0,
            buff_timer: 0.0,
            strength_bonus: 0,
        }
    }

    pub fn from_pet(pet: &Pet) -> Self {
        let speed = (2.5 - (pet.dexterity as f32 * 0.02)).max(0.8);
        Self {
            name: format!("{} (Pet)", pet.name),
            level: pet.level,
            is_player: false,
            hp: pet.current_health,
            max_hp: pet.max_health(),
            mp: 0,
            max_mana: 0,
            armor: (pet.dexterity) as i32,
            evasion: pet.evasion_rate(),
            accuracy: pet.accuracy_rate(),
            crit_chance: pet.critical_rate(),
            strength: pet.strength,
            dexterity: pet.dexterity,
            intelligence: 0,
            attack_cooldown: speed,
            attack_timer: speed,
            base_damage: pet.damage(),
            abilities: Vec::new(),
            consumables: Vec::new(),
            shield_value: 0,
            shield_timer: 0.0,
            poison_timer: 0.0,
            poison_damage: 0,
            burn_timer: 0.0,
            burn_damage: 0,
            buff_timer: 0.0,
            strength_bonus: 0,
        }
    }

    pub fn generate_npc(npc_type: &str, level: u32) -> Self {
        let hp = (level * 40) as i32 + 60;
        let mut armor = (level * 3) as i32;
        let mut evasion = 0.05 + (level as f32 * 0.005);
        let mut attack_cooldown = 3.0;
        let mut base_damage = (level * 3) as i32 + 5;
        let mut intelligence = level * 2;
        let mut abilities = Vec::new();

        match npc_type {
            "Goblin" => {
                evasion += 0.15;
                attack_cooldown = 2.0;
                base_damage = (level * 2) as i32 + 4;
            }
            "Bandit" => {
                base_damage = (level * 4) as i32 + 6;
            }
            "Knight" => {
                armor += 15;
                attack_cooldown = 3.5;
                abilities.push(AbilityState {
                    ability: Ability {
                        name: "Shield Defend".to_string(),
                        mana_cost: 0,
                        cooldown_seconds: 6.0,
                        description: "Raises defenses.".to_string(),
                        effect: AbilityEffect::Shield { base: 10 + (level * 2) as i32, intel_scale: 0.0, duration_seconds: 3.0 },
                    },
                    cooldown_timer: 0.0,
                });
            }
            "Mage" => {
                intelligence += 10;
                attack_cooldown = 4.0;
                abilities.push(AbilityState {
                    ability: Ability {
                        name: "Minor Spell".to_string(),
                        mana_cost: 0,
                        cooldown_seconds: 5.0,
                        description: "Casts fireball.".to_string(),
                        effect: AbilityEffect::Damage { base: 12 + (level * 2) as i32, strength_scale: 0.0, intel_scale: 1.2 },
                    },
                    cooldown_timer: 0.0,
                });
            }
            _ => {
                // Beast
                base_damage = (level * 5) as i32 + 7;
            }
        }

        Self {
            name: format!("Level {} {}", level, npc_type),
            level,
            is_player: false,
            hp,
            max_hp: hp,
            mp: 50,
            max_mana: 50,
            armor,
            evasion,
            accuracy: 1.0 + (level as f32 * 0.005),
            crit_chance: 0.05 + (level as f32 * 0.003),
            strength: level * 3,
            dexterity: level * 2,
            intelligence,
            attack_cooldown,
            attack_timer: attack_cooldown,
            base_damage,
            abilities,
            consumables: Vec::new(),
            shield_value: 0,
            shield_timer: 0.0,
            poison_timer: 0.0,
            poison_damage: 0,
            burn_timer: 0.0,
            burn_damage: 0,
            buff_timer: 0.0,
            strength_bonus: 0,
        }
    }

    pub fn tick_cooldowns(&mut self, delta: f32) {
        for state in &mut self.abilities {
            if state.cooldown_timer > 0.0 {
                state.cooldown_timer = (state.cooldown_timer - delta).max(0.0);
            }
        }
    }

    pub fn tick_statuses(&mut self, delta: f32, logs: &mut Vec<String>) {
        // Shield
        if self.shield_timer > 0.0 {
            self.shield_timer -= delta;
            if self.shield_timer <= 0.0 {
                self.shield_value = 0;
                logs.push(format!("{}'s shield expired.", self.name));
            }
        }

        // Buffs
        if self.buff_timer > 0.0 {
            self.buff_timer -= delta;
            if self.buff_timer <= 0.0 {
                self.strength_bonus = 0;
                logs.push(format!("{}'s strength buff expired.", self.name));
            }
        }

        // Poison Dot
        if self.poison_timer > 0.0 {
            let before = self.poison_timer;
            self.poison_timer -= delta;
            // Tick every second block (e.g. crossing integer boundaries)
            if before.floor() > self.poison_timer.floor() {
                self.hp -= self.poison_damage;
                logs.push(format!("{} suffers {} damage from poison status.", self.name, self.poison_damage));
            }
        }
    }

    pub fn tick_attack(&mut self, delta: f32) -> bool {
        self.attack_timer -= delta;
        if self.attack_timer <= 0.0 {
            self.attack_timer = self.attack_cooldown;
            return true;
        }
        false
    }

    pub fn perform_attack(
        &mut self,
        target: &mut Self,
        logs: &mut Vec<String>,
        sfx_writer: &mut MessageWriter<SoundEffect>,
    ) {
        let mut rng = rand::rng();

        // Check evasion
        let evasion_roll = rng.random_range(0.0..1.0);
        if evasion_roll < target.evasion {
            logs.push(format!("{} attacked {} but they EVADED!", self.name, target.name));
            return;
        }

        // Check hit
        let hit_roll = rng.random_range(0.0..1.0);
        if hit_roll > self.accuracy {
            logs.push(format!("{} swung at {} and MISSED!", self.name, target.name));
            return;
        }

        // Damage roll
        let mut dmg = self.base_damage;
        
        // Critical roll
        let crit_roll = rng.random_range(0.0..1.0);
        let mut is_crit = false;
        if crit_roll < self.crit_chance {
            dmg *= 2;
            is_crit = true;
        }

        // Armor reduction
        let final_dmg = (dmg - target.armor).max(1);

        // Deduct shield first
        if target.shield_value > 0 {
            let absorbed = final_dmg.min(target.shield_value);
            target.shield_value -= absorbed;
            logs.push(format!("{}'s shield absorbed {} damage.", target.name, absorbed));
            let excess = final_dmg - absorbed;
            if excess > 0 {
                target.hp -= excess;
                logs.push(format!("{} took {} overflow strike damage.", target.name, excess));
            }
        } else {
            target.hp -= final_dmg;
            if is_crit {
                logs.push(format!("CRITICAL HIT! {} strikes {} for {} damage!", self.name, target.name, final_dmg));
            } else {
                logs.push(format!("{} hits {} for {} damage.", self.name, target.name, final_dmg));
            }
        }

        sfx_writer.write(SoundEffect::PhysicalHit);
    }
}
