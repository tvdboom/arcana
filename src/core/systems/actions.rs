use rand::Rng;

use crate::core::player::{Character, Item, Pet, Slot, Transformation};
use crate::core::rules::abilities::AbilityDatabase;
use crate::core::rules::items::ItemDatabase;

pub struct ActionManager;

impl ActionManager {
    pub fn spend_ap(character: &mut Character, amount: u32) -> Result<(), String> {
        if character.ap < amount {
            return Err("Insufficient Action Points (AP).".to_string());
        }
        character.ap -= amount;
        Ok(())
    }

    pub fn train_stat(character: &mut Character, stat: &str) -> Result<(), String> {
        let gold_cost = character.level * 20;
        if character.gold < gold_cost {
            return Err("Insufficient Gold to train attributes.".to_string());
        }
        Self::spend_ap(character, 1)?;
        character.gold -= gold_cost;

        match stat {
            "strength" => character.spent_stats.strength += 1,
            "dexterity" => character.spent_stats.dexterity += 1,
            "intelligence" => character.spent_stats.intelligence += 1,
            "charisma" => character.spent_stats.charisma += 1,
            _ => return Err("Unknown attribute.".to_string()),
        }

        // Re-scale current health/mana values based on new stats
        character.current_health = character.max_health().min(character.current_health + 15);
        character.current_mana = character.max_mana().min(character.current_mana + 10);
        Ok(())
    }

    pub fn work(character: &mut Character) -> Result<u32, String> {
        Self::spend_ap(character, 2)?;
        let base_gold = 40;
        let earned = (base_gold as f32 * character.gold_work_mult()) as u32;
        character.gold += earned;
        Ok(earned)
    }

    pub fn rest(character: &mut Character) -> Result<(), String> {
        Self::spend_ap(character, 1)?;
        
        let hp_heal = (character.max_health() as f32 * 0.40 * character.rest_efficiency_health()) as i32;
        let mp_restore = (character.max_mana() as f32 * 0.40 * character.rest_efficiency_mana()) as i32;

        character.current_health = (character.current_health + hp_heal).min(character.max_health());
        character.current_mana = (character.current_mana + mp_restore).min(character.max_mana());
        Ok(())
    }

    pub fn craft_upgrade(character: &mut Character, slot: Slot) -> Result<(), String> {
        // Find equipped item in that slot and improve its main attributes
        let gold_cost = character.level * 30;
        let mut actual_cost = gold_cost;
        if character.race == crate::core::player::Race::Dwarf {
            actual_cost = (gold_cost as f32 * 0.80) as u32; // Dwarven discount
        }

        if character.gold < actual_cost {
            return Err("Insufficient Gold to craft upgrades.".to_string());
        }

        Self::spend_ap(character, 2)?;
        character.gold -= actual_cost;

        let modify_item = |item: &mut Item| {
            item.name = format!("{} +", item.name.trim_end_matches(" +"));
            item.modifiers.strength += 1;
            item.modifiers.dexterity += 1;
            item.modifiers.intelligence += 1;
            item.modifiers.armor += 5;
            item.modifiers.physical_power += 3;
            item.modifiers.spell_power += 3;
            item.cost += 15;
        };

        match slot {
            Slot::Helmet => {
                if let Some(ref mut item) = character.gear.helmet {
                    modify_item(item);
                } else {
                    return Err("No helmet equipped to upgrade.".to_string());
                }
            }
            Slot::Armor => {
                if let Some(ref mut item) = character.gear.armor {
                    modify_item(item);
                } else {
                    return Err("No armor equipped to upgrade.".to_string());
                }
            }
            Slot::Boots => {
                if let Some(ref mut item) = character.gear.boots {
                    modify_item(item);
                } else {
                    return Err("No boots equipped to upgrade.".to_string());
                }
            }
            Slot::Weapon => {
                if let Some(ref mut item) = character.gear.main_hand {
                    modify_item(item);
                } else {
                    return Err("No main-hand weapon equipped to upgrade.".to_string());
                }
            }
            _ => return Err("Invalid equipment slot.".to_string()),
        }

        Ok(())
    }

    pub fn study_ability(character: &mut Character, ability_name: &str) -> Result<(), String> {
        let gold_cost = character.level * 40;
        if character.gold < gold_cost {
            return Err("Insufficient Gold to study spells.".to_string());
        }
        
        // Check if already learned
        if character.abilities.iter().any(|a| a.name == ability_name) {
            return Err("Ability already learned.".to_string());
        }

        let ability = AbilityDatabase::find_ability(ability_name)
            .ok_or_else(|| "Ability not found in database.".to_string())?;

        Self::spend_ap(character, 2)?;
        character.gold -= gold_cost;
        character.abilities.push(ability);

        Ok(())
    }

