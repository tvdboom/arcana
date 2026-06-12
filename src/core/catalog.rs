use crate::core::build::abilities::Ability;
use crate::core::build::wearable::Wearable;
use crate::core::build::equipment::Equipment;
use crate::core::build::perks::Perk;
use crate::core::build::weapons::Weapon;
use std::sync::OnceLock;

static ABILITIES: OnceLock<Vec<Ability>> = OnceLock::new();
static PERKS: OnceLock<Vec<Perk>> = OnceLock::new();
static WEAPONS: OnceLock<Vec<Weapon>> = OnceLock::new();
static ARMOR: OnceLock<Vec<Wearable>> = OnceLock::new();
static EQUIPMENT: OnceLock<Vec<Equipment>> = OnceLock::new();

pub fn all_abilities() -> &'static [Ability] {
    ABILITIES.get_or_init(|| {
        let ron_str = include_str!("../../assets/inventory/abilities.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse abilities.ron: {}", e))
    })
}

pub fn all_perks() -> &'static [Perk] {
    PERKS.get_or_init(|| {
        let ron_str = include_str!("../../assets/inventory/perks.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse perks.ron: {}", e))
    })
}

pub fn all_weapons() -> &'static [Weapon] {
    WEAPONS.get_or_init(|| {
        let ron_str = include_str!("../../assets/inventory/weapons.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse weapons.ron: {}", e))
    })
}

pub fn all_wearables() -> &'static [Wearable] {
    ARMOR.get_or_init(|| {
        let ron_str = include_str!("../../assets/inventory/wearables.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse wearable.ron: {}", e))
    })
}

pub fn all_equipment() -> &'static [Equipment] {
    EQUIPMENT.get_or_init(|| {
        let mut items = Vec::new();
        for weapon in all_weapons() {
            items.push(Equipment::Weapon(weapon.clone()));
        }
        for wearable in all_wearables() {
            items.push(Equipment::Wearable(wearable.clone()));
        }
        items
    })
}

pub fn get_ability(name: &str) -> Option<Ability> {
    all_abilities().iter().find(|a| a.name == name).cloned()
}

pub fn get_perk(name: &str) -> Option<Perk> {
    all_perks().iter().find(|p| p.name == name).cloned()
}

pub fn get_equipment(name: &str) -> Option<Equipment> {
    all_equipment().iter().find(|e| e.name() == name).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_catalogs() {
        let abs = all_abilities();
        assert!(!abs.is_empty(), "Abilities catalog is empty");

        let pks = all_perks();
        assert!(!pks.is_empty(), "Perks catalog is empty");

        let wps = all_weapons();
        assert!(!wps.is_empty(), "Weapons catalog is empty");

        let arm = all_wearables();
        assert!(!arm.is_empty(), "Armor catalog is empty");
    }
}
