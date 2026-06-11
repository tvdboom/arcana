use crate::core::inventory::equipment::Kind;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum Modifier {
    /// Attributes
    BonusStrength(i32),
    BonusDexterity(i32),
    BonusConstitution(i32),
    BonusWisdom(i32),
    BonusCharisma(i32),
    BonusIntelligence(i32),

    /// Combat stats
    BonusAttack(i32),
    AttackMultiplier(f32),
    BonusDefense(i32),
    DefenseMultiplier(f32),
    BonusInitiative(i32),
    InitiativeMultiplier(f32),
    BonusCritChance(f32),
    CritChanceMultiplier(f32),
    BonusAttackSpeed(f32),
    AttackSpeedMultiplier(f32),

    /// Pets
    BonusPetDamage(i32),
    PetDamageMultiplier(f32),
    BonusDefenseDamage(i32),
    DefenseDamageMultiplier(f32),
    BonusPetInitiative(i32),
    PetInitiativeMultiplier(f32),

    /// Others
    HealingReceivedMultiplier(f32),

    /// Health & Mana
    BonusMaxHealth(i32),
    MaxHealthMultiplier(f32),
    BonusMaxMana(i32),
    MaxManaMultiplier(f32),

    /// Ability damage & healing improvements
    AbilityDamageMultiplier(Kind, f32),
    AbilityHealingMultiplier(Kind, f32),

    /// Damage reduction from effects and pets
    PoisonDamageTakenMultiplier(f32),
    BurnDamageTakenMultiplier(f32),
    BleedDamageTakenMultiplier(f32),
    EnemyPetDamageTakenMultiplier(f32),
}

impl Modifier {
    pub fn to_short_string(&self) -> String {
        match self {
            Modifier::BonusStrength(val) => format!("{:+} Str", val),
            Modifier::BonusDexterity(val) => format!("{:+} Dex", val),
            Modifier::BonusConstitution(val) => format!("{:+} Con", val),
            Modifier::BonusWisdom(val) => format!("{:+} Wis", val),
            Modifier::BonusCharisma(val) => format!("{:+} Cha", val),
            Modifier::BonusIntelligence(val) => format!("{:+} Int", val),
            Modifier::BonusAttack(val) => format!("{:+} Atk", val),
            Modifier::AttackMultiplier(val) => format!("{:+}% Atk", (val * 100.0) as i32),
            Modifier::BonusDefense(val) => format!("{:+} Def", val),
            Modifier::DefenseMultiplier(val) => format!("{:+}% Def", (val * 100.0) as i32),
            Modifier::BonusInitiative(val) => format!("{:+} Init", val),
            Modifier::InitiativeMultiplier(val) => format!("{:+}% Init", (val * 100.0) as i32),
            Modifier::BonusCritChance(val) => format!("{:+}% Crit", (val * 100.0) as i32),
            Modifier::CritChanceMultiplier(val) => format!("{:+}% Crit Mult", (val * 100.0) as i32),
            Modifier::BonusAttackSpeed(val) => format!("{:+}% AS", (val * 100.0) as i32),
            Modifier::AttackSpeedMultiplier(val) => format!("{:+}% AS Mult", (val * 100.0) as i32),
            Modifier::BonusPetDamage(val) => format!("{:+} Pet Dmg", val),
            Modifier::PetDamageMultiplier(val) => format!("{:+}% Pet Dmg", (val * 100.0) as i32),
            Modifier::BonusDefenseDamage(val) => format!("{:+} Def Dmg", val),
            Modifier::DefenseDamageMultiplier(val) => format!("{:+}% Def Dmg", (val * 100.0) as i32),
            Modifier::BonusPetInitiative(val) => format!("{:+} Pet Init", val),
            Modifier::PetInitiativeMultiplier(val) => format!("{:+}% Pet Init", (val * 100.0) as i32),
            Modifier::HealingReceivedMultiplier(val) => format!("{:+}% Heal Recv", (val * 100.0) as i32),

            Modifier::BonusMaxHealth(val) => format!("{:+} Max HP", val),
            Modifier::MaxHealthMultiplier(val) => format!("{:+.0}% Max HP", (val - 1.0) * 100.0),
            Modifier::BonusMaxMana(val) => format!("{:+} Max MP", val),
            Modifier::MaxManaMultiplier(val) => format!("{:+.0}% Max MP", (val - 1.0) * 100.0),
            Modifier::AbilityDamageMultiplier(kind, val) => format!("{:+.0}% {} Ability Dmg", (val - 1.0) * 100.0, kind),
            Modifier::AbilityHealingMultiplier(kind, val) => format!("{:+.0}% {} Ability Heal", (val - 1.0) * 100.0, kind),
            Modifier::PoisonDamageTakenMultiplier(val) => format!("{:+.0}% Poison Dmg Taken", (val - 1.0) * 100.0),
            Modifier::BurnDamageTakenMultiplier(val) => format!("{:+.0}% Burn Dmg Taken", (val - 1.0) * 100.0),
            Modifier::BleedDamageTakenMultiplier(val) => format!("{:+.0}% Bleed Dmg Taken", (val - 1.0) * 100.0),
            Modifier::EnemyPetDamageTakenMultiplier(val) => format!("{:+.0}% Enemy Pet Dmg Taken", (val - 1.0) * 100.0),
        }
    }
}
