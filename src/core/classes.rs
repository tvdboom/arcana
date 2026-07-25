//! Playable class and Ajah definitions together with their gameplay properties.

use crate::core::catalog::equipment::Kind;
use crate::core::catalog::weapons::Category;
use crate::core::identity::IdentityBonuses;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    Assassin,
    Druid,
    Mage(Ajah),
    #[default]
    Warrior,
    Monk,
    Bard,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ajah {
    #[default]
    Black,
    Red,
    Green,
    White,
}

impl Ajah {
    /// Returns the magical damage kind favored by this Ajah.
    pub fn kind(&self) -> Kind {
        match self {
            Ajah::Black => Kind::Shadow,
            Ajah::Green => Kind::Nature,
            Ajah::Red => Kind::Fire,
            Ajah::White => Kind::Ice,
        }
    }

    /// Returns the numerical combat package granted by this Ajah.
    pub fn bonuses(self) -> IdentityBonuses {
        ClassSpecialization::Mage(self).bonuses()
    }
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssassinPath {
    #[default]
    Nightblade,
    Venomhand,
    Duelist,
    Phantom,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarriorPath {
    #[default]
    Paladin,
    Templar,
    Berserker,
    Warden,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonkSchool {
    #[default]
    OpenHand,
    IronBody,
    ShadowStep,
    SpiritFist,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BardStyle {
    #[default]
    WarChant,
    SilverBallad,
    GraveDirge,
    WildRhythm,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetChoice {
    Owl,
    #[default]
    Rat,
    Snake,
    Weasel,
    Fox,
    Raven,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClassSpecialization {
    Assassin(AssassinPath),
    Druid(PetChoice),
    Mage(Ajah),
    Warrior(WarriorPath),
    Monk(MonkSchool),
    Bard(BardStyle),
}

impl Default for ClassSpecialization {
    /// Returns the specialization paired with the default Warrior class.
    fn default() -> Self {
        Self::Warrior(WarriorPath::default())
    }
}

impl Class {
    /// Returns the initial specialization highlighted for this class.
    pub fn default_specialization(self) -> ClassSpecialization {
        match self {
            Class::Assassin => ClassSpecialization::Assassin(AssassinPath::default()),
            Class::Druid => ClassSpecialization::Druid(PetChoice::default()),
            Class::Mage(ajah) => ClassSpecialization::Mage(ajah),
            Class::Warrior => ClassSpecialization::Warrior(WarriorPath::default()),
            Class::Monk => ClassSpecialization::Monk(MonkSchool::default()),
            Class::Bard => ClassSpecialization::Bard(BardStyle::default()),
        }
    }

    /// Returns whether this class primarily learns magical abilities.
    pub fn is_magical(self) -> bool {
        matches!(self, Class::Druid | Class::Mage(_) | Class::Bard)
    }

    /// Returns whether an ability kind belongs in this class's starting kit.
    pub fn accepts_starting_ability(self, kind: Kind) -> bool {
        match self {
            Class::Druid => kind == Kind::Nature,
            Class::Mage(_) | Class::Bard => kind.is_magic(),
            Class::Assassin | Class::Warrior | Class::Monk => kind == Kind::Physical,
        }
    }

    /// Returns whether a weapon category belongs in this class's starting kit.
    pub fn accepts_starting_weapon(self, category: Category) -> bool {
        match self {
            Class::Assassin | Class::Monk => category == Category::Finesse,
            Class::Druid | Class::Mage(_) | Class::Bard => category == Category::Magical,
            Class::Warrior => category == Category::Melee,
        }
    }

    /// Returns this class's permanent derived-stat bonuses.
    pub fn bonuses(self) -> IdentityBonuses {
        match self {
            Class::Assassin => IdentityBonuses {
                initiative: 2,
                ..Default::default()
            },
            Class::Druid => IdentityBonuses {
                max_mana: 10,
                ..Default::default()
            },
            Class::Mage(_) => IdentityBonuses {
                max_mana: 30,
                ..Default::default()
            },
            Class::Warrior => IdentityBonuses {
                max_health: 20,
                ..Default::default()
            },
            Class::Monk => IdentityBonuses {
                attack_speed: 0.10,
                ..Default::default()
            },
            Class::Bard => IdentityBonuses {
                max_mana: 20,
                ..Default::default()
            },
        }
    }
}

impl ClassSpecialization {
    /// Returns this specialization's permanent derived-stat bonuses.
    pub fn bonuses(self) -> IdentityBonuses {
        match self {
            ClassSpecialization::Assassin(path) => match path {
                AssassinPath::Nightblade => IdentityBonuses {
                    initiative: 3,
                    ..Default::default()
                },
                AssassinPath::Venomhand => IdentityBonuses {
                    ranged_attack: 1,
                    ..Default::default()
                },
                AssassinPath::Duelist => IdentityBonuses {
                    finesse_attack: 1,
                    ..Default::default()
                },
                AssassinPath::Phantom => IdentityBonuses {
                    crit_chance: 0.12,
                    ..Default::default()
                },
            },
            ClassSpecialization::Druid(_) => IdentityBonuses::default(),
            ClassSpecialization::Mage(ajah) => match ajah {
                Ajah::Black => IdentityBonuses {
                    attack: 1,
                    ..Default::default()
                },
                Ajah::Red => IdentityBonuses {
                    crit_chance: 0.12,
                    ..Default::default()
                },
                Ajah::Green => IdentityBonuses {
                    health_regen: 1,
                    ..Default::default()
                },
                Ajah::White => IdentityBonuses {
                    defense: 3,
                    ..Default::default()
                },
            },
            ClassSpecialization::Warrior(path) => match path {
                WarriorPath::Paladin => IdentityBonuses {
                    health_regen: 1,
                    ..Default::default()
                },
                WarriorPath::Templar => IdentityBonuses {
                    defense: 3,
                    ..Default::default()
                },
                WarriorPath::Berserker => IdentityBonuses {
                    attack: 1,
                    defense: -1,
                    ..Default::default()
                },
                WarriorPath::Warden => IdentityBonuses {
                    max_health: 5,
                    health_regen: 1,
                    ..Default::default()
                },
            },
            ClassSpecialization::Monk(school) => match school {
                MonkSchool::OpenHand => IdentityBonuses {
                    attack: 1,
                    max_mana: 5,
                    ..Default::default()
                },
                MonkSchool::IronBody => IdentityBonuses {
                    defense: 1,
                    health_regen: 1,
                    ..Default::default()
                },
                MonkSchool::ShadowStep => IdentityBonuses {
                    initiative: 2,
                    attack_speed: 0.05,
                    ..Default::default()
                },
                MonkSchool::SpiritFist => IdentityBonuses {
                    attack: 1,
                    mana_regen: 1,
                    ..Default::default()
                },
            },
            ClassSpecialization::Bard(style) => match style {
                BardStyle::WarChant => IdentityBonuses {
                    attack: 1,
                    max_mana: 5,
                    ..Default::default()
                },
                BardStyle::SilverBallad => IdentityBonuses {
                    max_health: 10,
                    health_regen: 1,
                    ..Default::default()
                },
                BardStyle::GraveDirge => IdentityBonuses {
                    attack: 1,
                    mana_regen: 1,
                    ..Default::default()
                },
                BardStyle::WildRhythm => IdentityBonuses {
                    initiative: 2,
                    attack_speed: 0.05,
                    ..Default::default()
                },
            },
        }
    }
}

impl PetChoice {
    /// Returns the stable monster-catalog name for this companion choice.
    pub fn monster_name(self) -> &'static str {
        match self {
            PetChoice::Owl => "Owl",
            PetChoice::Rat => "Rat",
            PetChoice::Snake => "Snake",
            PetChoice::Weasel => "Weasel",
            PetChoice::Fox => "Fox",
            PetChoice::Raven => "Raven",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::catalog::catalog::get_monster;
    use crate::core::catalog::effects::Effect;
    use strum::IntoEnumIterator;

    /// Ensures no specialization within a class has a materially larger numerical budget.
    fn assert_balanced_family(family: &[ClassSpecialization]) {
        let ratings = family
            .iter()
            .map(|specialization| specialization.bonuses().representative_combat_rating())
            .collect::<Vec<_>>();
        let minimum = ratings.iter().copied().fold(f32::INFINITY, f32::min);
        let maximum = ratings.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(maximum / minimum <= 1.06, "specialization ratings diverged: {ratings:?}");
    }

    #[test]
    /// Verifies each class's specialization options stay within a narrow combat-value band.
    fn specialization_families_have_comparable_combat_value() {
        assert_balanced_family(
            &AssassinPath::iter().map(ClassSpecialization::Assassin).collect::<Vec<_>>(),
        );
        assert_balanced_family(
            &WarriorPath::iter().map(ClassSpecialization::Warrior).collect::<Vec<_>>(),
        );
        assert_balanced_family(
            &MonkSchool::iter().map(ClassSpecialization::Monk).collect::<Vec<_>>(),
        );
        assert_balanced_family(
            &BardStyle::iter().map(ClassSpecialization::Bard).collect::<Vec<_>>(),
        );
        assert_balanced_family(&Ajah::iter().map(ClassSpecialization::Mage).collect::<Vec<_>>());
    }

    #[test]
    /// Verifies base class stat packages stay within a narrow representative combat-value band.
    fn base_class_bonuses_have_comparable_combat_value() {
        let ratings = Class::iter()
            .map(|class| class.bonuses().representative_combat_rating())
            .collect::<Vec<_>>();
        let minimum = ratings.iter().copied().fold(f32::INFINITY, f32::min);
        let maximum = ratings.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        assert!(maximum / minimum <= 1.10, "class ratings diverged: {ratings:?}");
    }

    #[test]
    /// Verifies every creation-time pet maps to an existing level-one companion.
    fn creation_pet_choices_resolve_to_comparable_catalog_entries() {
        let pets = PetChoice::iter()
            .map(|choice| {
                get_monster(choice.monster_name())
                    .unwrap_or_else(|| panic!("missing creation pet {}", choice.monster_name()))
            })
            .collect::<Vec<_>>();

        assert!(pets.iter().all(|pet| pet.level == 1));
        assert!(pets.iter().all(|pet| pet.attack == 6 && pet.defense == 5));
        assert!(pets.iter().all(|pet| (1.0..=1.12).contains(&pet.attack_speed)));
        assert!(
            pets.iter().map(|pet| pet.initiative).max().unwrap()
                - pets.iter().map(|pet| pet.initiative).min().unwrap()
                <= 1
        );
        assert!(pets.iter().all(|pet| pet.health_regen == 1));
        assert!(
            pets.iter().map(|pet| pet.max_health).max().unwrap()
                - pets.iter().map(|pet| pet.max_health).min().unwrap()
                <= 4
        );
        for pet in pets {
            assert_eq!(pet.effects.len(), 1);
            match &pet.effects[0] {
                Effect::Blind {
                    miss_pct,
                    duration,
                } => {
                    assert!(*miss_pct <= 9.0);
                    assert!(*duration <= 2.0);
                },
                Effect::Cleave {
                    damage_pct,
                    duration,
                } => {
                    assert!(*damage_pct <= 60.0);
                    assert_eq!(*duration, 0.0);
                },
                Effect::Poison {
                    damage,
                    duration,
                } => {
                    assert!(*damage <= 1);
                    assert!(*duration <= 0.5);
                },
                effect => panic!("unbalanced creation-pet effect: {effect:?}"),
            }
        }
    }

    #[test]
    /// Verifies class starting-kit rules match every advertised ability and weapon type.
    fn class_starting_kit_rules_match_descriptions() {
        for class in Class::iter() {
            let expected_ability_kind = match class {
                Class::Druid => Kind::Nature,
                Class::Mage(_) | Class::Bard => Kind::Fire,
                Class::Assassin | Class::Warrior | Class::Monk => Kind::Physical,
            };
            let expected_weapon_category = match class {
                Class::Assassin | Class::Monk => Category::Finesse,
                Class::Druid | Class::Mage(_) | Class::Bard => Category::Magical,
                Class::Warrior => Category::Melee,
            };

            assert!(class.accepts_starting_ability(expected_ability_kind));
            assert!(class.accepts_starting_weapon(expected_weapon_category));
        }
        assert!(!Class::Druid.accepts_starting_ability(Kind::Fire));
        assert!(!Class::Warrior.accepts_starting_weapon(Category::Magical));
    }
}
