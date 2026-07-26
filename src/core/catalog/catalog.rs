//! Catalog asset collections, global lookup helpers, and content invariant tests.

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

/// Performs the all monsters operation.
pub fn all_monsters() -> &'static [Monster] {
    MONSTERS.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/monsters.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse monsters.ron: {}", e))
    })
}

/// Performs the all abilities operation.
pub fn all_abilities() -> &'static [Ability] {
    ABILITIES.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/abilities.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse abilities.ron: {}", e))
    })
}

/// Performs the all perks operation.
pub fn all_perks() -> &'static [Perk] {
    PERKS.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/perks.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse perks.ron: {}", e))
    })
}

/// Performs the all weapons operation.
pub fn all_weapons() -> &'static [Weapon] {
    WEAPONS.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/weapons.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse weapons.ron: {}", e))
    })
}

/// Performs the all wearables operation.
pub fn all_wearables() -> &'static [Wearable] {
    WEARABLE.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/wearables.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse wearables.ron: {}", e))
    })
}

/// Performs the all consumables operation.
pub fn all_consumables() -> &'static [Consumable] {
    CONSUMABLES.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/consumables.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse consumables.ron: {}", e))
    })
}

/// Performs the all artifacts operation.
pub fn all_artifacts() -> &'static [Artifact] {
    ARTIFACTS.get_or_init(|| {
        let ron_str = include_str!("../../../assets/catalog/artifacts.ron");
        ron::from_str(ron_str).unwrap_or_else(|e| panic!("Failed to parse artifacts.ron: {}", e))
    })
}

/// Performs the all equipment operation.
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

/// Returns ability.
pub fn get_ability(name: &str) -> Option<Ability> {
    all_abilities().iter().find(|a| a.name == name).cloned()
}

/// Returns perk.
pub fn get_perk(name: &str) -> Option<Perk> {
    all_perks().iter().find(|p| p.name == name).cloned()
}

/// Returns artifact.
pub fn get_artifact(name: &str) -> Option<Artifact> {
    all_artifacts().iter().find(|a| a.name == name).cloned()
}

/// Returns equipment.
pub fn get_equipment(name: &str) -> Option<Equipment> {
    all_equipment().iter().find(|e| e.name() == name).cloned()
}

