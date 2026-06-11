use crate::core::inventory::abilities::AbilityKind;
use crate::core::inventory::equipment::Debuff;
use crate::core::inventory::weapons::WeaponKind;
use crate::core::player::Attribute;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Modifier {
    /// Attributes
    AttributeModifier(Attribute, i32),

    /// Combat stats
    AttackModifier(i32),
    DefenseModifier(i32),
    InitiativeModifier(i32),
    AttackSpeedModifier(f32),
    CritChanceModifier(f32),

    /// Pets
    PetAttackModifier(i32),
    PetDefenseModifier(i32),
    PetInitiativeModifier(i32),
    PetAttackSpeedModifier(i32),

    /// Health & Mana
    MaxHealthModifier(i32),
    MaxManaModifier(i32),

    /// Ability and weapon effects
    AbilityKindMultiplier(AbilityKind, f32),
    WeaponKindMultiplier(WeaponKind, f32),

    /// Damage reduction against debuffs
    DebuffReductionMultiplier(Debuff, f32),

    /// Others
    HealingReceivedMultiplier(f32),
}

impl Modifier {
    pub fn to_short_string(&self) -> String {
        "+2 px".to_string()
    }
}
