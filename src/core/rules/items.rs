use crate::core::player::{ConsumableType, Item, Slot, StatModifiers};

pub struct ItemDatabase;

impl ItemDatabase {
    pub fn get_shop_items() -> Vec<Item> {
        vec![
            // Helmets
            Item {
                name: "Iron Helmet".to_string(),
                slot: Slot::Helmet,
                modifiers: StatModifiers { strength: 1, armor: 8, ..Default::default() },
                cost: 40,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Leather Cowl".to_string(),
                slot: Slot::Helmet,
                modifiers: StatModifiers { dexterity: 1, armor: 4, ..Default::default() },
                cost: 30,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Wizard Hat".to_string(),
                slot: Slot::Helmet,
                modifiers: StatModifiers { intelligence: 1, armor: 2, ..Default::default() },
                cost: 35,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            // Armor
            Item {
                name: "Steel Plate Armor".to_string(),
                slot: Slot::Armor,
                modifiers: StatModifiers { strength: 2, armor: 25, ..Default::default() },
                cost: 100,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Leather Brigandine".to_string(),
                slot: Slot::Armor,
                modifiers: StatModifiers { dexterity: 2, armor: 12, ..Default::default() },
                cost: 80,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Mage Robes".to_string(),
                slot: Slot::Armor,
                modifiers: StatModifiers { intelligence: 3, armor: 6, ..Default::default() },
                cost: 90,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            // Boots
            Item {
                name: "Iron Greaves".to_string(),
                slot: Slot::Boots,
                modifiers: StatModifiers { strength: 1, armor: 6, ..Default::default() },
                cost: 30,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Light Leather Boots".to_string(),
                slot: Slot::Boots,
                modifiers: StatModifiers { dexterity: 1, armor: 3, ..Default::default() },
                cost: 25,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Silk Slippers".to_string(),
                slot: Slot::Boots,
                modifiers: StatModifiers { intelligence: 1, armor: 1, ..Default::default() },
                cost: 25,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            // Weapons
            Item {
                name: "Iron Broadsword".to_string(),
                slot: Slot::Weapon,
                modifiers: StatModifiers { strength: 2, physical_power: 12, ..Default::default() },
                cost: 65,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Assassin Dagger".to_string(),
                slot: Slot::Weapon,
                modifiers: StatModifiers { dexterity: 2, physical_power: 8, ..Default::default() },
                cost: 55,
                is_two_handed: false,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Steel Greataxe".to_string(),
                slot: Slot::Weapon,
                modifiers: StatModifiers { strength: 4, physical_power: 24, ..Default::default() },
                cost: 110,
                is_two_handed: true,
                is_consumable: false,
                consumable_type: None,
            },
            Item {
                name: "Elder Staff".to_string(),
                slot: Slot::Weapon,
                modifiers: StatModifiers { intelligence: 4, spell_power: 18, ..Default::default() },
                cost: 95,
                is_two_handed: true,
                is_consumable: false,
                consumable_type: None,
            },
            // Consumables
            Item {
                name: "Minor Health Potion".to_string(),
                slot: Slot::Consumable,
                modifiers: StatModifiers::default(),
                cost: 10,
                is_two_handed: false,
                is_consumable: true,
                consumable_type: Some(ConsumableType::HealthPotion),
            },
            Item {
                name: "Minor Mana Potion".to_string(),
                slot: Slot::Consumable,
                modifiers: StatModifiers::default(),
                cost: 10,
                is_two_handed: false,
                is_consumable: true,
                consumable_type: Some(ConsumableType::ManaPotion),
            },
            Item {
                name: "Elixir of Strength".to_string(),
                slot: Slot::Consumable,
                modifiers: StatModifiers::default(),
                cost: 30,
                is_two_handed: false,
                is_consumable: true,
                consumable_type: Some(ConsumableType::ElixirOfStrength),
            },
        ]
    }
}