/// Returns monster.
pub fn get_monster(name: &str) -> Option<Monster> {
    all_monsters().iter().find(|m| m.name == name).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::catalog::effects::Effect;
    use crate::core::catalog::equipment::Kind;
    use crate::core::catalog::weapons::Category;
    use crate::core::classes::{Ajah, Class};
    use std::collections::HashSet;
    use std::path::Path;
    use strum::IntoEnumIterator;

    /// Performs the effect duration operation.
    fn effect_duration(effect: &Effect) -> f32 {
        match effect {
            Effect::BeastFrenzy {
                duration,
                ..
            }
            | Effect::Berserk {
                duration,
                ..
            }
            | Effect::Blind {
                duration,
                ..
            }
            | Effect::Burn {
                duration,
                ..
            }
            | Effect::Clearcasting {
                duration,
                ..
            }
            | Effect::Cleave {
                duration,
                ..
            }
            | Effect::Empower {
                duration,
                ..
            }
            | Effect::Focus {
                duration,
                ..
            }
            | Effect::Fortify {
                duration,
                ..
            }
            | Effect::Freeze {
                duration,
                ..
            }
            | Effect::Haste {
                duration,
                ..
            }
            | Effect::Immobilize {
                duration,
            }
            | Effect::Lifesteal {
                duration,
                ..
            }
            | Effect::ManaFlow {
                duration,
                ..
            }
            | Effect::MonarchShield {
                duration,
            }
            | Effect::Paranoia {
                duration,
                ..
            }
            | Effect::Poison {
                duration,
                ..
            }
            | Effect::Regen {
                duration,
                ..
            }
            | Effect::Silence {
                duration,
            }
            | Effect::SoulLink {
                duration,
                ..
            }
            | Effect::StatBoost {
                duration,
                ..
            }
            | Effect::Stun {
                duration,
            }
            | Effect::Taunt {
                duration,
            }
            | Effect::Thorns {
                duration,
                ..
            }
            | Effect::Vulnerability {
                duration,
                ..
            } => *duration,
            Effect::Bleed {
                ..
            } => 12.0,
            Effect::Curse {
                timer,
                ..
            } => *timer as f32,
            _ => 0.0,
        }
    }

    /// Performs the targets self operation.
    fn targets_self(effect: &Effect) -> bool {
        matches!(
            effect,
            Effect::BeastFrenzy { .. }
                | Effect::Berserk { .. }
                | Effect::Bleed { .. }
                | Effect::Clearcasting { .. }
                | Effect::EchoStruck { .. }
                | Effect::Empower { .. }
                | Effect::Focus { .. }
                | Effect::Fortify { .. }
                | Effect::Haste { .. }
                | Effect::Heal { .. }
                | Effect::InstantMana { .. }
                | Effect::Lifesteal { .. }
                | Effect::ManaFlow { .. }
                | Effect::MonarchShield { .. }
                | Effect::Purge
                | Effect::Regen { .. }
                | Effect::SoulLink { .. }
                | Effect::StatBoost { .. }
                | Effect::Taunt { .. }
                | Effect::Thorns { .. }
        )
    }

    /// Asserts that a catalog does not contain duplicate display names.
    fn assert_unique<'a>(catalog: &str, names: impl IntoIterator<Item = &'a str>) {
        let mut seen = HashSet::new();
        for name in names {
            assert!(seen.insert(name), "duplicate {catalog} name: {name}");
        }
    }

    /// Asserts that a catalog image resolves inside the generated asset tree.
    fn assert_image_exists(image: &str) {
        assert!(Path::new("assets").join(image).is_file(), "catalog image is missing: {image}");
    }

    /// Returns the arithmetic mean of a non-empty integer sequence.
    fn average(values: impl Iterator<Item = u32>) -> f64 {
        let values = values.collect::<Vec<_>>();
        assert!(!values.is_empty(), "cannot average an empty balance band");
        values.iter().map(|value| *value as f64).sum::<f64>() / values.len() as f64
    }

    /// Asserts that endgame items cost substantially more than starter items.
    fn assert_price_scales(catalog: &str, entries: impl Iterator<Item = (u32, u32)>) {
        let entries = entries.collect::<Vec<_>>();
        let starter =
            average(entries.iter().filter(|(level, _)| *level <= 5).map(|(_, price)| *price));
        let endgame =
            average(entries.iter().filter(|(level, _)| *level >= 16).map(|(_, price)| *price));
        assert!(
            endgame > starter * 2.0,
            "{catalog} prices do not scale enough: {starter:.1} -> {endgame:.1}"
        );
    }

    #[test]
    /// Verifies that load all catalogs.
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

    #[test]
    /// Verifies every class and Ajah can receive a compatible starter ability.
    fn creation_ability_options_cover_every_class_and_ajah() {
        for class in Class::iter() {
            assert!(
                all_abilities().iter().any(|ability| {
                    ability.level == 1 && class.accepts_starting_ability(ability.kind)
                }),
                "{class:?} has no compatible level-one starting ability"
            );
        }

        for ajah in Ajah::iter() {
            assert!(
                all_abilities()
                    .iter()
                    .any(|ability| ability.level < 3 && ability.kind == ajah.kind()),
                "{ajah:?} has no compatible introductory ability"
            );
        }
    }

    #[test]
    /// Performs the catalog entries have unique names valid levels and images operation.
    fn catalog_entries_have_unique_names_valid_levels_and_images() {
        assert_unique("ability", all_abilities().iter().map(|item| item.name.as_str()));
        assert_unique("perk", all_perks().iter().map(|item| item.name.as_str()));
        assert_unique("monster", all_monsters().iter().map(|item| item.name.as_str()));
        assert_unique("equipment", all_equipment().iter().map(Equipment::name));

        for item in all_equipment() {
            assert!((1..=20).contains(&item.level()), "{} has invalid level", item.name());
            assert!(item.price() > 0, "{} has a zero price", item.name());
            match item {
                Equipment::Weapon(item) => assert_image_exists(&item.image),
                Equipment::Wearable(item) => assert_image_exists(&item.image),
                Equipment::Consumable(item) => assert_image_exists(&item.image),
                Equipment::Artifact(item) => assert_image_exists(&item.image),
            }
        }
        for ability in all_abilities() {
            assert!((1..=20).contains(&ability.level), "{} has invalid level", ability.name);
            assert_image_exists(&ability.image);
        }
        for perk in all_perks() {
            assert!((1..=20).contains(&perk.level), "{} has invalid level", perk.name);
            assert_image_exists(&perk.image);
        }
        for monster in all_monsters() {
            assert!((1..=20).contains(&monster.level), "{} has invalid level", monster.name);
            assert_eq!(
                monster.health, monster.max_health,
                "{} does not start at full health",
                monster.name
            );
            assert!(
                monster.max_health > 0 && monster.attack > 0,
                "{} has invalid combat stats",
                monster.name
            );
            assert!(monster.attack_speed > 0.0, "{} cannot attack", monster.name);
            assert_image_exists(&monster.image);
        }
    }

    #[test]
    /// Verifies that the added creatures keep their curated progression and combat roles.
    fn added_creatures_have_curated_levels_and_combat_roles() {
        let grave_warden = get_monster("Grave Warden").expect("Grave Warden is catalogued");
        assert_eq!(grave_warden.level, 9);
        assert!(grave_warden.is_from_image_dir("creatures"));
        assert!(grave_warden.effects.iter().any(|effect| matches!(effect, Effect::Curse { .. })));

        let void_reaver = get_monster("Void Reaver").expect("Void Reaver is catalogued");
        assert_eq!(void_reaver.level, 16);
        assert!(void_reaver.is_from_image_dir("creatures"));
        assert!(void_reaver
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::Vulnerability { .. })));
        assert!(void_reaver.attack > void_reaver.defense);
        assert!(void_reaver.initiative > grave_warden.initiative);
    }

    #[test]
    /// Verifies that the expanded creature and pet roster keeps its curated progression.
    fn expanded_monster_roster_has_curated_levels_and_families() {
        let creatures = [
            ("Bog Imp", 3),
            ("Mire Hag", 5),
            ("Bone Colossus", 8),
            ("Storm Harpy", 9),
            ("Crimson Minotaur", 11),
            ("Frostbound Wraith", 14),
            ("Wererat", 8),
            ("Werebear", 14),
            ("Werewolf", 17),
            ("Vampire", 19),
            ("Abyssal Behemoth", 20),
        ];
        let pets = [
            ("Fox", 1),
            ("Raven", 1),
            ("Badger", 2),
            ("Boar", 3),
            ("Lynx", 3),
            ("Shadow Panther", 5),
            ("Frost Stag", 6),
            ("Ember Drake", 8),
        ];

        for (name, level) in creatures {
            let monster = get_monster(name).unwrap_or_else(|| panic!("{name} is catalogued"));
            assert_eq!(monster.level, level);
            assert!(monster.is_from_image_dir("creatures"));
        }
        for (name, level) in pets {
            let monster = get_monster(name).unwrap_or_else(|| panic!("{name} is catalogued"));
            assert_eq!(monster.level, level);
            assert!(monster.is_from_image_dir("pets"));
        }
    }

    #[test]
    /// Verifies generated ability targeting, durations, and school distribution.
    fn generated_abilities_respect_targeting_and_cooldown_rules() {
        let physical_share =
            all_abilities().iter().filter(|ability| ability.kind == Kind::Physical).count() as f64
                / all_abilities().len() as f64;
        assert!(
            (0.35..=0.45).contains(&physical_share),
            "physical abilities should be roughly 40% of the catalog, got {:.1}%",
            physical_share * 100.0
        );

        for ability in all_abilities() {
            assert!(!ability.effects.is_empty(), "{} has no effect", ability.name);
            for effect in &ability.effects {
                assert_eq!(
                    targets_self(effect),
                    ability.on_self,
                    "{} mixes self and enemy targeting",
                    ability.name
                );
                assert!(
                    ability.cooldown > effect_duration(effect),
                    "{} has cooldown {} but effect duration {}",
                    ability.name,
                    ability.cooldown,
                    effect_duration(effect)
                );
            }
        }
    }

    #[test]
    /// Verifies semantic catalog generation exposes the intended range of combat effects and perks.
    fn generated_ability_and_perk_effects_cover_distinct_gameplay_roles() {
        let ability_effects: HashSet<String> = all_abilities()
            .iter()
            .flat_map(|ability| ability.effects.iter())
            .map(ToString::to_string)
            .collect();
        for effect in [
            "Berserk",
            "Bleed",
            "Blind",
            "Burn",
            "Clearcasting",
            "Cleave",
            "Curse",
            "Focus",
            "Fortify",
            "Freeze",
            "Haste",
            "Heal",
            "Immobilize",
            "InstantMana",
            "Lifesteal",
            "ManaBurn",
            "ManaFlow",
            "Manasteal",
            "Paranoia",
            "Pierce",
            "Poison",
            "Purge",
            "Regen",
            "Silence",
            "StatBoost",
            "Stun",
            "Taunt",
            "Thorns",
            "Vulnerability",
        ] {
            assert!(
                ability_effects.contains(effect),
                "{effect} is absent from generated abilities"
            );
        }

        let perk_modifiers = all_perks()
            .iter()
            .flat_map(|perk| perk.modifiers.iter())
            .map(|modifier| format!("{modifier:?}"))
            .collect::<Vec<_>>();
        for modifier in [
            "AttackModifier",
            "AttackSpeedModifier",
            "AttributeModifier",
            "CategoryPowerMultiplier",
            "CritChanceModifier",
            "DefenseModifier",
            "HealthRegen",
            "HealingMultiplier",
            "KindPowerMultiplier",
            "KindResistanceMultiplier",
            "ManaRegen",
            "MaxHealthModifier",
            "MaxManaModifier",
            "PetAttackModifier",
            "PetDefenseModifier",
            "PetInitiativeModifier",
        ] {
            assert!(
                perk_modifiers.iter().any(|generated| generated.starts_with(modifier)),
                "{modifier} is absent from generated perks"
            );
        }
    }

    #[test]
    /// Performs the generated items follow semantic balance rules operation.
    fn generated_items_follow_semantic_balance_rules() {
        let apple = get_artifact("Apple").expect("the apple artifact must exist");
        assert_eq!(apple.level, 1);
        assert!(apple.price <= 5, "a mundane apple should remain inexpensive");
        assert!(
            get_artifact("Dye Pigments").is_some(),
            "tailoring dyes should be a crafting artifact"
        );
        assert!(
            all_consumables().iter().all(|item| item.name != "Dye Pigments"),
            "tailoring dyes must not be drinkable"
        );

        for weapon in all_weapons() {
            if matches!(weapon.category, Category::Shield | Category::Book) {
                assert_eq!(weapon.attack, 0, "{} should not deal basic attack damage", weapon.name);
                assert_eq!(weapon.attack_speed, 0.0, "{} should not auto-attack", weapon.name);
                assert_eq!(weapon.crit_chance, 0.0, "{} should not critically strike", weapon.name);
            } else {
                assert!(
                    weapon.attack > 0 && weapon.attack_speed > 0.0,
                    "{} cannot attack",
                    weapon.name
                );
            }
            if weapon.level >= 8 && weapon.kind != Kind::Physical {
                assert!(!weapon.effects.is_empty(), "{} lacks its elemental effect", weapon.name);
            }
        }

        for wearable in all_wearables() {
            if wearable.kind == Kind::Nature {
                assert!(
                    !wearable.effects.iter().any(|effect| matches!(effect, Effect::Freeze { .. })),
                    "{} applies an unrelated frost effect",
                    wearable.name
                );
            }
        }

        for consumable in all_consumables() {
            if consumable.name.to_lowercase().contains("venom") {
                assert!(
                    consumable
                        .effects
                        .iter()
                        .any(|effect| matches!(effect, Effect::Empower { .. })),
                    "{} should act as a weapon coating",
                    consumable.name
                );
            }
        }
    }

    #[test]
    /// Verifies that item prices and monster strength scale across level bands.
    fn generated_catalogs_scale_with_level() {
        assert_price_scales("weapon", all_weapons().iter().map(|item| (item.level, item.price)));
        assert_price_scales(
            "wearable",
            all_wearables().iter().map(|item| (item.level, item.price)),
        );
        assert_price_scales(
            "consumable",
            all_consumables().iter().map(|item| (item.level, item.price)),
        );
        assert_price_scales(
            "artifact",
            all_artifacts().iter().map(|item| (item.level, item.price)),
        );

        let starter_power =
            average(all_monsters().iter().filter(|monster| monster.level <= 5).map(|monster| {
                monster.max_health + monster.attack * 3 + monster.defense * 2 + monster.initiative
            }));
        let endgame_power =
            average(all_monsters().iter().filter(|monster| monster.level >= 16).map(|monster| {
                monster.max_health + monster.attack * 3 + monster.defense * 2 + monster.initiative
            }));
        assert!(
            endgame_power > starter_power * 2.0,
            "monster power does not scale enough: {starter_power:.1} -> {endgame_power:.1}"
        );
    }
}