    pub fn trigger_quest(character: &mut Character) -> Result<(String, String), String> {
        Self::spend_ap(character, 3)?;
        
        let mut rng = rand::rng();
        let roll = rng.random_range(0..100);

        let (title, log) = match roll {
            0..=20 => {
                // Gold Reward
                let bonus = (character.charisma() * 10) + rng.random_range(20..80);
                character.gold += bonus;
                ("Treasures of the Crypt".to_string(), 
                 format!("You enter an ancestral tomb and bypass its traps. You recover a cache of ancient coins containing {} gold!", bonus))
            }
            21..=40 => {
                // High Quality Loot
                let shop_items = ItemDatabase::get_shop_items();
                let random_item = &shop_items[rng.random_range(0..shop_items.len())];
                character.inventory.push(random_item.clone());
                ("A Shady Merchant".to_string(), 
                 format!("You assist a merchant whose cart broke down. In gratitude, they gift you a high quality item: {}.", random_item.name))
            }
            41..=55 => {
                // Transformation chance: Vampire
                if character.transformation.is_none() {
                    character.transformation = Some(Transformation::Vampire);
                    ("Bite of the Night".to_string(), 
                     "During an investigation in a shrouded forest, a mysterious noble attacks and drinks your blood. You wake up with an insatiable hunger and dark runes on your flesh. You have transformed into a Vampire!".to_string())
                } else {
                    let gold_find = 50;
                    character.gold += gold_find;
                    ("Shadowy Pathways".to_string(), "You explore dark pathways and find a hidden purse containing 50 gold.".to_string())
                }
            }
            56..=70 => {
                // Transformation chance: Werewolf
                if character.transformation.is_none() {
                    character.transformation = Some(Transformation::Werewolf);
                    // Werewolves cannot wear armor or helmets - unequip them
                    let _ = character.gear.helmet.take().map(|i| character.inventory.push(i));
                    let _ = character.gear.armor.take().map(|i| character.inventory.push(i));
                    ("Curse of the Full Moon".to_string(), 
                     "A massive wild beast bites you during a hunting patrol. Under the moonlight, your skin turns to fur and muscles swell. You have transformed into a Werewolf! You can no longer wear heavy armor or cast magical spells.".to_string())
                } else {
                    let gold_find = 50;
                    character.gold += gold_find;
                    ("Howling Hills".to_string(), "You hear distant howling but safely complete your hunt, harvesting 50 gold worth of pelts.".to_string())
                }
            }
            71..=85 => {
                // Quest Pet reward
                if let Some(pet) = character.pet.as_mut() {
                    let pet_name = pet.name.clone();
                    pet.level_up();
                    ("Pet Training".to_string(), 
                     format!("You spend the day training with {}. Your pet levels up and grows stronger!", pet_name))
                } else {
                    let pet_roll = rng.random_range(0..3);
                    let (name, pet_type) = match pet_roll {
                        0 => ("Swiftwing".to_string(), crate::core::player::PetType::Falcon),
                        1 => ("Sparky".to_string(), crate::core::player::PetType::Imp),
                        _ => ("Draco".to_string(), crate::core::player::PetType::BabyDragon),
                    };
                    let pet_name = name.clone();
                    character.pet = Some(Pet {
                        name,
                        pet_type,
                        level: 1,
                        strength: 6,
                        dexterity: 10,
                        vitality: 6,
                        max_health: 50,
                        current_health: 50,
                    });
                    ("The Lost Companion".to_string(), 
                     format!("You rescue a rare creature trapped in hunter nets. It bows and pledges loyalty to you. You gained a Pet: {} ({:?})!", pet_name, pet_type))
                }
            }
            _ => {
                // Transformation chance: Lich or Gargoyle
                if character.transformation.is_none() {
                    if rng.random_bool(0.5) {
                        character.transformation = Some(Transformation::Lich);
                        ("The Forbidden Tome".to_string(), 
                         "You discover an ancient text containing dark necromantic spells. Releasing its locks turns your flesh cold and eyes into blue fire. You have transformed into a Lich!".to_string())
                    } else {
                        character.transformation = Some(Transformation::Gargoyle);
                        ("Touch of Stone".to_string(), 
                         "A stone relic curses your veins, hardening your skin into cold grey rock. You have transformed into a Gargoyle!".to_string())
                    }
                } else {
                    let gold_find = 100;
                    character.gold += gold_find;
                    ("Ancient Ruins".to_string(), "You discover gold plaques inside ancient ruins worth 100 gold.".to_string())
                }
            }
        };

        Ok((title, log))
    }
}
