use crate::core::catalog::abilities::Ability;
use crate::core::catalog::artifacts::Artifact;
use crate::core::catalog::consumables::Consumable;
use crate::core::catalog::equipment::Equipment;
use crate::core::catalog::perks::Perk;
use crate::core::catalog::weapons::Weapon;
use crate::core::catalog::wearables::Wearable;
use crate::core::monsters::Monster;
use std::sync::OnceLock;

static ABILITIES: OnceLock<Vec<Ability>> = OnceLock::new();
static PERKS: OnceLock<Vec<Perk>> = OnceLock::new();
static WEAPONS: OnceLock<Vec<Weapon>> = OnceLock::new();
static WEARABLE: OnceLock<Vec<Wearable>> = OnceLock::new();
static CONSUMABLES: OnceLock<Vec<Consumable>> = OnceLock::new();
static EQUIPMENT: OnceLock<Vec<Equipment>> = OnceLock::new();
static ARTIFACTS: OnceLock<Vec<Artifact>> = OnceLock::new();
static MONSTERS: OnceLock<Vec<Monster>> = OnceLock::new();

pub fn all_monsters() -> &'static [Monster] {
    MONSTERS.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/monsters.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse monsters.ron: {}", e))
    })
}

pub fn all_abilities() -> &'static [Ability] {
    ABILITIES.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/abilities.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse abilities.ron: {}", e))
    })
}

pub fn all_perks() -> &'static [Perk] {
    PERKS.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/perks.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse perks.ron: {}", e))
    })
}

pub fn all_weapons() -> &'static [Weapon] {
    WEAPONS.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/weapons.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse weapons.ron: {}", e))
    })
}

pub fn all_wearables() -> &'static [Wearable] {
    WEARABLE.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/wearables.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse wearables.ron: {}", e))
    })
}

pub fn all_consumables() -> &'static [Consumable] {
    CONSUMABLES.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/consumables.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse consumables.ron: {}", e))
    })
}

pub fn all_artifacts() -> &'static [Artifact] {
    ARTIFACTS.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/artifacts.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse artifacts.ron: {}", e))
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
        for consumable in all_consumables() {
            items.push(Equipment::Consumable(consumable.clone()));
        }
        for artifact in all_artifacts() {
            items.push(Equipment::Artifact(artifact.clone()));
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

pub fn get_artifact(name: &str) -> Option<Artifact> {
    all_artifacts().iter().find(|a| a.name == name).cloned()
}

pub fn get_equipment(name: &str) -> Option<Equipment> {
    all_equipment().iter().find(|e| e.name() == name).cloned()
}

pub fn get_monster(name: &str) -> Option<Monster> {
    all_monsters().iter().find(|m| m.name == name).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_catalogs() {
        let mns = all_monsters();
        assert!(!mns.is_empty(), "Monsters catalog is empty");

        let abs = all_abilities();
        assert!(!abs.is_empty(), "Abilities catalog is empty");

        let pks = all_perks();
        assert!(!pks.is_empty(), "Perks catalog is empty");

        let wps = all_weapons();
        assert!(!wps.is_empty(), "Weapons catalog is empty");

        let arm = all_wearables();
        assert!(!arm.is_empty(), "Wearable catalog is empty");

        let con = all_consumables();
        assert!(!con.is_empty(), "Consumable catalog is empty");

        let art = all_artifacts();
        assert!(!art.is_empty(), "Artifact catalog is empty");
    }
}
