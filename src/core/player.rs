use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Race {
    #[default]
    Human,
    Elf,
    Orc,
    Dwarf,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Class {
    #[default]
    Warrior,
    Rogue,
    Mage,
    Druid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AjahColor {
    Yellow,
    Green,
    Red,
    Blue,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transformation {
    Vampire,
    Werewolf,
    Lich,
    Gargoyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Helmet,
    Armor,
    Boots,
    Weapon,
    Consumable,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct StatModifiers {
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub charisma: i32,
    pub armor: i32,
    pub physical_power: i32,
    pub spell_power: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub slot: Slot,
    pub modifiers: StatModifiers,
    pub cost: u32,
    pub is_two_handed: bool,
    pub is_consumable: bool,
    pub consumable_type: Option<ConsumableType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumableType {
    HealthPotion,
    ManaPotion,
    ElixirOfStrength,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Ability {
    pub name: String,
    pub mana_cost: u32,
    pub cooldown_seconds: f32,
    pub description: String,
    pub effect: AbilityEffect,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AbilityEffect {
    Damage { base: i32, strength_scale: f32, intel_scale: f32 },
    Heal { base: i32, intel_scale: f32 },
    Shield { base: i32, intel_scale: f32, duration_seconds: f32 },
    Debuff { duration_seconds: f32, damage_per_tick: i32 },
    Buff { duration_seconds: f32, strength_bonus: i32 },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Pet {
    pub name: String,
    pub pet_type: PetType,
    pub level: u32,
    pub strength: u32,
    pub dexterity: u32,
    pub vitality: u32,
    pub max_health: i32,
    pub current_health: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetType {
    Wolf,
    Bear,
    Falcon,
    Imp,
    BabyDragon,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Stats {
    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,
    pub charisma: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct EquippedGear {
    pub helmet: Option<Item>,
    pub armor: Option<Item>,
    pub boots: Option<Item>,
    pub main_hand: Option<Item>,
    pub off_hand: Option<Item>,
}

#[derive(Resource, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Character {
    pub name: String,
    pub level: u32,
    pub xp: u32,
    pub ap: u32,
    pub max_ap: u32,
    pub race: Race,
    pub class: Class,
    pub ajah: Option<AjahColor>,
    pub transformation: Option<Transformation>,
    pub base_stats: Stats,
    pub spent_stats: Stats,
    pub current_health: i32,
    pub current_mana: i32,
    pub gold: u32,
    pub inventory: Vec<Item>,
    pub gear: EquippedGear,
    pub abilities: Vec<Ability>,       // general pool learned
    pub equipped_abilities: Vec<String>, // name references of up to 3 equipped
    pub equipped_consumables: Vec<Item>, // up to 2 consumables
    pub pet: Option<Pet>,
}

impl Character {
    pub fn new(name: String, race: Race, class: Class, ajah: Option<AjahColor>) -> Self {
        let mut base_stats = Stats::default();
        match race {
            Race::Human => {
                base_stats.strength = 11;
                base_stats.dexterity = 11;
                base_stats.intelligence = 11;
                base_stats.charisma = 11;
            }
            Race::Elf => {
                base_stats.strength = 8;
                base_stats.dexterity = 12;
                base_stats.intelligence = 14;
                base_stats.charisma = 10;
            }
            Race::Orc => {
                base_stats.strength = 14;
                base_stats.dexterity = 10;
                base_stats.intelligence = 9;
                base_stats.charisma = 8;
            }
            Race::Dwarf => {
                base_stats.strength = 12;
                base_stats.dexterity = 9;
                base_stats.intelligence = 10;
                base_stats.charisma = 9;
            }
        }

        let mut character = Self {
            name,
            level: 1,
            xp: 0,
            ap: 10,
            max_ap: 10,
            race,
            class,
            ajah,
            transformation: None,
            base_stats,
            spent_stats: Stats::default(),
            current_health: 100,
            current_mana: 50,
            gold: 150,
            inventory: Vec::new(),
            gear: EquippedGear::default(),
            abilities: Vec::new(),
            equipped_abilities: Vec::new(),
            equipped_consumables: Vec::new(),
            pet: None,
        };

        // Initialize starting equipment and skills
        character.init_class_defaults();
        character.current_health = character.max_health();
        character.current_mana = character.max_mana();
        character
    }

    pub fn init_class_defaults(&mut self) {
        // Starting items and skills based on class
        match self.class {
            Class::Warrior => {
                let sword = Item {
                    name: "Rusty Sword".to_string(),
                    slot: Slot::Weapon,
                    modifiers: StatModifiers { strength: 1, physical_power: 5, ..default() },
                    cost: 30,
                    is_two_handed: false,
                    is_consumable: false,
                    consumable_type: None,
                };
                let shield = Item {
                    name: "Wooden Shield".to_string(),
                    slot: Slot::Weapon,
                    modifiers: StatModifiers { armor: 10, ..default() },
                    cost: 20,
                    is_two_handed: false,
                    is_consumable: false,
                    consumable_type: None,
                };
                self.gear.main_hand = Some(sword);
                self.gear.off_hand = Some(shield);

                let heavy_strike = Ability {
                    name: "Heavy Strike".to_string(),
                    mana_cost: 0,
                    cooldown_seconds: 4.0,
                    description: "A mighty swing dealing physical damage scaling with Strength.".to_string(),
                    effect: AbilityEffect::Damage { base: 12, strength_scale: 1.5, intel_scale: 0.0 },
                };
                let shield_block = Ability {
                    name: "Shield Block".to_string(),
                    mana_cost: 5,
                    cooldown_seconds: 8.0,
                    description: "Raises your shield, creating a shield absorbing damage.".to_string(),
                    effect: AbilityEffect::Shield { base: 15, intel_scale: 0.0, duration_seconds: 4.0 },
                };
                self.abilities.push(heavy_strike);
                self.abilities.push(shield_block);
                self.equipped_abilities.push("Heavy Strike".to_string());
                self.equipped_abilities.push("Shield Block".to_string());
            }
            Class::Rogue => {
                let dagger = Item {
                    name: "Rusty Dagger".to_string(),
                    slot: Slot::Weapon,
                    modifiers: StatModifiers { dexterity: 1, physical_power: 3, ..default() },
                    cost: 25,
                    is_two_handed: false,
                    is_consumable: false,
                    consumable_type: None,
                };
                self.gear.main_hand = Some(dagger.clone());
                self.gear.off_hand = Some(dagger);

                let swift_slash = Ability {
                    name: "Swift Slash".to_string(),
                    mana_cost: 0,
                    cooldown_seconds: 2.0,
                    description: "Fast strike with high base critical rating potential.".to_string(),
                    effect: AbilityEffect::Damage { base: 8, strength_scale: 0.5, intel_scale: 0.0 },
                };
                let poison_blade = Ability {
                    name: "Poison Blade".to_string(),
                    mana_cost: 10,
                    cooldown_seconds: 6.0,
                    description: "Coats weapon in poison, dealing damage over time.".to_string(),
                    effect: AbilityEffect::Debuff { duration_seconds: 5.0, damage_per_tick: 4 },
                };
                self.abilities.push(swift_slash);
                self.abilities.push(poison_blade);
                self.equipped_abilities.push("Swift Slash".to_string());
                self.equipped_abilities.push("Poison Blade".to_string());
            }
            Class::Mage => {
                let staff = Item {
                    name: "Apprentice Staff".to_string(),
                    slot: Slot::Weapon,
                    modifiers: StatModifiers { intelligence: 2, spell_power: 8, ..default() },
                    cost: 40,
                    is_two_handed: true,
                    is_consumable: false,
                    consumable_type: None,
                };
                self.gear.main_hand = Some(staff);

                let fireball = Ability {
                    name: "Fireball".to_string(),
                    mana_cost: 15,
                    cooldown_seconds: 5.0,
                    description: "Shoot a blazing fireball scaling with Intelligence.".to_string(),
                    effect: AbilityEffect::Damage { base: 15, strength_scale: 0.0, intel_scale: 2.0 },
                };
                let mana_shield = Ability {
                    name: "Mana Shield".to_string(),
                    mana_cost: 10,
                    cooldown_seconds: 10.0,
                    description: "A defensive bubble absorbing damage scaling with Intelligence.".to_string(),
                    effect: AbilityEffect::Shield { base: 20, intel_scale: 1.5, duration_seconds: 5.0 },
                };
                self.abilities.push(fireball);
                self.abilities.push(mana_shield);
                self.equipped_abilities.push("Fireball".to_string());
                self.equipped_abilities.push("Mana Shield".to_string());

                // Apply Ajah bonus spells
                if let Some(color) = &self.ajah {
                    match color {
                        AjahColor::Yellow => {
                            self.abilities.push(Ability {
                                name: "Heal".to_string(),
                                mana_cost: 12,
                                cooldown_seconds: 6.0,
                                description: "Restores health scaling with Intelligence (Yellow Ajah).".to_string(),
                                effect: AbilityEffect::Heal { base: 20, intel_scale: 2.0 },
                            });
                            self.equipped_abilities.push("Heal".to_string());
                        }
                        AjahColor::Green => {
                            self.abilities.push(Ability {
                                name: "Chain Lightning".to_string(),
                                mana_cost: 20,
                                cooldown_seconds: 7.0,
                                description: "Electrifying arcs dealing high damage scaling with Intelligence (Green Ajah).".to_string(),
                                effect: AbilityEffect::Damage { base: 22, strength_scale: 0.0, intel_scale: 2.2 },
                            });
                            self.equipped_abilities.push("Chain Lightning".to_string());
                        }
                        AjahColor::Red => {
                            self.abilities.push(Ability {
                                name: "Fire Blast".to_string(),
                                mana_cost: 18,
                                cooldown_seconds: 4.5,
                                description: "Explosive flame wave scaling with Intelligence (Red Ajah).".to_string(),
                                effect: AbilityEffect::Damage { base: 18, strength_scale: 0.0, intel_scale: 2.1 },
                            });
                            self.equipped_abilities.push("Fire Blast".to_string());
                        }
                        AjahColor::Blue => {
                            self.abilities.push(Ability {
                                name: "Frostbolt".to_string(),
                                mana_cost: 10,
                                cooldown_seconds: 5.5,
                                description: "Launches a freezing projectile scaling with Intelligence (Blue Ajah).".to_string(),
                                effect: AbilityEffect::Damage { base: 12, strength_scale: 0.0, intel_scale: 1.6 },
                            });
                            self.equipped_abilities.push("Frostbolt".to_string());
                        }
                    }
                }
            }
            Class::Druid => {
                let club = Item {
                    name: "Oak Club".to_string(),
                    slot: Slot::Weapon,
                    modifiers: StatModifiers { strength: 1, physical_power: 4, ..default() },
                    cost: 25,
                    is_two_handed: false,
                    is_consumable: false,
                    consumable_type: None,
                };
                self.gear.main_hand = Some(club);

                let strike = Ability {
                    name: "Primal Strike".to_string(),
                    mana_cost: 0,
                    cooldown_seconds: 3.5,
                    description: "Melee strike infused with nature's fury.".to_string(),
                    effect: AbilityEffect::Damage { base: 10, strength_scale: 1.0, intel_scale: 0.0 },
                };
                let regrowth = Ability {
                    name: "Regrowth".to_string(),
                    mana_cost: 15,
                    cooldown_seconds: 8.0,
                    description: "Infuses the target with natural healing.".to_string(),
                    effect: AbilityEffect::Heal { base: 15, intel_scale: 1.5 },
                };
                self.abilities.push(strike);
                self.abilities.push(regrowth);
                self.equipped_abilities.push("Primal Strike".to_string());
                self.equipped_abilities.push("Regrowth".to_string());

                // Set up starting pet (defaults to Wolf)
                self.pet = Some(Pet {
                    name: "Greyfang".to_string(),
                    pet_type: PetType::Wolf,
                    level: 1,
                    strength: 8,
                    dexterity: 8,
                    vitality: 8,
                    max_health: 60,
                    current_health: 60,
                });
            }
        }

        // Add starting potions
        self.equipped_consumables.push(Item {
            name: "Minor Health Potion".to_string(),
            slot: Slot::Consumable,
            modifiers: StatModifiers::default(),
            cost: 10,
            is_two_handed: false,
            is_consumable: true,
            consumable_type: Some(ConsumableType::HealthPotion),
        });
        self.equipped_consumables.push(Item {
            name: "Minor Mana Potion".to_string(),
            slot: Slot::Consumable,
            modifiers: StatModifiers::default(),
            cost: 10,
            is_two_handed: false,
            is_consumable: true,
            consumable_type: Some(ConsumableType::ManaPotion),
        });
    }

    // Formulas & Attribute Conversions
    pub fn strength(&self) -> u32 {
        let mut val = self.base_stats.strength + self.spent_stats.strength;
        if self.transformation == Some(Transformation::Werewolf) {
            val = (val as f32 * 1.30) as u32;
        }
        val
    }

    pub fn dexterity(&self) -> u32 {
        let mut val = self.base_stats.dexterity + self.spent_stats.dexterity;
        if let Some(t) = &self.transformation {
            match t {
                Transformation::Vampire => val = (val as f32 * 1.20) as u32,
                Transformation::Gargoyle => val = (val as f32 * 0.80) as u32,
                _ => {}
            }
        }
        val
    }

    pub fn intelligence(&self) -> u32 {
        let mut val = self.base_stats.intelligence + self.spent_stats.intelligence;
        if self.transformation == Some(Transformation::Lich) {
            val = (val as f32 * 1.25) as u32;
        }
        val
    }

    pub fn charisma(&self) -> u32 {
        self.base_stats.charisma + self.spent_stats.charisma
    }

    pub fn max_health(&self) -> i32 {
        let mut hp = (self.strength() * 15) as i32 + 50;
        if self.race == Race::Orc {
            hp = (hp as f32 * 1.10) as i32; // Orc passive health/durability
        }
        hp
    }

    pub fn max_mana(&self) -> i32 {
        (self.intelligence() * 10) as i32 + 20
    }

    pub fn armor(&self) -> i32 {
        let mut arm = (self.strength() * 2) as i32;
        // Add gear armor
        if let Some(helmet) = &self.gear.helmet { arm += helmet.modifiers.armor; }
        if let Some(armor) = &self.gear.armor { arm += armor.modifiers.armor; }
        if let Some(boots) = &self.gear.boots { arm += boots.modifiers.armor; }
        if let Some(mh) = &self.gear.main_hand { arm += mh.modifiers.armor; }
        if let Some(oh) = &self.gear.off_hand { arm += oh.modifiers.armor; }

        if self.class == Class::Warrior {
            arm = (arm as f32 * 1.10) as i32;
        }
        if let Some(Transformation::Gargoyle) = &self.transformation {
            arm = (arm as f32 * 1.40) as i32;
        }
        arm
    }

    pub fn evasion_rate(&self) -> f32 {
        let mut rate = self.dexterity() as f32 * 0.005; // 0.5% per point
        if self.class == Class::Rogue {
            rate += 0.10;
        }
        if rate > 0.50 { rate = 0.50; } // Cap at 50%
        rate
    }

    pub fn accuracy_rate(&self) -> f32 {
        1.0 + (self.dexterity() as f32 * 0.005)
    }

    pub fn critical_rate(&self) -> f32 {
        self.dexterity() as f32 * 0.003
    }

    pub fn gold_work_mult(&self) -> f32 {
        1.0 + (self.charisma() as f32 * 0.05)
    }

    pub fn shop_discount_rate(&self) -> f32 {
        let mut rate = self.charisma() as f32 * 0.005;
        if rate > 0.40 { rate = 0.40; } // Cap at 40%
        rate
    }

    pub fn rest_efficiency_health(&self) -> f32 {
        if let Some(Transformation::Vampire) = &self.transformation {
            0.5 // Vampires rest at 50% healing efficiency
        } else {
            1.0
        }
    }

    pub fn rest_efficiency_mana(&self) -> f32 {
        if self.race == Race::Elf {
            1.1 // Elves rest at +10% mana efficiency
        } else {
            1.0
        }
    }
}
