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
}
