//! Pure combat rules and Bevy systems for attacks, abilities, effects, and victory.

use bevy::prelude::*;
use rand::{rng, RngExt};
use std::collections::HashMap;

use crate::core::actions::hunt::{hunt_pet_chance, PendingHuntPet, PendingHuntXp};
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::{get_ability, get_equipment};
use crate::core::catalog::effects::Effect;
use crate::core::catalog::equipment::Equipment;
use crate::core::catalog::equipment::Kind;
use crate::core::catalog::modifiers::Modifier;
use crate::core::catalog::weapons::{Category, Weapon};
use crate::core::combat::tactics::{
    ability_poise_damage, enemy_move_rotation, CombatStance, EnemyMove, EnemyMoveKind,
    EnemyMoveTarget,
};
use crate::core::combat::ui::{
    CombatCmp, CombatContinueWithPetButton, CombatContinueWithPetSlot, CombatEffectIcon,
    CombatPetName, CombatPortraitLevel, CombatPortraitName, CombatStatLabel,
};
use crate::core::menu::systems::{CombatMenuSuspended, GameMenuOrigin};
use crate::core::monsters::{ActiveMonster, Monster, MonsterArchetype};
use crate::core::player::{Attribute, Player};
use crate::core::races::Mutation;
use crate::core::states::GameState;
use crate::core::ui::playing::TooltipNode;
use crate::core::ui::utils::despawn_descendants_manual;

/// Hotkeys used to trigger the 5 active abilities (must match combat::ui).
pub const ABILITY_HOTKEYS: [KeyCode; 5] =
    [KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR, KeyCode::KeyT];

/// Hotkeys used to trigger equipped consumables (must match combat::ui).
pub const CONSUMABLE_HOTKEYS: [KeyCode; 8] = [
    KeyCode::KeyA,
    KeyCode::KeyS,
    KeyCode::KeyD,
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyJ,
    KeyCode::KeyK,
];

pub const GUARD_HOTKEY: KeyCode = KeyCode::KeyZ;
pub const STANCE_HOTKEYS: [KeyCode; 4] =
    [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4];

const BAR_LERP_SPEED: f32 = 6.0;
const ATTACK_PERIOD_MULTIPLIER: f32 = 2.0;
const ABILITY_MANA_COST_MULTIPLIER: u32 = 2;
const HIT_TEXT_SIZE: f32 = 4.6;
const XP_REWARD_TEXT_SIZE: f32 = 9.0;
const XP_REWARD_TEXT_LIFE: f32 = 3.2;
const DEATH_SKULL_ANIM_DURATION: f32 = 0.9;
const DEATH_SKULL_START_SIZE: f32 = 6.0;
const DEATH_SKULL_END_SIZE: f32 = 50.0;
/// Action points added as a penalty when a (non-duel) combat is lost.
const LOST_COMBAT_AP_PENALTY: u32 = 5;
/// Bounds and step for the adjustable combat speed.
const COMBAT_SPEED_MIN: f32 = 0.25;
const COMBAT_SPEED_MAX: f32 = 8.0;
const DODGE_BASE_CHANCE: f32 = 0.18;
const DODGE_POINT_MULTIPLIER: f32 = 0.018;
const DODGE_CHANCE_MIN: f32 = 0.08;
const DODGE_CHANCE_MAX: f32 = 0.70;
/// Maximum number of copies of one consumable available in a single combat.
pub const MAX_COMBAT_CONSUMABLES_PER_TYPE: usize = 5;
const GUARD_DURATION: f32 = 0.90;
pub const GUARD_MANA_COST_PER_LEVEL: f32 = 5.0;
const GUARD_DAMAGE_REDUCTION: f32 = 0.62;
const PERFECT_GUARD_DAMAGE_REDUCTION: f32 = 0.92;
const PERFECT_GUARD_POISE_DAMAGE: f32 = 28.0;
const BREAK_STUN_DURATION: f32 = 1.6;
const BREAK_VULNERABILITY_DURATION: f32 = 3.2;
const BREAK_VULNERABILITY_PERCENT: f32 = 25.0;

/// Adjustable time multiplier for combat, persisted across battles. Controlled
/// with Ctrl+Shift+Left/Right and applied to every time-driven combat system.
#[derive(Resource)]
pub struct CombatSpeed(pub f32);

/// Marker inserted while a *networked* duel combat is running. The standard
/// single-player combat systems check for it and stand aside so the duel
/// systems (in `core::network`) can drive an authoritative, synced fight.
#[derive(Resource)]
pub struct DuelActive;

impl Default for CombatSpeed {
    /// Returns the default value.
    fn default() -> Self {
        Self(1.0)
    }
}

impl CombatSpeed {
    /// Performs the faster operation.
    pub fn faster(&mut self) {
        self.0 = (self.0 * 2.0).min(COMBAT_SPEED_MAX);
    }

    /// Performs the slower operation.
    pub fn slower(&mut self) {
        self.0 = (self.0 / 2.0).max(COMBAT_SPEED_MIN);
    }

    /// Human-readable label such as "1x", "1.5x" or "0.25x".
    pub fn label(&self) -> String {
        let s = format!("{:.2}", self.0);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        format!("{}x", s)
    }
}

/// Marker for the small combat-speed label shown beside the forfeit button.
#[derive(Component)]
pub struct CombatSpeedText;

// ---------------------------------------------------------------------------
// Components placed on combat UI cards (spawned by combat::ui).
// ---------------------------------------------------------------------------

/// Identifies a clickable combat card and what it triggers.
#[derive(Component, Clone)]
pub enum CombatCard {
    Ability(usize),
    Consumable(String),
    Guard,
    Stance(CombatStance),
}

/// Dark overlay child of an ability card. Its height encodes cooldown progress.
#[derive(Component)]
pub struct AbilityCooldownOverlay {
    pub slot: usize,
    pub is_player: bool,
}

/// Root node of a consumable card, tagged with its catalog key for despawn/sync.
#[derive(Component)]
pub struct ConsumableCardRoot {
    pub key: String,
    pub is_player: bool,
}

/// Inventory count text shown on a player's consumable combat card.
#[derive(Component)]
pub struct ConsumableCardCount {
    pub key: String,
}

/// Identifies whether an equipment slot is for player or opponent/enemy.
#[derive(Component)]
pub struct CombatSlot {
    pub is_player: bool,
}

/// Remaining cooldown seconds overlay text child of an ability card.
#[derive(Component)]
pub struct AbilityCooldownText {
    pub slot: usize,
    pub is_player: bool,
}

/// Marker for the bottom combat button (forfeit / continue).
#[derive(Component)]
pub struct CombatEndButton;

/// Marker for the text inside the combat end button.
#[derive(Component)]
pub struct CombatEndButtonText;

/// Floating combat feedback text that drifts upward and fades.
#[derive(Component)]
pub struct FloatingCombatText {
    pub timer: f32,
    pub start_top: f32,
    pub life: f32,
    /// When true the text is centered on the player portrait and barely drifts.
    pub centered: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeathSkullSide {
    Player,
    Enemy,
    Pet,
}

#[derive(Component)]
pub struct DeathSkullOverlay {
    pub side: DeathSkullSide,
    pub timer: f32,
}

// ---------------------------------------------------------------------------
// Combat state
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FxSide {
    Player,
    Enemy,
}

pub struct CombatFx {
    pub side: FxSide,
    pub text: String,
    pub color: Color,
}

#[derive(Clone)]
pub struct TimedEffect {
    pub effect: Effect,
    pub remaining: f32,
    pub tick_acc: f32,
    pub magnitude_multiplier: f32,
}

/// A weapon-bound effect and whether it triggers on landing a hit (offensive
/// weapons) or on being hit (defensive: shield, book).
#[derive(Clone)]
pub struct WeaponEffect {
    pub effect: Effect,
    pub on_hit: bool,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct CombatWeapon {
    pub name: String,
    pub kind: Kind,
    pub category: Category,
    pub attack_speed: f32,
    pub attack_timer: f32,
    pub attack: f32,
    pub crit_chance: f32,
    pub effects: Vec<WeaponEffect>,
    pub attack_style: AttackStyle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttackStyle {
    Melee,
    Finesse,
    Range,
    Other,
}

#[allow(dead_code)]
pub struct Fighter {
    pub max_health: f32,
    pub health: f32,
    pub display_health: f32,
    pub max_mana: f32,
    pub mana: f32,
    pub display_mana: f32,
    pub base_attack: f32,
    pub base_defense: f32,
    pub base_initiative: f32,
    pub base_attack_speed: f32,
    pub crit_chance: f32,
    pub health_regen: f32,
    pub mana_regen: f32,
    pub attack_timer: f32,
    pub effects: Vec<TimedEffect>,
    pub weapon_effects: Vec<WeaponEffect>,
    pub attack_style: AttackStyle,
    pub intelligence_mod: f32,
    pub passive_modifiers: Vec<Modifier>,
    pub mutation: Option<Mutation>,
    pub alive: bool,
    pub weapons: Vec<CombatWeapon>,
}

impl Fighter {
    /// Performs the eff attack speed for operation.
    pub fn eff_attack_speed_for(&self, base_speed: f32) -> f32 {
        let mut v = base_speed;
        for te in &self.effects {
            if let Effect::Freeze {
                attack_speed_pct,
                ..
            } = &te.effect
            {
                v *= (1.0 + attack_speed_pct / 100.0).max(0.1);
            }
            if let Effect::BeastFrenzy {
                attack_speed_pct,
                ..
            } = &te.effect
            {
                v *= 1.0 + attack_speed_pct / 100.0;
            }
        }
        v.max(0.1)
    }

    /// Performs the attack period for operation.
    pub fn attack_period_for(&self, base_speed: f32) -> f32 {
        (ATTACK_PERIOD_MULTIPLIER / self.eff_attack_speed_for(base_speed)).clamp(0.2, 10.0)
    }

    /// Performs the eff attack for operation.
    pub fn eff_attack_for(&self, base_attack: f32) -> f32 {
        let mut v = base_attack;
        for te in &self.effects {
            match &te.effect {
                Effect::Berserk {
                    attack_pct,
                    ..
                } => v *= 1.0 + attack_pct / 100.0,
                Effect::Empower {
                    damage_pct,
                    ..
                } => v *= 1.0 + damage_pct / 100.0,
                Effect::BeastFrenzy {
                    attack_pct,
                    ..
                } => v *= 1.0 + attack_pct / 100.0,
                _ => {},
            }
        }
        v += self.stat_boost_bonus(&[Attribute::Strength, Attribute::Intelligence]);
        v.max(0.0)
    }

    /// Sums the flat combat bonus granted by active `StatBoost` effects whose
    /// attribute governs the requested combat stat.
    fn stat_boost_bonus(&self, attrs: &[Attribute]) -> f32 {
        let mut bonus = 0.0;
        for te in &self.effects {
            if let Effect::StatBoost {
                attribute,
                amount,
                ..
            } = &te.effect
            {
                if attrs.contains(attribute) {
                    bonus += *amount as f32;
                }
            }
        }
        bonus
    }

    /// Fraction by which active `Clearcasting` effects reduce ability mana cost.
    fn clearcasting_reduction(&self) -> f32 {
        let mut r = 0.0;
        for te in &self.effects {
            if let Effect::Clearcasting {
                reduction_pct,
                ..
            } = &te.effect
            {
                r += reduction_pct / 100.0;
            }
        }
        r.clamp(0.0, 0.9)
    }

    /// Whether an active `Taunt` is forcing enemies to strike this fighter's pet.
    fn has_taunt(&self) -> bool {
        self.effects.iter().any(|te| matches!(te.effect, Effect::Taunt { .. }))
    }

    #[allow(dead_code)]
    /// Performs the attack period operation.
    fn attack_period(&self) -> f32 {
        (ATTACK_PERIOD_MULTIPLIER / self.eff_attack_speed()).clamp(0.2, 10.0)
    }

    #[allow(dead_code)]
    /// Performs the eff attack speed operation.
    fn eff_attack_speed(&self) -> f32 {
        let mut v = self.base_attack_speed;
        for te in &self.effects {
            if let Effect::Freeze {
                attack_speed_pct,
                ..
            } = &te.effect
            {
                v *= (1.0 + attack_speed_pct / 100.0).max(0.1);
            }
            if let Effect::BeastFrenzy {
                attack_speed_pct,
                ..
            } = &te.effect
            {
                v *= 1.0 + attack_speed_pct / 100.0;
            }
        }
        v.max(0.1)
    }

    #[allow(dead_code)]
    /// Performs the eff attack operation.
    fn eff_attack(&self) -> f32 {
        let mut v = self.base_attack;
        for te in &self.effects {
            match &te.effect {
                Effect::Berserk {
                    attack_pct,
                    ..
                } => v *= 1.0 + attack_pct / 100.0,
                Effect::Empower {
                    damage_pct,
                    ..
                } => v *= 1.0 + damage_pct / 100.0,
                Effect::BeastFrenzy {
                    attack_pct,
                    ..
                } => v *= 1.0 + attack_pct / 100.0,
                _ => {},
            }
        }
        v.max(0.0)
    }

    /// Performs the eff defense operation.
    fn eff_defense(&self) -> f32 {
        let mut v = self.base_defense;
        for te in &self.effects {
            if let Effect::Fortify {
                defense_pct,
                ..
            } = &te.effect
            {
                v *= 1.0 + defense_pct / 100.0;
            }
        }
        v += self.stat_boost_bonus(&[Attribute::Constitution, Attribute::Wisdom]);
        v.max(0.0)
    }

    /// Performs the eff initiative operation.
    fn eff_initiative(&self) -> f32 {
        let mut v = self.base_initiative;
        for te in &self.effects {
            match &te.effect {
                Effect::Haste {
                    initiative_pct,
                    ..
                } => v *= 1.0 + initiative_pct / 100.0,
                Effect::Paranoia {
                    initiative_pct,
                    ..
                } => v *= (1.0 - initiative_pct / 100.0).max(0.0),
                _ => {},
            }
        }
        v += self.stat_boost_bonus(&[Attribute::Dexterity, Attribute::Charisma]);
        v.max(0.0)
    }

    /// Performs the miss chance operation.
    fn miss_chance(&self) -> f32 {
        let mut m = 0.0;
        for te in &self.effects {
            if let Effect::Blind {
                miss_pct,
                ..
            } = &te.effect
            {
                m += miss_pct / 100.0;
            }
        }
        m.clamp(0.0, 0.9)
    }

    /// Performs the extra crit operation.
    fn extra_crit(&self) -> f32 {
        let mut c = 0.0;
        for te in &self.effects {
            if let Effect::Focus {
                crit_chance_pct,
                ..
            } = &te.effect
            {
                c += crit_chance_pct / 100.0;
            }
        }
        c
    }

    /// Performs the incoming multiplier operation.
    fn incoming_multiplier(&self) -> f32 {
        let mut v = 1.0;
        for te in &self.effects {
            if let Effect::Vulnerability {
                damage_pct,
                ..
            } = &te.effect
            {
                v *= 1.0 + damage_pct / 100.0;
            }
        }
        v
    }

    /// Returns whether dodge.
    fn can_dodge(&self) -> bool {
        !self.effects.iter().any(|te| matches!(te.effect, Effect::Immobilize { .. }))
    }

    /// Returns whether act.
    fn can_act(&self) -> bool {
        !self
            .effects
            .iter()
            .any(|te| matches!(te.effect, Effect::Stun { .. } | Effect::MonarchShield { .. }))
    }

    /// Returns whether cast.
    fn can_cast(&self) -> bool {
        !self.effects.iter().any(|te| {
            matches!(
                te.effect,
                Effect::Silence { .. } | Effect::Stun { .. } | Effect::MonarchShield { .. }
            )
        })
    }

    /// Performs the lifesteal operation.
    fn lifesteal(&self) -> f32 {
        let mut v = self
            .passive_modifiers
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::LifeSteal(percentage) => Some(percentage / 100.0),
                _ => None,
            })
            .sum::<f32>();
        for te in &self.effects {
            if let Effect::Lifesteal {
                percentage,
                ..
            } = &te.effect
            {
                v += percentage / 100.0;
            }
        }
        v.max(0.0)
    }

    /// Multiplies outgoing damage for the supplied element and optional weapon category.
    fn outgoing_damage_multiplier(&self, kind: Kind, category: Option<Category>) -> f32 {
        let mut percentage = self
            .passive_modifiers
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::KindPowerMultiplier(modifier_kind, value) if *modifier_kind == kind => {
                    Some(*value)
                },
                Modifier::CategoryPowerMultiplier(modifier_category, value)
                    if category == Some(*modifier_category) =>
                {
                    Some(*value)
                },
                _ => None,
            })
            .sum::<f32>();
        if self.mutation == Some(Mutation::Vampire) {
            percentage += 15.0;
        }
        (1.0 + percentage / 100.0).max(0.0)
    }

    /// Multiplies incoming damage after elemental and weapon-category resistances.
    fn incoming_damage_multiplier(&self, kind: Kind, category: Option<Category>) -> f32 {
        let mut percentage = self
            .passive_modifiers
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::KindResistanceMultiplier(modifier_kind, value)
                    if *modifier_kind == kind =>
                {
                    Some(*value)
                },
                Modifier::CategoryResistanceMultiplier(modifier_category, value)
                    if category == Some(*modifier_category) =>
                {
                    Some(*value)
                },
                _ => None,
            })
            .sum::<f32>();
        if self.mutation == Some(Mutation::Vampire) && kind == Kind::Fire {
            percentage -= 15.0;
        }
        (1.0 - percentage / 100.0).max(0.0)
    }

    /// Returns whether this fighter's mutation completely negates an effect.
    fn is_immune_to_effect(&self, effect: &Effect) -> bool {
        self.mutation == Some(Mutation::Undead)
            && matches!(effect, Effect::Poison { .. } | Effect::Freeze { .. })
    }

    /// Multiplies direct and periodic healing caused by this fighter.
    fn healing_multiplier(&self) -> f32 {
        let percentage = self
            .passive_modifiers
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::HealingMultiplier(value) => Some(*value),
                _ => None,
            })
            .sum::<f32>();
        (1.0 + percentage / 100.0).max(0.0)
    }

    /// Performs the take damage operation.
    fn take_damage(&mut self, dmg: f32) {
        self.health = (self.health - dmg).max(0.0);
        if self.health <= 0.0 {
            self.alive = false;
        }
    }

    /// Performs the heal operation.
    fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Performs the restore mana operation.
    fn restore_mana(&mut self, amount: f32) {
        self.mana = (self.mana + amount).min(self.max_mana);
    }
}

#[derive(Clone)]
pub struct AbilitySlot {
    pub key: Option<String>,
    pub cooldown: f32,
    pub remaining: f32,
    pub mana_cost: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyCast {
    pub movement: EnemyMove,
    pub elapsed: f32,
}

#[derive(Clone, Debug)]
pub struct EnemyTactics {
    pub archetype: MonsterArchetype,
    pub rotation: Vec<EnemyMove>,
    pub next_index: usize,
    pub recovery: f32,
    pub recovery_max: f32,
    pub active_cast: Option<EnemyCast>,
    pub phase_two: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CombatStatus {
    Ongoing,
    Over,
}

#[derive(Resource)]
pub struct CombatState {
    pub player: Fighter,
    pub pet: Option<Fighter>,
    pub enemy: Fighter,
    pub enemy_pet: Option<Fighter>,
    pub abilities: Vec<AbilitySlot>,
    pub enemy_abilities: Vec<AbilitySlot>,
    pub player_consumables: HashMap<String, usize>,
    pub enemy_consumables: HashMap<String, usize>,
    pub stance: CombatStance,
    pub guard_remaining: f32,
    pub perfect_guard_remaining: f32,
    pub enemy_poise: f32,
    pub enemy_max_poise: f32,
    pub enemy_break_remaining: f32,
    pub enemy_tactics: Option<EnemyTactics>,
    pub status: CombatStatus,
    pub player_won: bool,
    pub player_level: u32,
    pub enemy_level: u32,
    pub mutation_candidate: Option<Mutation>,
    pub fx: Vec<CombatFx>,
    pub paused: bool,
    pub dodge_word: String,
    pub miss_word: String,
    pub xp_word: String,
    pub guard_word: String,
    pub parry_word: String,
    pub break_word: String,
    pub shatter_word: String,
    pub detonate_word: String,
    pub doom_word: String,
    pub exploit_word: String,
    pub cleanse_word: String,
    pub phase_word: String,
}

impl CombatState {
    /// Performs the xp reward operation.
    pub fn xp_reward(&self) -> u32 {
        if !self.player_won {
            return 0;
        }
        let diff = self.enemy_level as i32 - self.player_level as i32;
        (2 + diff).max(0) as u32
    }
}

/// Selects equipped consumables for combat without removing excess stock from the inventory.
fn select_combat_consumables(player: &Player) -> HashMap<String, usize> {
    player
        .equipped_consumables
        .iter()
        .filter(|key| matches!(get_equipment(key), Some(Equipment::Consumable(_))))
        .map(|key| {
            let count = player.inventory.iter().filter(|item| *item == key).count();
            (key.clone(), count.min(MAX_COMBAT_CONSUMABLES_PER_TYPE))
        })
        .filter(|(_, count)| *count > 0)
        .collect()
}

/// Takes one consumable from combat stock, returning whether one was available.
fn take_combat_consumable(stock: &mut HashMap<String, usize>, key: &str) -> bool {
    let Some(remaining) = stock.get_mut(key) else {
        return false;
    };
    if *remaining == 0 {
        return false;
    }
    *remaining -= 1;
    true
}

/// Collects offensive weapon effects and defensive equipment effects for combat.
fn player_equipment_effects(player: &Player) -> Vec<WeaponEffect> {
    let mut out = Vec::new();
    for eq in player.equipped_equipment() {
        let (effects, on_hit) = match eq {
            Equipment::Weapon(weapon) => {
                (weapon.effects, !matches!(weapon.category, Category::Shield | Category::Book))
            },
            Equipment::Wearable(wearable) => (wearable.effects, false),
            Equipment::Consumable(_) | Equipment::Artifact(_) => continue,
        };
        for effect in effects {
            out.push(WeaponEffect {
                effect,
                on_hit,
            });
        }
    }
    out
}

/// Performs the player attack style operation.
fn player_attack_style(player: &Player) -> AttackStyle {
    if player.has_equipped_range() {
        AttackStyle::Range
    } else if player.has_equipped_finesse() {
        AttackStyle::Finesse
    } else if player.has_equipped_melee() {
        AttackStyle::Melee
    } else {
        AttackStyle::Other
    }
}

/// Returns the full flat attack contribution of one equipped weapon.
fn weapon_attack(weapon: &Weapon) -> f32 {
    let modifier = weapon
        .modifiers
        .iter()
        .filter_map(|modifier| match modifier {
            Modifier::AttackModifier(value) => Some(*value),
            _ => None,
        })
        .sum::<i32>();
    (weapon.attack as i32 + modifier).max(0) as f32
}

/// Returns the attack presentation used by a weapon category.
fn category_attack_style(category: Category) -> AttackStyle {
    match category {
        Category::Range => AttackStyle::Range,
        Category::Finesse => AttackStyle::Finesse,
        Category::Melee => AttackStyle::Melee,
        Category::Magical | Category::Shield | Category::Book => AttackStyle::Other,
    }
}

/// Builds basic attacks for a player without letting shields or books auto-attack.
fn player_combat_weapons(player: &Player) -> Vec<CombatWeapon> {
    let equipped = player.equipped_equipment();
    let attacking_weapons = equipped
        .iter()
        .filter_map(|equipment| match equipment {
            Equipment::Weapon(weapon)
                if !matches!(weapon.category, Category::Shield | Category::Book) =>
            {
                Some(weapon)
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    if attacking_weapons.is_empty() {
        return Vec::new();
    }

    let weapon_attack_total =
        attacking_weapons.iter().map(|weapon| weapon_attack(weapon)).sum::<f32>();
    let shared_attack =
        (player.attack() as f32 - weapon_attack_total).max(0.0) / attacking_weapons.len() as f32;
    let speed_multiplier = player.attack_speed_multiplier();
    let non_weapon_crit = player.non_weapon_crit_chance();

    attacking_weapons
        .into_iter()
        .map(|weapon| CombatWeapon {
            name: weapon.name.clone(),
            kind: weapon.kind,
            category: weapon.category,
            attack_speed: weapon.attack_speed * speed_multiplier,
            attack_timer: 0.0,
            attack: shared_attack + weapon_attack(weapon),
            crit_chance: (weapon.crit_chance + non_weapon_crit).clamp(0.0, 1.0),
            effects: weapon
                .effects
                .iter()
                .map(|effect| WeaponEffect {
                    effect: effect.clone(),
                    on_hit: true,
                })
                .collect(),
            attack_style: category_attack_style(weapon.category),
        })
        .collect()
}

/// Builds the combat fighter used for either local or networked players.
fn fighter_from_player(player: &Player) -> Fighter {
    Fighter {
        max_health: player.max_health() as f32,
        health: player.health() as f32,
        display_health: player.health() as f32,
        max_mana: player.max_mana() as f32,
        mana: player.mana() as f32,
        display_mana: player.mana() as f32,
        base_attack: player.attack() as f32,
        base_defense: player.defense() as f32,
        base_initiative: player.initiative() as f32,
        base_attack_speed: player.attack_speed(),
        crit_chance: player.crit_chance(),
        health_regen: player.health_regen() as f32,
        mana_regen: player.mana_regen() as f32,
        attack_timer: 0.0,
        effects: Vec::new(),
        weapon_effects: player_equipment_effects(player),
        attack_style: player_attack_style(player),
        intelligence_mod: player.intelligence_mod() as f32,
        passive_modifiers: player.active_modifiers(),
        mutation: player.mutation,
        alive: true,
        weapons: player_combat_weapons(player),
    }
}

/// Builds a pet combatant while applying all pet-specific owner modifiers.
fn fighter_from_pet(pet: &Monster, owner: &Player) -> Fighter {
    let attack = (pet.attack as i32 + owner.pet_attack_bonus()).max(0) as f32;
    let defense = (pet.defense as i32 + owner.pet_defense_bonus()).max(0) as f32;
    let initiative = (pet.initiative as i32 + owner.pet_initiative_bonus()).max(0) as f32;
    let attack_speed = pet.attack_speed * owner.pet_attack_speed_multiplier();
    let effects = pet
        .effects
        .iter()
        .map(|effect| WeaponEffect {
            effect: effect.clone(),
            on_hit: true,
        })
        .collect::<Vec<_>>();

    Fighter {
        max_health: pet.max_health as f32,
        health: pet.health as f32,
        display_health: pet.health as f32,
        max_mana: 0.0,
        mana: 0.0,
        display_mana: 0.0,
        base_attack: attack,
        base_defense: defense,
        base_initiative: initiative,
        base_attack_speed: attack_speed,
        crit_chance: 0.0,
        health_regen: pet.health_regen as f32,
        mana_regen: 0.0,
        attack_timer: 0.0,
        effects: Vec::new(),
        weapon_effects: effects.clone(),
        attack_style: AttackStyle::Other,
        intelligence_mod: 0.0,
        passive_modifiers: Vec::new(),
        mutation: None,
        alive: true,
        weapons: vec![CombatWeapon {
            name: "Basic Attack".to_string(),
            kind: Kind::Physical,
            category: Category::Melee,
            attack_speed,
            attack_timer: 0.0,
            attack,
            crit_chance: 0.0,
            effects,
            attack_style: AttackStyle::Other,
        }],
    }
}

/// Builds a non-player monster combatant.
fn fighter_from_monster(monster: &Monster) -> Fighter {
    let effects = monster
        .effects
        .iter()
        .map(|effect| WeaponEffect {
            effect: effect.clone(),
            on_hit: true,
        })
        .collect::<Vec<_>>();
    Fighter {
        max_health: monster.max_health as f32,
        health: monster.health as f32,
        display_health: monster.health as f32,
        max_mana: 0.0,
        mana: 0.0,
        display_mana: 0.0,
        base_attack: monster.attack as f32,
        base_defense: monster.defense as f32,
        base_initiative: monster.initiative as f32,
        base_attack_speed: monster.attack_speed,
        crit_chance: 0.0,
        health_regen: monster.health_regen as f32,
        mana_regen: 0.0,
        attack_timer: 0.0,
        effects: Vec::new(),
        weapon_effects: effects.clone(),
        attack_style: AttackStyle::Other,
        intelligence_mod: 0.0,
        passive_modifiers: monster.modifiers.clone(),
        mutation: None,
        alive: true,
        weapons: vec![CombatWeapon {
            name: "Basic Attack".to_string(),
            kind: Kind::Physical,
            category: Category::Melee,
            attack_speed: monster.attack_speed,
            attack_timer: 0.0,
            attack: monster.attack as f32,
            crit_chance: 0.0,
            effects,
            attack_style: AttackStyle::Other,
        }],
    }
}

/// Builds the complete combat snapshot from the player, pet, monster, and optional duel.
///
/// Shared character attack is divided between two attacking weapons so dual-wielding adds
/// weapon cadence and effects without counting the same base statistics twice.
pub fn setup_combat_state(
    mut commands: Commands,
    player: Res<Player>,
    active_monster: Option<Res<ActiveMonster>>,
    settings: Res<crate::core::settings::Settings>,
    localization: Res<crate::core::localization::Localization>,
    existing_state: Option<Res<CombatState>>,
    duel_state: Option<Res<crate::core::network::DuelState>>,
) {
    if existing_state.is_some() {
        return;
    }
    let Some(active_monster) = active_monster else {
        return;
    };
    let monster = &active_monster.monster;

    let player_fighter = fighter_from_player(&player);
    let pet_fighter = player.pet.as_ref().map(|pet| fighter_from_pet(pet, &player));
    let opponent = duel_state.as_ref().and_then(|duel| duel.opponent.as_ref());
    let enemy_fighter = opponent.map_or_else(|| fighter_from_monster(monster), fighter_from_player);
    let enemy_pet_fighter = opponent
        .and_then(|opponent| opponent.pet.as_ref().map(|pet| fighter_from_pet(pet, opponent)));

    let build_ability_slots = |active_abilities: &[Option<String>]| {
        active_abilities
            .iter()
            .map(|opt| {
                let (cooldown, mana_cost) = opt
                    .as_deref()
                    .and_then(get_ability)
                    .map(|a| (a.cooldown, a.mana_cost))
                    .unwrap_or((0.0, 0));
                AbilitySlot {
                    key: opt.clone(),
                    cooldown,
                    remaining: 0.0,
                    mana_cost: mana_cost.saturating_mul(ABILITY_MANA_COST_MULTIPLIER),
                }
            })
            .collect::<Vec<_>>()
    };
    let abilities = build_ability_slots(&player.active_abilities);
    let player_consumables = select_combat_consumables(&player);
    let enemy_consumables = opponent.map(select_combat_consumables).unwrap_or_default();
    let enemy_abilities = duel_state
        .as_ref()
        .and_then(|duel| duel.opponent.as_ref())
        .map(|opp| build_ability_slots(&opp.active_abilities))
        .unwrap_or_default();
    let enemy_max_poise = 42.0 + monster.level as f32 * 4.0;
    let enemy_tactics = opponent.is_none().then(|| EnemyTactics {
        archetype: monster.archetype,
        rotation: enemy_move_rotation(monster.archetype, monster.level),
        next_index: 0,
        recovery: 2.6,
        recovery_max: 2.6,
        active_cast: None,
        phase_two: false,
    });

    commands.insert_resource(CombatState {
        player: player_fighter,
        pet: pet_fighter,
        enemy: enemy_fighter,
        enemy_pet: enemy_pet_fighter,
        abilities,
        enemy_abilities,
        player_consumables,
        enemy_consumables,
        stance: CombatStance::Aggressive,
        guard_remaining: 0.0,
        perfect_guard_remaining: 0.0,
        enemy_poise: enemy_max_poise,
        enemy_max_poise,
        enemy_break_remaining: 0.0,
        enemy_tactics,
        status: CombatStatus::Ongoing,
        player_won: false,
        player_level: player.level(),
        enemy_level: monster.level,
        mutation_candidate: opponent
            .is_none()
            .then(|| Mutation::from_monster_name(&monster.name))
            .flatten(),
        fx: Vec::new(),
        paused: false,
        dodge_word: localization.get("general.dodge", settings.language),
        miss_word: localization.get("general.miss", settings.language),
        xp_word: localization.get("general.xp", settings.language),
        guard_word: localization.get("combat.guard", settings.language),
        parry_word: localization.get("combat.parry", settings.language),
        break_word: localization.get("combat.break", settings.language),
        shatter_word: localization.get("combat.combo_shatter", settings.language),
        detonate_word: localization.get("combat.combo_detonate", settings.language),
        doom_word: localization.get("combat.combo_doom", settings.language),
        exploit_word: localization.get("combat.combo_exploit", settings.language),
        cleanse_word: localization.get("combat.combo_cleanse", settings.language),
        phase_word: localization.get("combat.phase_two", settings.language),
    });
}

// ---------------------------------------------------------------------------
// Combat tick
// ---------------------------------------------------------------------------

/// Performs the dodge chance operation.
fn dodge_chance(attacker_init: f32, defender_init: f32) -> f32 {
    (DODGE_BASE_CHANCE + (defender_init - attacker_init) * DODGE_POINT_MULTIPLIER)
        .clamp(DODGE_CHANCE_MIN, DODGE_CHANCE_MAX)
}

/// Performs the ability dodge chance operation.
fn ability_dodge_chance(
    attacker_init: f32,
    defender_init: f32,
    caster_intelligence_mod: f32,
) -> f32 {
    (DODGE_BASE_CHANCE + (defender_init - attacker_init) * DODGE_POINT_MULTIPLIER
        - caster_intelligence_mod * DODGE_POINT_MULTIPLIER)
        .clamp(DODGE_CHANCE_MIN, DODGE_CHANCE_MAX)
}

/// Computes damage.
fn compute_damage(attack: f32, defense: f32, crit: bool, incoming_mult: f32) -> f32 {
    let base = (attack * attack) / (attack + defense).max(1.0);
    let mut rng = rng();
    let variance = rng.random_range(0.85..1.15);
    let mut dmg = base * variance * incoming_mult;
    if crit {
        dmg *= 2.0;
    }
    dmg.max(1.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Who {
    Player,
    Pet,
    Enemy,
    EnemyPet,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AttackOutcome {
    Miss,
    Dodge,
    Hit,
}

/// Performs the random sword slice key operation.
fn random_sword_slice_key() -> &'static str {
    let mut rng = rng();
    match rng.random_range(0..4) {
        0 => "sword_slice",
        1 => "sword_slice_2",
        2 => "sword_slice_3",
        _ => "sword_slice_violent",
    }
}

/// Handles attack launch sound.
fn on_attack_launch_sound(style: AttackStyle) -> Option<&'static str> {
    match style {
        AttackStyle::Range => Some("arrow_swish"),
        _ => None,
    }
}

/// Handles attack hit sound.
fn on_attack_hit_sound(style: AttackStyle) -> &'static str {
    match style {
        AttackStyle::Range => "arrow_impact",
        AttackStyle::Melee | AttackStyle::Finesse => random_sword_slice_key(),
        AttackStyle::Other => "armor_impact",
    }
}

/// Handles attack dodge sound.
fn on_attack_dodge_sound(style: AttackStyle) -> &'static str {
    match style {
        AttackStyle::Melee | AttackStyle::Finesse => "sword_clash",
        _ => "armor_impact",
    }
}

impl CombatState {
    /// Performs the get operation.
    fn get(&self, who: Who) -> Option<&Fighter> {
        match who {
            Who::Player => Some(&self.player),
            Who::Pet => self.pet.as_ref(),
            Who::Enemy => Some(&self.enemy),
            Who::EnemyPet => self.enemy_pet.as_ref(),
        }
    }

    /// Returns mut.
    fn get_mut(&mut self, who: Who) -> Option<&mut Fighter> {
        match who {
            Who::Player => Some(&mut self.player),
            Who::Pet => self.pet.as_mut(),
            Who::Enemy => Some(&mut self.enemy),
            Who::EnemyPet => self.enemy_pet.as_mut(),
        }
    }
}

/// Resolves one basic attack, including avoidance, damage, lifesteal, and equipment effects.
fn resolve_basic_attack(
    state: &mut CombatState,
    attacker: Who,
    defender: Who,
    weapon_index: usize,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) -> Option<(AttackStyle, AttackOutcome)> {
    let (
        atk,
        atk_init,
        crit_chance,
        extra_crit,
        miss,
        weapon_effects,
        lifesteal,
        healing_multiplier,
        attack_style,
        kind,
        category,
        outgoing_multiplier,
    ) = {
        let a = match state.get(attacker) {
            Some(a) if a.alive => a,
            _ => return None,
        };
        let weapon = a.weapons.get(weapon_index)?;
        (
            a.eff_attack_for(weapon.attack),
            a.eff_initiative(),
            weapon.crit_chance,
            a.extra_crit(),
            a.miss_chance(),
            weapon.effects.clone(),
            a.lifesteal(),
            a.healing_multiplier(),
            weapon.attack_style,
            weapon.kind,
            weapon.category,
            a.outgoing_damage_multiplier(weapon.kind, Some(weapon.category)),
        )
    };

    let (def, def_init, can_dodge, incoming_mult, resistance_multiplier, def_alive) = {
        let d = state.get(defender)?;
        (
            d.eff_defense(),
            d.eff_initiative(),
            d.can_dodge(),
            d.incoming_multiplier(),
            d.incoming_damage_multiplier(kind, Some(category)),
            d.alive,
        )
    };
    if !def_alive {
        return None;
    }

    let fx_side = side_of(attacker);
    let def_fx_side = side_of(defender);
    let mut rng = rng();

    if rng.random_bool(miss as f64) {
        let miss_word = state.miss_word.clone();
        state.fx.push(CombatFx {
            side: fx_side,
            text: miss_word,
            color: Color::srgb(0.7, 0.7, 0.7),
        });
        return Some((attack_style, AttackOutcome::Miss));
    }
    if can_dodge && rng.random_bool(dodge_chance(atk_init, def_init) as f64) {
        let dodge_word = state.dodge_word.clone();
        state.fx.push(CombatFx {
            side: def_fx_side,
            text: dodge_word,
            color: Color::srgb(0.85, 0.85, 0.4),
        });
        return Some((attack_style, AttackOutcome::Dodge));
    }

    let stance_crit = if attacker == Who::Player {
        state.stance.critical_chance_bonus()
    } else {
        0.0
    };
    let crit = rng.random_bool((crit_chance + extra_crit + stance_crit).clamp(0.0, 1.0) as f64);

    // Bleed: consume a one-shot bleed buff on the attacker for bonus damage.
    let mut bonus_pct = 0.0;
    if let Some(a) = state.get_mut(attacker) {
        if let Some(pos) = a.effects.iter().position(|te| matches!(te.effect, Effect::Bleed { .. }))
        {
            if let Effect::Bleed {
                damage_pct,
            } = a.effects[pos].effect
            {
                bonus_pct = damage_pct / 100.0;
            }
            a.effects.remove(pos);
        }
    }

    let mut dmg =
        compute_damage(atk, def, crit, incoming_mult) * outgoing_multiplier * resistance_multiplier;
    dmg *= 1.0 + bonus_pct;
    if attacker == Who::Player {
        dmg *= state.stance.attack_damage_multiplier();
    }
    if defender == Who::Player {
        dmg *= state.stance.incoming_damage_multiplier();
        dmg = mitigate_with_guard(state, dmg, play_audio_msg);
    }

    if let Some(d) = state.get_mut(defender) {
        d.take_damage(dmg);
    }

    state.fx.push(CombatFx {
        side: def_fx_side,
        text: format!("-{}", dmg.round() as i32),
        color: if crit {
            Color::srgb(1.0, 0.85, 0.2)
        } else {
            Color::srgb(1.0, 0.4, 0.4)
        },
    });

    if attacker == Who::Player {
        let poise_damage = state.stance.poise_damage();
        if matches!(defender, Who::Enemy | Who::EnemyPet) {
            damage_enemy_poise(state, poise_damage, play_audio_msg);
        }
    } else if attacker == Who::Pet && matches!(defender, Who::Enemy | Who::EnemyPet) {
        damage_enemy_poise(state, 3.0, play_audio_msg);
    }

    // Lifesteal heals the attacker.
    if lifesteal > 0.0 {
        if let Some(a) = state.get_mut(attacker) {
            a.heal(dmg * lifesteal * healing_multiplier);
        }
    }

    // Thorns on the defender reflect damage back to the attacker.
    let thorns: f32 = state
        .get(defender)
        .map(|d| {
            d.effects
                .iter()
                .filter_map(|te| {
                    if let Effect::Thorns {
                        damage_reflected_pct,
                        ..
                    } = &te.effect
                    {
                        Some(damage_reflected_pct / 100.0)
                    } else {
                        None
                    }
                })
                .sum()
        })
        .unwrap_or(0.0);
    if thorns > 0.0 {
        if let Some(a) = state.get_mut(attacker) {
            a.take_damage(dmg * thorns);
        }
    }

    for we in weapon_effects.iter().filter(|w| w.on_hit) {
        // Self-buffs (Bleed, Clearcasting, Lifesteal, ManaFlow, Thorns, ...)
        // belong on the wielder; offensive effects hit the target.
        let tgt = if effect_targets_self(&we.effect) {
            attacker
        } else {
            defender
        };
        apply_effect(state, attacker, tgt, &we.effect, kind, Some(category), play_audio_msg);
    }
    // Apply the defender's on-being-hit weapon effects (shields/books).
    let def_when_hit: Vec<Effect> = state
        .get(defender)
        .map(|d| d.weapon_effects.iter().filter(|w| !w.on_hit).map(|w| w.effect.clone()).collect())
        .unwrap_or_default();
    for e in def_when_hit {
        let tgt = if effect_targets_self(&e) {
            defender
        } else {
            attacker
        };
        apply_effect(state, defender, tgt, &e, Kind::Physical, None, play_audio_msg);
    }
    Some((attack_style, AttackOutcome::Hit))
}

/// Performs the side of operation.
fn side_of(who: Who) -> FxSide {
    match who {
        Who::Player | Who::Pet => FxSide::Player,
        Who::Enemy | Who::EnemyPet => FxSide::Enemy,
    }
}

/// Damages enemy Poise and triggers a vulnerable break when it is exhausted.
fn damage_enemy_poise(
    state: &mut CombatState,
    amount: f32,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.enemy_break_remaining > 0.0 || !state.enemy.alive {
        return;
    }
    state.enemy_poise = (state.enemy_poise - amount.max(0.0)).max(0.0);
    if state.enemy_poise > 0.0 {
        return;
    }

    state.enemy_break_remaining = BREAK_STUN_DURATION;
    if let Some(tactics) = state.enemy_tactics.as_mut() {
        tactics.active_cast = None;
        tactics.recovery = BREAK_STUN_DURATION + 1.0;
        tactics.recovery_max = tactics.recovery;
    }
    push_timed(
        state,
        Who::Enemy,
        Effect::Stun {
            duration: BREAK_STUN_DURATION,
        },
        1.0,
    );
    push_timed(
        state,
        Who::Enemy,
        Effect::Vulnerability {
            damage_pct: BREAK_VULNERABILITY_PERCENT,
            duration: BREAK_VULNERABILITY_DURATION,
        },
        1.0,
    );
    state.fx.push(CombatFx {
        side: FxSide::Enemy,
        text: state.break_word.clone(),
        color: Color::srgb(0.85, 0.55, 1.0),
    });
    play_audio_msg.write(PlayAudioMsg::new("sword_clash"));
}

/// Reduces an incoming player hit when Guard is active and resolves perfect parries.
fn mitigate_with_guard(
    state: &mut CombatState,
    damage: f32,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) -> f32 {
    if state.guard_remaining <= 0.0 {
        return damage;
    }
    let perfect = state.perfect_guard_remaining > 0.0;
    state.guard_remaining = 0.0;
    state.perfect_guard_remaining = 0.0;
    if perfect {
        damage_enemy_poise(state, PERFECT_GUARD_POISE_DAMAGE, play_audio_msg);
        state.fx.push(CombatFx {
            side: FxSide::Player,
            text: state.parry_word.clone(),
            color: Color::srgb(1.0, 0.86, 0.35),
        });
        play_audio_msg.write(PlayAudioMsg::new("sword_clash"));
        damage * (1.0 - PERFECT_GUARD_DAMAGE_REDUCTION)
    } else {
        state.fx.push(CombatFx {
            side: FxSide::Player,
            text: state.guard_word.clone(),
            color: Color::srgb(0.45, 0.75, 1.0),
        });
        play_audio_msg.write(PlayAudioMsg::new("armor_impact"));
        damage * (1.0 - GUARD_DAMAGE_REDUCTION)
    }
}

/// Applies a single effect from `source` onto `target`.
fn apply_effect(
    state: &mut CombatState,
    source: Who,
    target: Who,
    effect: &Effect,
    kind: Kind,
    category: Option<Category>,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.get(target).is_some_and(|fighter| fighter.is_immune_to_effect(effect)) {
        return;
    }

    let outgoing_multiplier = state
        .get(source)
        .map(|fighter| fighter.outgoing_damage_multiplier(kind, category))
        .unwrap_or(1.0);
    let incoming_multiplier = state
        .get(target)
        .map(|fighter| fighter.incoming_damage_multiplier(kind, category))
        .unwrap_or(1.0);
    let damage_multiplier = outgoing_multiplier * incoming_multiplier;
    let healing_multiplier = state.get(source).map(Fighter::healing_multiplier).unwrap_or(1.0);

    if matches!(target, Who::Player) && effect.debuff_icon().is_some() {
        play_audio_msg.write(PlayAudioMsg::new("curse"));
    }
    match effect {
        Effect::Heal {
            heal_pct,
        } => {
            if let Some(t) = state.get_mut(target) {
                let missing = t.max_health - t.health;
                t.heal(missing * (*heal_pct as f32 / 100.0) * healing_multiplier);
            }
        },
        Effect::Pierce {
            damage,
        }
        | Effect::Burn {
            damage,
            ..
        }
        | Effect::Poison {
            damage,
            ..
        } => {
            // Pierce is instant; Burn/Poison handled as DoT below too, but their
            // initial application also lands an instant tick for responsiveness.
            let scaled_damage = *damage as f32 * damage_multiplier;
            if let Some(t) = state.get_mut(target) {
                t.take_damage(scaled_damage);
            }
            let color = match effect {
                Effect::Pierce {
                    ..
                } => Color::srgb(1.0, 0.5, 0.3),
                Effect::Burn {
                    ..
                } => Color::srgb(1.0, 0.3, 0.1),
                Effect::Poison {
                    ..
                } => Color::srgb(0.2, 0.8, 0.2),
                _ => Color::WHITE,
            };
            state.fx.push(CombatFx {
                side: side_of(target),
                text: format!("-{}", scaled_damage.round() as i32),
                color,
            });
            push_timed(state, target, effect.clone(), damage_multiplier);
        },
        Effect::InstantMana {
            amount,
        } => {
            if let Some(t) = state.get_mut(target) {
                t.restore_mana(*amount as f32);
            }
        },
        Effect::ManaBurn {
            amount,
        } => {
            if let Some(t) = state.get_mut(target) {
                t.mana = (t.mana - *amount as f32 * damage_multiplier).max(0.0);
            }
        },
        Effect::Manasteal {
            percentage,
        } => {
            let stolen = state.get(target).map(|t| t.mana * percentage / 100.0).unwrap_or(0.0);
            if let Some(t) = state.get_mut(target) {
                t.mana = (t.mana - stolen).max(0.0);
            }
            if let Some(s) = state.get_mut(source) {
                s.restore_mana(stolen);
            }
        },
        Effect::Purge => {
            if let Some(t) = state.get_mut(target) {
                t.effects.retain(|te| is_positive(&te.effect));
            }
        },
        Effect::Cleave {
            damage_pct,
            ..
        } => {
            // A cleaving strike: deal a percentage of the source's attack as
            // immediate damage to the target, mitigated by its defense.
            let atk = state.get(source).map(|s| s.eff_attack_for(s.base_attack)).unwrap_or(0.0);
            let (def, incoming, resistance) = state
                .get(target)
                .map(|t| {
                    (
                        t.eff_defense(),
                        t.incoming_multiplier(),
                        t.incoming_damage_multiplier(kind, category),
                    )
                })
                .unwrap_or((0.0, 1.0, 1.0));
            let dmg = compute_damage(atk * (damage_pct / 100.0), def, false, incoming)
                * outgoing_multiplier
                * resistance;
            if let Some(t) = state.get_mut(target) {
                t.take_damage(dmg);
            }
            state.fx.push(CombatFx {
                side: side_of(target),
                text: format!("-{}", dmg.round() as i32),
                color: Color::srgb(1.0, 0.6, 0.2),
            });
        },
        // Timed buffs / debuffs / damage-over-time / heal-over-time.
        _ => {
            let magnitude_multiplier = match effect {
                Effect::Curse {
                    ..
                } => damage_multiplier,
                Effect::Regen {
                    ..
                } => healing_multiplier,
                _ => 1.0,
            };
            push_timed(state, target, effect.clone(), magnitude_multiplier);
        },
    }
}

/// Adds a timed effect, refreshing non-stackable effects of the same variant.
fn push_timed(state: &mut CombatState, target: Who, effect: Effect, magnitude_multiplier: f32) {
    let duration = effect_duration(&effect);
    if let Some(t) = state.get_mut(target) {
        let refresh_existing = !matches!(effect, Effect::Bleed { .. } | Effect::Curse { .. });
        if refresh_existing {
            if let Some(existing) = t.effects.iter_mut().find(|timed| {
                std::mem::discriminant(&timed.effect) == std::mem::discriminant(&effect)
            }) {
                existing.effect = effect;
                existing.remaining = existing.remaining.max(duration);
                existing.magnitude_multiplier = magnitude_multiplier;
                return;
            }
        }
        t.effects.push(TimedEffect {
            effect,
            remaining: duration,
            tick_acc: 0.0,
            magnitude_multiplier,
        });
    }
}

/// Returns whether positive.
fn is_positive(effect: &Effect) -> bool {
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
            | Effect::Lifesteal { .. }
            | Effect::ManaFlow { .. }
            | Effect::MonarchShield { .. }
            | Effect::Regen { .. }
            | Effect::SoulLink { .. }
            | Effect::StatBoost { .. }
            | Effect::Taunt { .. }
            | Effect::Thorns { .. }
    )
}

/// Whether an effect should be applied to the caster's own side (self / pet)
/// rather than the opponent. This keeps buffs and self-affecting mechanics on
/// the caster even when an ability bundles them, instead of trusting only the
/// ability-level `on_self` flag.
fn effect_targets_self(effect: &Effect) -> bool {
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
            ..
        }
        | Effect::Poison {
            duration,
            ..
        }
        | Effect::Paranoia {
            duration,
            ..
        }
        | Effect::Regen {
            duration,
            ..
        }
        | Effect::Silence {
            duration,
            ..
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
            ..
        }
        | Effect::Taunt {
            duration,
            ..
        }
        | Effect::Thorns {
            duration,
            ..
        }
        | Effect::Immobilize {
            duration,
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

/// Per-second damage/heal applied by a timed effect; returns (hp_delta, mp_delta).
fn effect_per_second(effect: &Effect) -> (f32, f32) {
    match effect {
        Effect::Burn {
            damage,
            ..
        }
        | Effect::Poison {
            damage,
            ..
        } => (-(*damage as f32), 0.0),
        Effect::Regen {
            heal,
            ..
        } => (*heal as f32, 0.0),
        Effect::ManaFlow {
            amount,
            ..
        } => (0.0, *amount as f32),
        _ => (0.0, 0.0),
    }
}

/// Advances fighter effects.
fn tick_fighter_effects(fighter: &mut Fighter, dt: f32) -> Vec<(FxSide, String, Color)> {
    let mut fx = Vec::new();
    let mut curse_damage = Vec::new();
    for te in fighter.effects.iter_mut() {
        let (hp_s, mp_s) = effect_per_second(&te.effect);
        let hp_s = hp_s * te.magnitude_multiplier;
        let mp_s = mp_s * te.magnitude_multiplier;
        if hp_s != 0.0 || mp_s != 0.0 {
            te.tick_acc += dt;
            while te.tick_acc >= 1.0 {
                te.tick_acc -= 1.0;
                if hp_s < 0.0 {
                    fighter.health = (fighter.health - (-hp_s)).max(0.0);
                } else if hp_s > 0.0 {
                    fighter.health = (fighter.health + hp_s).min(fighter.max_health);
                }
                if mp_s > 0.0 {
                    fighter.mana = (fighter.mana + mp_s).min(fighter.max_mana);
                }
            }
        }
        te.remaining -= dt;
        // Curse detonates when it expires.
        if let Effect::Curse {
            damage,
            ..
        } = &te.effect
        {
            if te.remaining <= 0.0 {
                curse_damage.push(*damage as f32 * te.magnitude_multiplier);
            }
        }
    }
    for d in curse_damage {
        fighter.health = (fighter.health - d).max(0.0);
        fx.push((FxSide::Player, format!("-{}", d as i32), Color::srgb(0.6, 0.2, 0.8)));
    }
    fighter.effects.retain(|te| te.remaining > 0.0);
    if fighter.health <= 0.0 {
        fighter.alive = false;
    }
    fx
}

/// Returns the live fighter targeted by a telegraphed enemy move.
fn enemy_move_target(state: &CombatState, target: EnemyMoveTarget) -> Who {
    match target {
        EnemyMoveTarget::Player => Who::Player,
        EnemyMoveTarget::Pet if state.pet.as_ref().is_some_and(|fighter| fighter.alive) => Who::Pet,
        EnemyMoveTarget::Pet => Who::Player,
        EnemyMoveTarget::SelfSide => Who::Enemy,
    }
}

/// Deals a telegraphed enemy move's direct damage through defense, stance, and Guard.
fn deal_enemy_move_damage(
    state: &mut CombatState,
    target: Who,
    attack_multiplier: f32,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) -> f32 {
    let attack = state.enemy.eff_attack_for(state.enemy.base_attack) * attack_multiplier;
    let Some(defender) = state.get(target) else {
        return 0.0;
    };
    let mut damage =
        compute_damage(attack, defender.eff_defense(), false, defender.incoming_multiplier());
    if target == Who::Player {
        damage *= state.stance.incoming_damage_multiplier();
        damage = mitigate_with_guard(state, damage, play_audio_msg);
    }
    if let Some(defender) = state.get_mut(target) {
        defender.take_damage(damage);
    }
    state.fx.push(CombatFx {
        side: side_of(target),
        text: format!("-{}", damage.round() as i32),
        color: Color::srgb(1.0, 0.25, 0.18),
    });
    damage
}

/// Resolves one completed telegraphed enemy move.
fn resolve_enemy_move(
    state: &mut CombatState,
    movement: EnemyMove,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    let level = state.enemy_level;
    let target = enemy_move_target(state, movement.target);
    match movement.kind {
        EnemyMoveKind::CrushingBlow => {
            deal_enemy_move_damage(state, target, 1.65, play_audio_msg);
            play_audio_msg.write(PlayAudioMsg::new("armor_impact"));
        },
        EnemyMoveKind::DarkRitual => {
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Berserk {
                    attack_pct: 24.0 + level as f32,
                    duration: 6.0,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Haste {
                    initiative_pct: 18.0 + level as f32 * 0.5,
                    duration: 6.0,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::VenomSpray => {
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Poison {
                    damage: 1 + level.div_ceil(3),
                    duration: 5.0,
                },
                Kind::Nature,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::DefensiveShell => {
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Fortify {
                    defense_pct: 38.0 + level as f32,
                    duration: 6.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Thorns {
                    damage_reflected_pct: 12.0 + level as f32 * 0.6,
                    duration: 6.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("armor_impact"));
        },
        EnemyMoveKind::Execution => {
            let low_health = state
                .get(target)
                .is_some_and(|fighter| fighter.health <= fighter.max_health * 0.35);
            deal_enemy_move_damage(
                state,
                target,
                if low_health {
                    2.35
                } else {
                    0.90
                },
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("sword_slice_violent"));
        },
        EnemyMoveKind::DevourCompanion => {
            let damage = deal_enemy_move_damage(
                state,
                target,
                if target == Who::Pet {
                    1.75
                } else {
                    1.05
                },
                play_audio_msg,
            );
            state.enemy.heal(damage * 0.65);
            play_audio_msg.write(PlayAudioMsg::new("sword_slice_violent"));
        },
        EnemyMoveKind::DrainLife => {
            let damage = deal_enemy_move_damage(state, target, 1.05, play_audio_msg);
            state.enemy.heal(damage * 0.80);
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::WitheringHex => {
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Curse {
                    damage: 4 + level * 2,
                    timer: 4,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Vulnerability {
                    damage_pct: 14.0 + level as f32 * 0.6,
                    duration: 5.0,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::GlacialBreath => {
            deal_enemy_move_damage(state, target, 1.20, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Freeze {
                    attack_speed_pct: -(18.0 + level as f32),
                    duration: 5.0,
                },
                Kind::Ice,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("cast"));
        },
        EnemyMoveKind::SmokeVeil => {
            apply_effect(
                state,
                Who::Enemy,
                Who::Player,
                &Effect::Blind {
                    miss_pct: 20.0 + level as f32,
                    duration: 5.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Haste {
                    initiative_pct: 24.0 + level as f32,
                    duration: 5.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::SunderArmor => {
            deal_enemy_move_damage(state, target, 1.25, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Vulnerability {
                    damage_pct: 12.0 + level as f32 * 0.5,
                    duration: 5.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("armor_impact"));
        },
        EnemyMoveKind::FlameBreath => {
            deal_enemy_move_damage(state, target, 1.15, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Burn {
                    damage: 1 + level.div_ceil(3),
                    duration: 5.0,
                },
                Kind::Fire,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("cast"));
        },
        EnemyMoveKind::EntanglingRoots => {
            deal_enemy_move_damage(state, target, 0.85, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Immobilize {
                    duration: 3.0,
                },
                Kind::Nature,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("cast"));
        },
        EnemyMoveKind::BloodFeast => {
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Regen {
                    heal: 1 + level.div_ceil(3),
                    duration: 6.0,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Lifesteal {
                    percentage: 18.0 + level as f32 * 0.5,
                    duration: 6.0,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::ArcaneRupture => {
            deal_enemy_move_damage(state, target, 1.10, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::ManaBurn {
                    amount: 4 + level * 2,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("cast"));
        },
        EnemyMoveKind::WarCry => {
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Berserk {
                    attack_pct: 18.0 + level as f32,
                    duration: 6.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Haste {
                    initiative_pct: 14.0 + level as f32 * 0.5,
                    duration: 6.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("horn"));
        },
        EnemyMoveKind::Earthshatter => {
            deal_enemy_move_damage(state, target, 1.45, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Stun {
                    duration: 1.2 + level as f32 * 0.02,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("armor_impact"));
        },
        EnemyMoveKind::GraveChill => {
            deal_enemy_move_damage(state, target, 1.0, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Freeze {
                    attack_speed_pct: -(14.0 + level as f32),
                    duration: 5.0,
                },
                Kind::Ice,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Curse {
                    damage: 3 + level,
                    timer: 4,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::SoulSiphon => {
            let damage = deal_enemy_move_damage(state, target, 1.20, play_audio_msg);
            state.enemy.heal(damage * 0.65);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::ManaBurn {
                    amount: 3 + level * 2,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::ShieldBash => {
            deal_enemy_move_damage(state, target, 1.10, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Stun {
                    duration: 0.8 + level as f32 * 0.015,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("armor_impact"));
        },
        EnemyMoveKind::IronBulwark => {
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Fortify {
                    defense_pct: 45.0 + level as f32,
                    duration: 6.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Thorns {
                    damage_reflected_pct: 16.0 + level as f32 * 0.6,
                    duration: 6.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("armor_impact"));
        },
        EnemyMoveKind::FanOfKnives => {
            deal_enemy_move_damage(state, target, 1.0, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Poison {
                    damage: 1 + level.div_ceil(4),
                    duration: 6.0,
                },
                Kind::Nature,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("sword_slice_violent"));
        },
        EnemyMoveKind::Shadowstep => {
            deal_enemy_move_damage(state, target, 1.55, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Blind {
                    miss_pct: 18.0 + level as f32 * 0.7,
                    duration: 4.0,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("sword_slice_violent"));
        },
        EnemyMoveKind::CorrosiveBite => {
            deal_enemy_move_damage(state, target, 1.20, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Poison {
                    damage: 1 + level.div_ceil(3),
                    duration: 6.0,
                },
                Kind::Nature,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Vulnerability {
                    damage_pct: 10.0 + level as f32 * 0.5,
                    duration: 5.0,
                },
                Kind::Nature,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("sword_slice_violent"));
        },
        EnemyMoveKind::ParasiticBloom => {
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Regen {
                    heal: 2 + level.div_ceil(3),
                    duration: 7.0,
                },
                Kind::Nature,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                &Effect::Fortify {
                    defense_pct: 24.0 + level as f32,
                    duration: 7.0,
                },
                Kind::Nature,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("curse"));
        },
        EnemyMoveKind::Meteor => {
            deal_enemy_move_damage(state, target, 1.70, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Burn {
                    damage: 2 + level.div_ceil(3),
                    duration: 6.0,
                },
                Kind::Fire,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("cast"));
        },
        EnemyMoveKind::TemporalLock => {
            deal_enemy_move_damage(state, target, 0.75, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Stun {
                    duration: 1.8 + level as f32 * 0.02,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Paranoia {
                    initiative_pct: -(18.0 + level as f32),
                    duration: 5.0,
                },
                Kind::Shadow,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("cast"));
        },
        EnemyMoveKind::SavagePounce => {
            deal_enemy_move_damage(state, target, 1.40, play_audio_msg);
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Immobilize {
                    duration: 2.5,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("sword_slice_violent"));
        },
        EnemyMoveKind::TerrifyingRoar => {
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Paranoia {
                    initiative_pct: -(20.0 + level as f32),
                    duration: 6.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            apply_effect(
                state,
                Who::Enemy,
                target,
                &Effect::Vulnerability {
                    damage_pct: 12.0 + level as f32 * 0.5,
                    duration: 6.0,
                },
                Kind::Physical,
                None,
                play_audio_msg,
            );
            play_audio_msg.write(PlayAudioMsg::new("horn"));
        },
    }
}

/// Triggers the one-time low-health phase associated with an enemy archetype.
fn trigger_enemy_phase(
    state: &mut CombatState,
    archetype: MonsterArchetype,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    let duration = 14.0;
    match archetype {
        MonsterArchetype::Berserker => push_timed(
            state,
            Who::Enemy,
            Effect::Berserk {
                attack_pct: 38.0,
                duration,
            },
            1.0,
        ),
        MonsterArchetype::Beast => push_timed(
            state,
            Who::Enemy,
            Effect::BeastFrenzy {
                attack_pct: 25.0,
                attack_speed_pct: 28.0,
                duration,
            },
            1.0,
        ),
        MonsterArchetype::Necromancer | MonsterArchetype::Leech => {
            state.enemy.heal(state.enemy.max_health * 0.18);
            push_timed(
                state,
                Who::Enemy,
                Effect::Lifesteal {
                    percentage: 24.0,
                    duration,
                },
                1.0,
            );
        },
        MonsterArchetype::Knight => {
            push_timed(
                state,
                Who::Enemy,
                Effect::Fortify {
                    defense_pct: 42.0,
                    duration,
                },
                1.0,
            );
            push_timed(
                state,
                Who::Enemy,
                Effect::Thorns {
                    damage_reflected_pct: 24.0,
                    duration,
                },
                1.0,
            );
        },
        MonsterArchetype::Assassin => {
            push_timed(
                state,
                Who::Enemy,
                Effect::Haste {
                    initiative_pct: 35.0,
                    duration,
                },
                1.0,
            );
            push_timed(
                state,
                Who::Enemy,
                Effect::Focus {
                    crit_chance_pct: 25.0,
                    duration,
                },
                1.0,
            );
        },
        MonsterArchetype::Mage => {
            push_timed(
                state,
                Who::Enemy,
                Effect::Empower {
                    damage_pct: 32.0,
                    duration,
                },
                1.0,
            );
            push_timed(
                state,
                Who::Enemy,
                Effect::Haste {
                    initiative_pct: 25.0,
                    duration,
                },
                1.0,
            );
        },
    }
    state.fx.push(CombatFx {
        side: FxSide::Enemy,
        text: state.phase_word.clone(),
        color: Color::srgb(1.0, 0.32, 0.22),
    });
    play_audio_msg.write(PlayAudioMsg::new("warning"));
}

/// Advances the deterministic PvE intent rotation and resolves completed casts.
fn tick_enemy_tactics(
    state: &mut CombatState,
    dt: f32,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.enemy_tactics.is_none() || !state.enemy.alive {
        return;
    }
    let phase_archetype = state.enemy_tactics.as_ref().and_then(|tactics| {
        (!tactics.phase_two && state.enemy.health <= state.enemy.max_health * 0.50)
            .then_some(tactics.archetype)
    });
    if let Some(archetype) = phase_archetype {
        if let Some(tactics) = state.enemy_tactics.as_mut() {
            tactics.phase_two = true;
            if tactics.recovery > 0.8 {
                tactics.recovery = 0.8;
                tactics.recovery_max = 0.8;
            }
        }
        trigger_enemy_phase(state, archetype, play_audio_msg);
    }
    if state.enemy_break_remaining > 0.0 {
        return;
    }
    let enemy_can_cast = state.enemy.can_cast();
    let mut completed = None;
    if let Some(tactics) = state.enemy_tactics.as_mut() {
        if let Some(active_cast) = tactics.active_cast.as_mut() {
            active_cast.elapsed += dt;
            if active_cast.elapsed >= active_cast.movement.cast_time {
                completed = tactics.active_cast.take().map(|cast| cast.movement);
            }
        } else if enemy_can_cast {
            tactics.recovery = (tactics.recovery - dt).max(0.0);
            if tactics.recovery <= 0.0 && !tactics.rotation.is_empty() {
                let movement = tactics.rotation[tactics.next_index % tactics.rotation.len()];
                tactics.next_index = (tactics.next_index + 1) % tactics.rotation.len();
                tactics.active_cast = Some(EnemyCast {
                    movement,
                    elapsed: 0.0,
                });
                play_audio_msg.write(PlayAudioMsg::new("warning"));
            }
        }
    }
    if let Some(movement) = completed {
        resolve_enemy_move(state, movement, play_audio_msg);
        if let Some(tactics) = state.enemy_tactics.as_mut() {
            tactics.recovery = movement.recovery
                * if tactics.phase_two {
                    0.78
                } else {
                    1.0
                };
            tactics.recovery_max = tactics.recovery;
        }
    }
}

/// Advance the combat simulation by `dt` seconds, mutating only the
/// [`CombatState`]. Shared by single-player combat and networked duels (where
/// the host drives this directly and streams the result to the client).
pub fn step_combat(
    state: &mut CombatState,
    dt: f32,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.status == CombatStatus::Over {
        return;
    }
    if state.paused {
        return;
    }

    state.guard_remaining = (state.guard_remaining - dt).max(0.0);
    state.perfect_guard_remaining = (state.perfect_guard_remaining - dt).max(0.0);
    if state.enemy_break_remaining > 0.0 {
        state.enemy_break_remaining = (state.enemy_break_remaining - dt).max(0.0);
        if state.enemy_break_remaining <= 0.0 {
            state.enemy_poise = state.enemy_max_poise;
        }
    } else if state.enemy_tactics.as_ref().is_some_and(|tactics| tactics.active_cast.is_none()) {
        state.enemy_poise = (state.enemy_poise + dt * 4.0).min(state.enemy_max_poise);
    }

    // Ability cooldowns.
    for slots in [&mut state.abilities, &mut state.enemy_abilities] {
        for slot in slots.iter_mut() {
            if slot.remaining > 0.0 {
                slot.remaining = (slot.remaining - dt).max(0.0);
            }
        }
    }

    // Regen.
    for who in [Who::Player, Who::Pet, Who::Enemy, Who::EnemyPet] {
        if let Some(f) = state.get_mut(who) {
            regenerate_fighter(f, dt);
        }
    }

    tick_enemy_tactics(state, dt, play_audio_msg);

    // Damage/heal over time + effect expiry.
    for who in [Who::Player, Who::Pet, Who::Enemy, Who::EnemyPet] {
        let fx = if let Some(f) = state.get_mut(who) {
            tick_fighter_effects(f, dt)
        } else {
            continue;
        };
        let side = side_of(who);
        for (_, text, color) in fx {
            state.fx.push(CombatFx {
                side,
                text,
                color,
            });
        }
    }

    // Basic attacks paced by attack speed. A Taunt on the player's side forces
    // the enemy to strike the pet instead of the player (only while it lives).
    let enemy_target = if state.pet.as_ref().map(|p| p.alive).unwrap_or(false)
        && (state.player.has_taunt() || state.pet.as_ref().map(|p| p.has_taunt()).unwrap_or(false))
    {
        Who::Pet
    } else {
        Who::Player
    };
    let player_target = if state.enemy_pet.as_ref().map(|pet| pet.alive).unwrap_or(false)
        && (state.enemy.has_taunt()
            || state.enemy_pet.as_ref().map(|pet| pet.has_taunt()).unwrap_or(false))
    {
        Who::EnemyPet
    } else {
        Who::Enemy
    };
    for (attacker, defender) in [
        (Who::Player, player_target),
        (Who::Pet, player_target),
        (Who::Enemy, enemy_target),
        (Who::EnemyPet, enemy_target),
    ] {
        let enemy_is_casting = attacker == Who::Enemy
            && state.enemy_tactics.as_ref().is_some_and(|tactics| tactics.active_cast.is_some());
        let num_weapons = {
            let Some(f) = state.get(attacker) else {
                continue;
            };
            if !f.alive || !f.can_act() || enemy_is_casting {
                if let Some(f_mut) = state.get_mut(attacker) {
                    for w in &mut f_mut.weapons {
                        w.attack_timer = 0.0;
                    }
                }
                continue;
            }
            f.weapons.len()
        };

        for weapon_index in 0..num_weapons {
            let ready = {
                let stance_speed = if attacker == Who::Player {
                    state.stance.attack_speed_multiplier()
                } else {
                    1.0
                };
                let Some(f) = state.get_mut(attacker) else {
                    continue;
                };
                let Some(w) = f.weapons.get(weapon_index) else {
                    continue;
                };
                let speed = w.attack_speed * stance_speed;
                let period = f.attack_period_for(speed);
                let w_mut = f.weapons.get_mut(weapon_index).unwrap();
                w_mut.attack_timer += dt;
                if w_mut.attack_timer >= period {
                    w_mut.attack_timer -= period;
                    true
                } else {
                    false
                }
            };

            if ready && state.get(defender).map(|d| d.alive).unwrap_or(false) {
                let launch_style = state
                    .get(attacker)
                    .and_then(|f| f.weapons.get(weapon_index))
                    .map(|w| w.attack_style)
                    .unwrap_or(AttackStyle::Other);
                if let Some(key) = on_attack_launch_sound(launch_style) {
                    play_audio_msg.write(PlayAudioMsg::new(key));
                }

                if let Some((style, outcome)) =
                    resolve_basic_attack(state, attacker, defender, weapon_index, play_audio_msg)
                {
                    match outcome {
                        AttackOutcome::Hit => {
                            play_audio_msg.write(PlayAudioMsg::new(on_attack_hit_sound(style)));
                        },
                        AttackOutcome::Dodge => {
                            play_audio_msg.write(PlayAudioMsg::new(on_attack_dodge_sound(style)));
                        },
                        AttackOutcome::Miss => {
                            play_audio_msg.write(PlayAudioMsg::new("click"));
                        },
                    }
                }
            }
        }
    }

    // End condition: check both actual health and display health (rounded to 0)
    let player_side_dead = !state.player.alive || state.player.display_health.round() as i32 <= 0;
    let enemy_dead = !state.enemy.alive || state.enemy.display_health.round() as i32 <= 0;
    if enemy_dead || player_side_dead {
        state.status = CombatStatus::Over;
        state.player_won =
            enemy_dead && (state.player.alive && state.player.display_health.round() as i32 > 0);
        if state.player_won {
            play_audio_msg.write(PlayAudioMsg::new("victory").volume(-10.));
            let xp_reward = state.xp_reward();
            let xp_word = state.xp_word.clone();
            state.fx.push(CombatFx {
                side: FxSide::Player,
                text: format!("+{} {}", xp_reward, xp_word),
                color: Color::srgb(1.0, 0.9, 0.3),
            });
        } else {
            play_audio_msg.write(PlayAudioMsg::new("defeat"));
        }
    }
}

/// Restores exactly the fighter's listed Health and Mana regeneration each second.
fn regenerate_fighter(fighter: &mut Fighter, dt: f32) {
    if !fighter.alive {
        return;
    }
    fighter.health = (fighter.health + fighter.health_regen * dt).min(fighter.max_health);
    fighter.mana = (fighter.mana + fighter.mana_regen * dt).min(fighter.max_mana);
}

/// Performs the combat tick operation.
pub fn combat_tick(
    time: Res<Time>,
    combat_speed: Res<CombatSpeed>,
    mut state: Option<ResMut<CombatState>>,
    mut player: ResMut<Player>,
    active_monster: Option<ResMut<ActiveMonster>>,
    mut pending_hunt_pet: Option<ResMut<PendingHuntPet>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    let Some(state) = state.as_mut() else {
        return;
    };
    if state.status == CombatStatus::Over {
        return;
    }
    if state.paused {
        return;
    }
    let dt = time.delta_secs() * combat_speed.0;

    step_combat(state, dt, &mut play_audio_msg);

    // Losing a combat applies a 5 action-point penalty. `combat_tick` only runs
    // outside networked duels (see its `run_if` in the schedule), so PvP duels
    // are naturally excluded. This fires exactly once: on the frame the fight is
    // decided, later frames early-return above once the status is `Over`.
    if state.status == CombatStatus::Over && !state.player_won {
        player.ap += LOST_COMBAT_AP_PENALTY;
    }

    if state.status == CombatStatus::Over && state.player_won {
        if let Some(pending_hunt_pet) = pending_hunt_pet.as_mut() {
            if !pending_hunt_pet.offer_available {
                let mut rng = rng();
                if rng.random_bool(hunt_pet_chance(&player)) {
                    pending_hunt_pet.offer_available = true;
                }
            }
        }
    }

    // Sync working values back to the Player / pet / monster resources so other
    // displays stay coherent and combat results persist after leaving. Only write
    // when the rounded value actually changed to avoid needless change detection.
    let new_hp = state.player.health.round() as u32;
    if player.health() != new_hp {
        player.set_health(new_hp);
    }
    let new_mp = state.player.mana.round() as u32;
    if player.mana() != new_mp {
        player.set_mana(new_mp);
    }
    if let Some(pet_fighter) = state.pet.as_ref() {
        let new_pet_hp = pet_fighter.health.round() as u32;
        if player.pet.as_ref().map(|p| p.health) != Some(new_pet_hp) {
            if let Some(pet) = player.pet.as_mut() {
                pet.health = new_pet_hp;
            }
        }
    }
    if let Some(mut am) = active_monster {
        let new_enemy_hp = state.enemy.health.round() as u32;
        if am.monster.health != new_enemy_hp {
            am.monster.health = new_enemy_hp;
        }
    }
}

// ---------------------------------------------------------------------------
// Casting abilities & using consumables
// ---------------------------------------------------------------------------

/// Deals bonus combo damage and emits its tactical payoff label.
fn deal_combo_damage(state: &mut CombatState, damage: f32, kind: Kind, label: String) {
    let scaled = damage
        * state.player.outgoing_damage_multiplier(kind, None)
        * state.enemy.incoming_damage_multiplier(kind, None)
        * state.enemy.incoming_multiplier();
    state.enemy.take_damage(scaled);
    state.fx.push(CombatFx {
        side: FxSide::Enemy,
        text: format!("{label} -{}", scaled.round() as i32),
        color: Color::srgb(1.0, 0.68, 0.18),
    });
}

/// Consumes compatible enemy primers and returns bonus Poise damage for a payoff cast.
fn resolve_ability_combos(
    state: &mut CombatState,
    ability: &crate::core::catalog::abilities::Ability,
) -> f32 {
    let direct_power = ability
        .effects
        .iter()
        .map(|effect| match effect {
            Effect::Pierce {
                damage,
            } => *damage as f32,
            Effect::Cleave {
                damage_pct,
                ..
            } => state.player.base_attack * damage_pct / 100.0,
            _ => 0.0,
        })
        .sum::<f32>();
    if direct_power <= 0.0 {
        return 0.0;
    }

    let mut bonus_poise = 0.0;
    if matches!(ability.kind, Kind::Physical | Kind::Ice) {
        if let Some(index) = state
            .enemy
            .effects
            .iter()
            .position(|timed| matches!(timed.effect, Effect::Freeze { .. }))
        {
            state.enemy.effects.remove(index);
            deal_combo_damage(state, direct_power * 0.55, ability.kind, state.shatter_word.clone());
            bonus_poise += 20.0;
        }
    }
    if ability.kind == Kind::Fire {
        if let Some(index) =
            state.enemy.effects.iter().position(|timed| matches!(timed.effect, Effect::Burn { .. }))
        {
            let timed = state.enemy.effects.remove(index);
            let pending_damage = match timed.effect {
                Effect::Burn {
                    damage,
                    ..
                } => damage as f32 * timed.remaining.max(1.0) * timed.magnitude_multiplier,
                _ => 0.0,
            };
            deal_combo_damage(
                state,
                pending_damage * 0.75 + direct_power * 0.25,
                Kind::Fire,
                state.detonate_word.clone(),
            );
            bonus_poise += 10.0;
        }
    }
    if ability.kind == Kind::Shadow {
        if let Some(index) = state
            .enemy
            .effects
            .iter()
            .position(|timed| matches!(timed.effect, Effect::Curse { .. }))
        {
            let timed = state.enemy.effects.remove(index);
            let curse_damage = match timed.effect {
                Effect::Curse {
                    damage,
                    ..
                } => damage as f32 * timed.magnitude_multiplier,
                _ => 0.0,
            };
            deal_combo_damage(
                state,
                curse_damage + direct_power * 0.35,
                Kind::Shadow,
                state.doom_word.clone(),
            );
            bonus_poise += 12.0;
        }
    }
    if ability.kind == Kind::Physical
        && state
            .enemy
            .effects
            .iter()
            .any(|timed| matches!(timed.effect, Effect::Immobilize { .. } | Effect::Blind { .. }))
    {
        deal_combo_damage(state, direct_power * 0.30, Kind::Physical, state.exploit_word.clone());
        bonus_poise += 15.0;
    }
    bonus_poise
}

/// Rewards a successful self-cleanse based on the number of removed debuffs.
fn reward_purge_combo(state: &mut CombatState, removed_debuffs: usize) {
    if removed_debuffs == 0 {
        return;
    }
    let healing = state.player.max_health * 0.04 * removed_debuffs as f32;
    state.player.heal(healing);
    state.fx.push(CombatFx {
        side: FxSide::Player,
        text: state.cleanse_word.clone(),
        color: Color::srgb(0.45, 1.0, 0.72),
    });
}

/// Returns Guard's Mana cost for a player level.
pub fn guard_mana_cost(player_level: u32) -> f32 {
    player_level.max(1) as f32 * GUARD_MANA_COST_PER_LEVEL
}

/// Pays Guard's Mana cost and opens its short defensive window.
fn begin_guard(state: &mut CombatState) -> bool {
    let mana_cost = guard_mana_cost(state.player_level);
    if state.status == CombatStatus::Over
        || state.paused
        || !state.player.can_act()
        || state.guard_remaining > 0.0
        || state.player.mana < mana_cost
    {
        return false;
    }
    state.player.mana -= mana_cost;
    state.guard_remaining = GUARD_DURATION;
    state.perfect_guard_remaining = state.stance.perfect_guard_window();
    true
}

/// Activates the player's short Guard window when its Mana cost can be paid.
pub fn try_guard(state: &mut CombatState, play_audio_msg: &mut MessageWriter<PlayAudioMsg>) {
    if !begin_guard(state) {
        return;
    }
    play_audio_msg.write(PlayAudioMsg::new("sword_clash"));
}

/// Switches the player's auto-attack stance without interrupting attack progress.
pub fn set_combat_stance(
    state: &mut CombatState,
    stance: CombatStance,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.status == CombatStatus::Over || state.paused || state.stance == stance {
        return;
    }
    state.stance = stance;
    play_audio_msg.write(PlayAudioMsg::new("click"));
}

/// Performs the try cast ability operation.
pub fn try_cast_ability(
    state: &mut CombatState,
    index: usize,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.status == CombatStatus::Over {
        return;
    }
    let Some(slot) = state.abilities.get(index).cloned() else {
        return;
    };
    let Some(key) = slot.key.clone() else {
        return;
    };
    let effective_cost =
        (slot.mana_cost as f32 * (1.0 - state.player.clearcasting_reduction())).round();
    if slot.remaining > 0.0 || !state.player.can_cast() {
        play_audio_msg.write(PlayAudioMsg::new("error"));
        return;
    }
    if state.player.mana < effective_cost {
        play_audio_msg.write(PlayAudioMsg::new("error"));
        return;
    }
    let Some(ability) = get_ability(&key) else {
        return;
    };
    let ability_category = if ability.kind == Kind::Physical {
        state.player.weapons.first().map(|weapon| weapon.category)
    } else {
        None
    };
    state.player.mana -= effective_cost;
    let purge_targets_player = ability.effects.iter().any(|effect| matches!(effect, Effect::Purge));
    let debuffs_before_purge = if purge_targets_player {
        state.player.effects.iter().filter(|timed| !is_positive(&timed.effect)).count()
    } else {
        0
    };

    // Allies that beneficial effects land on (self, plus the pet when AoE).
    let mut allies = vec![Who::Player];
    if ability.is_aoe {
        allies.push(Who::Pet);
    }

    // Offensive effects can be dodged by the enemy; roll once for the cast.
    let has_offensive = ability.effects.iter().any(|e| !effect_targets_self(e));
    let enemy_dodged = if has_offensive {
        let mut rng = rng();
        state.enemy.can_dodge()
            && rng.random_bool(ability_dodge_chance(
                state.player.eff_initiative(),
                state.enemy.eff_initiative(),
                state.player.intelligence_mod,
            ) as f64)
    } else {
        false
    };
    if enemy_dodged {
        let dodge_word = state.dodge_word.clone();
        state.fx.push(CombatFx {
            side: FxSide::Enemy,
            text: dodge_word,
            color: Color::srgb(0.85, 0.85, 0.4),
        });
    }
    let combo_poise = if !enemy_dodged && state.enemy_tactics.is_some() {
        resolve_ability_combos(state, &ability)
    } else {
        0.0
    };

    // Route each effect to its proper target based on the effect's nature so a
    // bundled self-buff never benefits the enemy (and vice versa).
    for effect in &ability.effects {
        if effect_targets_self(effect) {
            for &ally in &allies {
                if state.get(ally).is_some() {
                    apply_effect(
                        state,
                        Who::Player,
                        ally,
                        effect,
                        ability.kind,
                        ability_category,
                        play_audio_msg,
                    );
                }
            }
        } else if !enemy_dodged && state.enemy.alive {
            apply_effect(
                state,
                Who::Player,
                Who::Enemy,
                effect,
                ability.kind,
                ability_category,
                play_audio_msg,
            );
        }
    }
    if purge_targets_player {
        reward_purge_combo(state, debuffs_before_purge);
    }
    if !enemy_dodged && state.enemy_tactics.is_some() && has_offensive {
        let control_interrupt = ability
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::Stun { .. } | Effect::Silence { .. }));
        let poise_damage = if control_interrupt {
            state.enemy_poise
        } else {
            ability_poise_damage(&ability) + combo_poise
        };
        damage_enemy_poise(state, poise_damage, play_audio_msg);
    }

    if let Some(slot_mut) = state.abilities.get_mut(index) {
        slot_mut.remaining = slot_mut.cooldown;
    }
    state.fx.push(CombatFx {
        side: FxSide::Player,
        text: "Cast!".to_string(),
        color: Color::srgb(0.5, 0.8, 1.0),
    });
    let cast_sound = if ability.kind == Kind::Holy {
        "holy"
    } else {
        "cast"
    };
    play_audio_msg.write(PlayAudioMsg::new(cast_sound));
}

/// Performs the try use consumable operation.
pub fn try_use_consumable(
    state: &mut CombatState,
    player: &mut Player,
    key: &str,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.status == CombatStatus::Over {
        return;
    }
    if !player.inventory.iter().any(|k| k == key) {
        return;
    }
    let Some(Equipment::Consumable(consumable)) = get_equipment(key) else {
        return;
    };
    if !take_combat_consumable(&mut state.player_consumables, key) {
        return;
    }

    for effect in &consumable.effects {
        // Beneficial effects buff the player; any offensive effect is thrown at
        // the enemy so a consumable never debuffs its own user.
        if effect_targets_self(effect) {
            apply_effect(
                state,
                Who::Player,
                Who::Player,
                effect,
                Kind::Physical,
                None,
                play_audio_msg,
            );
        } else if state.enemy.alive {
            apply_effect(
                state,
                Who::Player,
                Who::Enemy,
                effect,
                Kind::Physical,
                None,
                play_audio_msg,
            );
        }
    }

    // Consume one instance from the inventory.
    if let Some(pos) = player.inventory.iter().position(|k| k == key) {
        player.inventory.remove(pos);
    }
    if !player.inventory.iter().any(|k| k == key) {
        player.equipped_consumables.retain(|k| k != key);
    }

    state.fx.push(CombatFx {
        side: FxSide::Player,
        text: "Used!".to_string(),
        color: Color::srgb(0.5, 0.9, 0.6),
    });
    play_audio_msg.write(PlayAudioMsg::new("drink"));
}

/// Apply an ability cast by the networked opponent (the `Enemy` side). Used by
/// the duel host to fold a remote player's ability into the authoritative sim.
#[cfg(not(target_arch = "wasm32"))]
pub fn enemy_cast_ability(
    state: &mut CombatState,
    key: &str,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.status == CombatStatus::Over {
        return;
    }
    if !state.enemy.alive || !state.enemy.can_cast() {
        return;
    }
    let Some(ability) = get_ability(key) else {
        return;
    };
    let ability_category = if ability.kind == Kind::Physical {
        state.enemy.weapons.first().map(|weapon| weapon.category)
    } else {
        None
    };
    let Some(slot_index) =
        state.enemy_abilities.iter().position(|slot| slot.key.as_deref() == Some(key))
    else {
        return;
    };
    let effective_cost = (state.enemy_abilities[slot_index].mana_cost as f32
        * (1.0 - state.enemy.clearcasting_reduction()))
    .round();
    if state.enemy_abilities[slot_index].remaining > 0.0 || state.enemy.mana < effective_cost {
        return;
    }
    state.enemy.mana -= effective_cost;

    // The host player can dodge offensive effects.
    let has_offensive = ability.effects.iter().any(|e| !effect_targets_self(e));
    let player_dodged = if has_offensive {
        let mut rng = rng();
        state.player.can_dodge()
            && rng.random_bool(ability_dodge_chance(
                state.enemy.eff_initiative(),
                state.player.eff_initiative(),
                state.enemy.intelligence_mod,
            ) as f64)
    } else {
        false
    };
    if player_dodged {
        let dodge_word = state.dodge_word.clone();
        state.fx.push(CombatFx {
            side: FxSide::Player,
            text: dodge_word,
            color: Color::srgb(0.85, 0.85, 0.4),
        });
    }

    for effect in &ability.effects {
        if effect_targets_self(effect) {
            let allies = if ability.is_aoe {
                [Some(Who::Enemy), Some(Who::EnemyPet)]
            } else {
                [Some(Who::Enemy), None]
            };
            for ally in allies.into_iter().flatten() {
                if state.get(ally).is_some() {
                    apply_effect(
                        state,
                        Who::Enemy,
                        ally,
                        effect,
                        ability.kind,
                        ability_category,
                        play_audio_msg,
                    );
                }
            }
        } else if !player_dodged && state.player.alive {
            apply_effect(
                state,
                Who::Enemy,
                Who::Player,
                effect,
                ability.kind,
                ability_category,
                play_audio_msg,
            );
        }
    }
    let slot = &mut state.enemy_abilities[slot_index];
    if slot.cooldown <= 0.0 {
        slot.cooldown = ability.cooldown;
    }
    slot.remaining = slot.cooldown;

    state.fx.push(CombatFx {
        side: FxSide::Enemy,
        text: "Cast!".to_string(),
        color: Color::srgb(0.5, 0.8, 1.0),
    });
    let cast_sound = if ability.kind == Kind::Holy {
        "holy"
    } else {
        "cast"
    };
    play_audio_msg.write(PlayAudioMsg::new(cast_sound));
}

/// Apply a consumable used by the networked opponent (the `Enemy` side).
#[cfg(not(target_arch = "wasm32"))]
pub fn enemy_use_consumable(
    state: &mut CombatState,
    key: &str,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    if state.status == CombatStatus::Over {
        return;
    }
    let Some(Equipment::Consumable(consumable)) = get_equipment(key) else {
        return;
    };
    if !take_combat_consumable(&mut state.enemy_consumables, key) {
        return;
    }

    for effect in &consumable.effects {
        if effect_targets_self(effect) {
            apply_effect(
                state,
                Who::Enemy,
                Who::Enemy,
                effect,
                Kind::Physical,
                None,
                play_audio_msg,
            );
        } else if state.player.alive {
            apply_effect(
                state,
                Who::Enemy,
                Who::Player,
                effect,
                Kind::Physical,
                None,
                play_audio_msg,
            );
        }
    }

    state.fx.push(CombatFx {
        side: FxSide::Enemy,
        text: "Used!".to_string(),
        color: Color::srgb(0.5, 0.9, 0.6),
    });
    play_audio_msg.write(PlayAudioMsg::new("drink"));
}

/// Handles combat card click.
pub fn handle_combat_card_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    card_q: Query<&CombatCard>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    mut state: Option<ResMut<CombatState>>,
    mut player: ResMut<Player>,
    duel: Option<Res<DuelActive>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    // During a networked duel, card clicks are routed through the duel systems.
    if duel.is_some() {
        return;
    }
    let Some(state) = state.as_mut() else {
        return;
    };
    let Ok(card) = card_q.get(event.entity) else {
        return;
    };
    match card.clone() {
        CombatCard::Ability(index) => try_cast_ability(state, index, &mut play_audio_msg),
        CombatCard::Consumable(key) => {
            try_use_consumable(state, &mut player, &key, &mut play_audio_msg);
            // Using a consumable can despawn its card (stock exhausted) before the
            // hover system emits an interaction change, leaving the tooltip stuck
            // open. Clear any open tooltip so it disappears on use.
            for entity in &tooltip_q {
                commands.entity(entity).try_despawn();
            }
        },
        CombatCard::Guard => try_guard(state, &mut play_audio_msg),
        CombatCard::Stance(stance) => {
            set_combat_stance(state, stance, &mut play_audio_msg);
        },
    }
}

/// Performs the combat input operation.
pub fn combat_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut combat_speed: ResMut<CombatSpeed>,
    mut state: Option<ResMut<CombatState>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut pending_hunt_xp: ResMut<PendingHuntXp>,
    mut game_menu_origin: ResMut<GameMenuOrigin>,
    mut combat_menu_suspended: ResMut<CombatMenuSuspended>,
) {
    let Some(state) = state.as_mut() else {
        return;
    };

    // Combat speed: Ctrl+Right doubles it, Ctrl+Left halves it.
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl {
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            combat_speed.faster();
            play_audio_msg.write(PlayAudioMsg::new("click"));
        } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
            combat_speed.slower();
            play_audio_msg.write(PlayAudioMsg::new("click"));
        }
    }

    if state.status != CombatStatus::Over
        && (keyboard.just_released(KeyCode::Escape)
            || keyboard.just_released(KeyCode::Enter)
            || keyboard.just_released(KeyCode::NumpadEnter))
    {
        game_menu_origin.0 = Some(GameState::Combat);
        combat_menu_suspended.0 = true;
        next_game_state.set(GameState::GameMenu);
        return;
    }

    if state.status == CombatStatus::Over {
        if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
            pending_hunt_xp.amount = state.xp_reward();
            maybe_queue_mutation_offer(&mut commands, state.mutation_candidate);
            play_audio_msg.write(PlayAudioMsg::new("button"));
            next_game_state.set(GameState::Playing);
        }
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
    if state.paused {
        return;
    }

    for (i, key) in ABILITY_HOTKEYS.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            try_cast_ability(state, i, &mut play_audio_msg);
        }
    }
    if keyboard.just_pressed(GUARD_HOTKEY) {
        try_guard(state, &mut play_audio_msg);
    }
    for (index, hotkey) in STANCE_HOTKEYS.iter().enumerate() {
        if keyboard.just_pressed(*hotkey) {
            set_combat_stance(state, CombatStance::ALL[index], &mut play_audio_msg);
        }
    }

    let equipped: Vec<String> = consumable_card_order(&player, &state.player_consumables);
    for (i, hotkey) in CONSUMABLE_HOTKEYS.iter().enumerate() {
        if keyboard.just_pressed(*hotkey) {
            if let Some(key) = equipped.get(i) {
                let key = key.clone();
                try_use_consumable(state, &mut player, &key, &mut play_audio_msg);
            }
        }
    }
}

/// The order consumables appear on screen (mirrors combat::ui spawn order).
pub fn consumable_card_order(
    player: &Player,
    combat_stock: &HashMap<String, usize>,
) -> Vec<String> {
    let mut consumables: Vec<(String, u32, String)> = player
        .equipped_consumables
        .iter()
        .filter(|key| combat_stock.get(*key).copied().unwrap_or(0) > 0)
        .filter_map(|key| match get_equipment(key) {
            Some(Equipment::Consumable(item)) => Some((key.clone(), item.level, item.name)),
            _ => None,
        })
        .collect();
    consumables.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));
    consumables.into_iter().map(|(k, _, _)| k).take(8).collect()
}

// ---------------------------------------------------------------------------
// Visuals: smooth bars, labels, cooldown overlays, monster bar, floating text
// ---------------------------------------------------------------------------

/// Updates combat pause indicator.
pub fn update_combat_pause_indicator(
    state: Option<Res<CombatState>>,
    mut overlay_q: Query<&mut Visibility, With<crate::core::combat::ui::CombatPausedOverlay>>,
) {
    let paused = state.map(|s| s.paused && s.status != CombatStatus::Over).unwrap_or(false);
    for mut vis in &mut overlay_q {
        *vis = if paused {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Keeps the combat-speed label in sync with the current `CombatSpeed`.
pub fn update_combat_speed_label(
    combat_speed: Res<CombatSpeed>,
    mut text_q: Query<&mut Text, With<CombatSpeedText>>,
) {
    if let Some(mut text) = text_q.iter_mut().next() {
        let label = combat_speed.label();
        if text.0 != label {
            text.0 = label;
        }
    }
}

/// Refreshes cached combat-event words after the language changes mid-battle.
pub fn refresh_combat_translation_cache(
    state: Option<ResMut<CombatState>>,
    settings: Res<crate::core::settings::Settings>,
    localization: Res<crate::core::localization::Localization>,
) {
    let Some(mut state) = state else {
        return;
    };
    let language = settings.language;
    state.dodge_word = localization.get("general.dodge", language);
    state.miss_word = localization.get("general.miss", language);
    state.xp_word = localization.get("general.xp", language);
    state.guard_word = localization.get("combat.guard", language);
    state.parry_word = localization.get("combat.parry", language);
    state.break_word = localization.get("combat.break", language);
    state.shatter_word = localization.get("combat.combo_shatter", language);
    state.detonate_word = localization.get("combat.combo_detonate", language);
    state.doom_word = localization.get("combat.combo_doom", language);
    state.exploit_word = localization.get("combat.combo_exploit", language);
    state.cleanse_word = localization.get("combat.combo_cleanse", language);
    state.phase_word = localization.get("combat.phase_two", language);
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct CombatTranslationParams<'w, 's> {
    pub name_q: Query<
        'w,
        's,
        (&'static mut Text, &'static CombatPortraitName),
        (
            Without<crate::core::ui::playing::StatLabel>,
            Without<crate::core::combat::ui::CombatMonsterHealthText>,
            Without<CombatEndButtonText>,
        ),
    >,
    pub level_q: Query<
        'w,
        's,
        (&'static mut Text, &'static CombatPortraitLevel),
        (
            Without<crate::core::ui::playing::StatLabel>,
            Without<crate::core::combat::ui::CombatMonsterHealthText>,
            Without<CombatEndButtonText>,
            Without<CombatPortraitName>,
        ),
    >,
    pub stat_label_q: Query<
        'w,
        's,
        (&'static mut Text, &'static CombatStatLabel),
        (
            Without<crate::core::ui::playing::StatLabel>,
            Without<crate::core::combat::ui::CombatMonsterHealthText>,
            Without<CombatEndButtonText>,
            Without<CombatPortraitName>,
            Without<CombatPortraitLevel>,
        ),
    >,
    pub pet_name_q: Query<
        'w,
        's,
        &'static mut Text,
        (
            With<CombatPetName>,
            Without<crate::core::ui::playing::StatLabel>,
            Without<crate::core::combat::ui::CombatMonsterHealthText>,
            Without<CombatEndButtonText>,
            Without<CombatPortraitName>,
            Without<CombatPortraitLevel>,
            Without<CombatStatLabel>,
        ),
    >,
    pub enemy_mana_label_q: Query<
        'w,
        's,
        &'static mut Text,
        (
            With<crate::core::combat::ui::CombatEnemyManaText>,
            Without<crate::core::ui::playing::StatLabel>,
            Without<crate::core::combat::ui::CombatMonsterHealthText>,
            Without<CombatEndButtonText>,
            Without<CombatPortraitName>,
            Without<CombatPortraitLevel>,
            Without<CombatStatLabel>,
            Without<CombatPetName>,
        ),
    >,
    pub cooldown_text_q: Query<
        'w,
        's,
        (&'static AbilityCooldownText, &'static mut Text, &'static mut Visibility),
        (
            Without<crate::core::ui::playing::StatLabel>,
            Without<crate::core::combat::ui::CombatMonsterHealthText>,
            Without<CombatEndButtonText>,
            Without<CombatPortraitName>,
            Without<CombatPortraitLevel>,
            Without<CombatStatLabel>,
            Without<CombatPetName>,
            Without<crate::core::combat::ui::CombatEnemyManaText>,
        ),
    >,
}

/// Localizes monster name.
pub fn localize_monster_name(
    name: &str,
    kind: crate::core::monsters::MonsterKind,
    localization: &crate::core::localization::Localization,
    lang: crate::core::settings::Language,
) -> String {
    if let Some(localized_name) = localization.monster_name(name, lang) {
        return localized_name;
    }

    if kind == crate::core::monsters::MonsterKind::Dragon {
        let name_cap = crate::utils::capitalize_words(name);
        let mut parts = name_cap.split_whitespace();
        if let Some(color) = parts.next() {
            let mut stage_parts = parts.collect::<Vec<_>>();
            if stage_parts.first().is_some_and(|part| part.eq_ignore_ascii_case("dragon")) {
                stage_parts.remove(0);
            }
            let stage = stage_parts.join(" ");
            let color_key = format!("general.{}", color.to_lowercase());
            let color_loc = localization.get_opt(&color_key, lang).unwrap_or_else(|| {
                localization.get_opt(color, lang).unwrap_or_else(|| color.to_string())
            });
            let dragon_loc = localization
                .get_opt("general.dragon", lang)
                .unwrap_or_else(|| "Dragon".to_string());
            if stage.is_empty() {
                return format!("{} {}", color_loc, dragon_loc);
            } else {
                let stage_loc =
                    localization.get_opt(&stage, lang).unwrap_or_else(|| stage.to_string());
                return format!("{} {} ({})", color_loc, dragon_loc, stage_loc);
            }
        }
    }

    crate::utils::capitalize_words(name)
}

/// Updates combat visuals.
pub fn update_combat_visuals(
    time: Res<Time>,
    mut commands: Commands,
    state: Option<ResMut<CombatState>>,
    player: Res<Player>,
    active_monster: Option<Res<ActiveMonster>>,
    settings: Res<crate::core::settings::Settings>,
    localization: Res<crate::core::localization::Localization>,
    assets: Res<crate::core::assets::WorldAssets>,
    mut bar_q: ParamSet<(
        Query<&mut Node, With<crate::core::ui::playing::HealthBarFill>>,
        Query<&mut Node, With<crate::core::ui::playing::ManaBarFill>>,
        Query<&mut Node, With<crate::core::ui::playing::PetHealthBarFill>>,
        Query<&mut Node, With<crate::core::combat::ui::CombatMonsterHealthFill>>,
        Query<&mut Node, With<crate::core::combat::ui::CombatEnemyManaFill>>,
    )>,
    mut overlay_q: Query<
        (&AbilityCooldownOverlay, &mut Node),
        (
            Without<crate::core::ui::playing::HealthBarFill>,
            Without<crate::core::ui::playing::ManaBarFill>,
            Without<crate::core::ui::playing::PetHealthBarFill>,
            Without<crate::core::combat::ui::CombatMonsterHealthFill>,
            Without<crate::core::combat::ui::CombatEnemyManaFill>,
        ),
    >,
    mut ability_image_q: Query<
        (&crate::core::combat::ui::AbilityCardImage, &mut ImageNode),
        (
            Without<crate::core::ui::playing::HealthBarFill>,
            Without<crate::core::ui::playing::ManaBarFill>,
            Without<crate::core::ui::playing::PetHealthBarFill>,
            Without<crate::core::combat::ui::CombatMonsterHealthFill>,
            Without<crate::core::combat::ui::CombatEnemyManaFill>,
        ),
    >,
    mut label_q: Query<(&mut Text, &crate::core::ui::playing::StatLabel)>,
    mut monster_label_q: Query<
        &mut Text,
        (
            With<crate::core::combat::ui::CombatMonsterHealthText>,
            Without<crate::core::ui::playing::StatLabel>,
        ),
    >,
    mut end_btn_text_q: Query<
        &mut Text,
        (
            With<CombatEndButtonText>,
            Without<crate::core::ui::playing::StatLabel>,
            Without<crate::core::combat::ui::CombatMonsterHealthText>,
        ),
    >,
    player_portrait_q: Query<Entity, With<crate::core::combat::ui::CombatPlayerPortrait>>,
    mut translation_params: CombatTranslationParams,
) {
    let Some(mut state) = state else {
        return;
    };
    let dt = time.delta_secs();
    let lang = settings.language;

    for (mut text, name_comp) in &mut translation_params.name_q {
        let name_str = if name_comp.is_player {
            crate::utils::capitalize_words(&player.name)
        } else if let Some(ref am) = active_monster {
            localize_monster_name(&am.monster.name, am.monster.kind, &localization, lang)
        } else {
            "Enemy".to_string()
        };
        if text.0 != name_str {
            text.0 = name_str;
        }
    }

    let level_word = localization.get("general.level", lang);
    for (mut text, level_comp) in &mut translation_params.level_q {
        let lvl = if level_comp.is_player {
            state.player_level
        } else {
            state.enemy_level
        };
        let level_str = format!("{} {}", level_word, lvl);
        if text.0 != level_str {
            text.0 = level_str;
        }
    }

    if let Some(ref pet) = player.pet {
        if let Ok(mut text) = translation_params.pet_name_q.single_mut() {
            let pet_name = localization
                .get_opt(&pet.name, lang)
                .unwrap_or_else(|| crate::utils::capitalize_words(&pet.name));
            if text.0 != pet_name {
                text.0 = pet_name;
            }
        }
    }

    for (mut text, stat_lbl) in &mut translation_params.stat_label_q {
        let label_str = localization.get(&stat_lbl.title_key, lang);
        if text.0 != label_str {
            text.0 = label_str;
        }
    }

    let t = (BAR_LERP_SPEED * dt).clamp(0.0, 1.0);

    // Smoothly interpolate displayed values toward the true values.
    state.player.display_health += (state.player.health - state.player.display_health) * t;
    state.player.display_mana += (state.player.mana - state.player.display_mana) * t;
    state.enemy.display_health += (state.enemy.health - state.enemy.display_health) * t;
    if let Some(pet) = state.pet.as_mut() {
        pet.display_health += (pet.health - pet.display_health) * t;
    }

    let p_hp_ratio =
        (state.player.display_health / state.player.max_health).clamp(0.0, 1.0) * 100.0;
    let p_mp_ratio = if state.player.max_mana > 0.0 {
        (state.player.display_mana / state.player.max_mana).clamp(0.0, 1.0) * 100.0
    } else {
        0.0
    };
    let e_hp_ratio = (state.enemy.display_health / state.enemy.max_health).clamp(0.0, 1.0) * 100.0;
    let e_mp_ratio = if state.enemy.max_mana > 0.0 {
        (state.enemy.display_mana / state.enemy.max_mana).clamp(0.0, 1.0) * 100.0
    } else {
        0.0
    };

    if let Ok(mut node) = bar_q.p0().single_mut() {
        node.width = Val::Percent(p_hp_ratio);
    }
    if let Ok(mut node) = bar_q.p1().single_mut() {
        node.width = Val::Percent(p_mp_ratio);
    }
    if let Ok(mut node) = bar_q.p3().single_mut() {
        node.width = Val::Percent(e_hp_ratio);
    }
    if let Ok(mut node) = bar_q.p4().single_mut() {
        node.width = Val::Percent(e_mp_ratio);
    }
    if let Some(pet) = state.pet.as_ref() {
        if let Ok(mut node) = bar_q.p2().single_mut() {
            let ratio = (pet.display_health / pet.max_health).clamp(0.0, 1.0) * 100.0;
            node.width = Val::Percent(ratio);
        }
    }

    // Text labels.
    let health_word = localization.get("general.health", lang);
    let mana_word = localization.get("general.mana", lang);
    for (mut text, label) in &mut label_q {
        use crate::core::ui::playing::PlayingStat::*;
        match label.0 {
            Health => {
                text.0 = format!(
                    "{} / {} (+{}) {}",
                    state.player.health.round() as i32,
                    state.player.max_health.round() as i32,
                    player.health_regen(),
                    health_word
                )
            },
            Mana => {
                text.0 = format!(
                    "{} / {} (+{}) {}",
                    state.player.mana.round() as i32,
                    state.player.max_mana.round() as i32,
                    player.mana_regen(),
                    mana_word
                )
            },
            PetHealth => {
                if let Some(pet) = state.pet.as_ref() {
                    text.0 = format!(
                        "{} / {} {}",
                        pet.health.round().max(0.0) as i32,
                        pet.max_health.round() as i32,
                        health_word
                    );
                }
            },
            _ => {},
        }
    }
    if let Ok(mut text) = monster_label_q.single_mut() {
        text.0 = format!(
            "{} / {} (+{}) {}",
            state.enemy.health.round().max(0.0) as i32,
            state.enemy.max_health.round() as i32,
            active_monster.map(|am| am.monster.health_regen).unwrap_or(0),
            health_word
        );
    }
    if let Ok(mut text) = translation_params.enemy_mana_label_q.single_mut() {
        text.0 = format!(
            "{} / {} (+{}) {}",
            state.enemy.mana.round().max(0.0) as i32,
            state.enemy.max_mana.round() as i32,
            state.enemy.mana_regen.round() as i32,
            mana_word
        );
    }

    // Ability cooldown / disabled overlays.
    for (overlay, mut node) in &mut overlay_q {
        let slots = if overlay.is_player {
            &state.abilities
        } else {
            &state.enemy_abilities
        };
        let frac = slots
            .get(overlay.slot)
            .map(|slot| {
                if slot.key.is_none() {
                    0.0
                } else if slot.cooldown > 0.0 && slot.remaining > 0.0 {
                    (slot.remaining / slot.cooldown).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        node.height = Val::Percent(frac * 100.0);
    }

    // Cooldown text overlays.
    for (cooldown_text, mut text, mut vis) in &mut translation_params.cooldown_text_q {
        let slots = if cooldown_text.is_player {
            &state.abilities
        } else {
            &state.enemy_abilities
        };
        let remaining = slots.get(cooldown_text.slot).map(|slot| slot.remaining).unwrap_or(0.0);
        if remaining > 0.0 {
            text.0 = format!("{:.1}s", remaining);
            *vis = Visibility::Visible;
        } else {
            text.0 = "".to_string();
            *vis = Visibility::Hidden;
        }
    }
    for (ability, mut image) in &mut ability_image_q {
        let (slots, mana) = if ability.is_player {
            (&state.abilities, state.player.mana)
        } else {
            (&state.enemy_abilities, state.enemy.mana)
        };
        let out_of_resources = slots
            .get(ability.slot)
            .map(|slot| slot.key.is_some() && slot.remaining <= 0.0 && mana < slot.mana_cost as f32)
            .unwrap_or(false);
        image.color = if out_of_resources {
            Color::srgba(0.45, 0.45, 0.45, 1.0)
        } else {
            Color::WHITE
        };
    }

    // End-of-combat button label.
    if let Ok(mut text) = end_btn_text_q.single_mut() {
        let label = if state.status == CombatStatus::Over {
            localization.get("general.continue", lang)
        } else {
            localization.get("general.forfeit_combat", lang)
        };
        if text.0 != label {
            text.0 = label;
        }
    }

    // Spawn floating combat text for queued events.
    let player_portrait = player_portrait_q.single().ok();
    let fx: Vec<CombatFx> = state.fx.drain(..).collect();
    for f in fx {
        spawn_floating_text(&mut commands, &assets, &f, player_portrait);
    }
}

/// Synchronizes Guard, stances, Poise, and telegraphed enemy intents.
#[allow(clippy::too_many_arguments)]
pub fn update_combat_tactics_visuals(
    state: Option<Res<CombatState>>,
    assets: Res<crate::core::assets::WorldAssets>,
    localization: Res<crate::core::localization::Localization>,
    settings: Res<crate::core::settings::Settings>,
    mut poise_fill_q: Query<
        &mut Node,
        (
            With<crate::core::combat::ui::CombatPoiseFill>,
            Without<CombatCard>,
            Without<crate::core::combat::ui::CombatPoiseBreakFill>,
            Without<crate::core::combat::ui::CombatEnemyCastFill>,
            Without<crate::core::combat::ui::CombatEnemyRecoveryFill>,
        ),
    >,
    mut poise_break_fill_q: Query<
        &mut Node,
        (
            With<crate::core::combat::ui::CombatPoiseBreakFill>,
            Without<CombatCard>,
            Without<crate::core::combat::ui::CombatPoiseFill>,
            Without<crate::core::combat::ui::CombatEnemyCastFill>,
            Without<crate::core::combat::ui::CombatEnemyRecoveryFill>,
        ),
    >,
    mut poise_text_q: Query<
        &mut Text,
        (
            With<crate::core::combat::ui::CombatPoiseText>,
            Without<crate::core::combat::ui::CombatEnemyIntentName>,
            Without<crate::core::combat::ui::CombatEnemyIntentDescription>,
            Without<crate::core::combat::ui::CombatEnemyCastText>,
        ),
    >,
    mut tactic_card_q: Query<
        (&CombatCard, &mut Node, &mut BorderColor, &mut BackgroundColor, &mut ImageNode),
        (
            Without<crate::core::combat::ui::AbilityCardImage>,
            Without<crate::core::combat::ui::CombatEnemyIntentImage>,
        ),
    >,
    mut intent_image_q: Query<
        &mut ImageNode,
        (
            With<crate::core::combat::ui::CombatEnemyIntentImage>,
            Without<crate::core::combat::ui::AbilityCardImage>,
            Without<CombatCard>,
        ),
    >,
    mut intent_name_q: Query<
        &mut Text,
        (
            With<crate::core::combat::ui::CombatEnemyIntentName>,
            Without<crate::core::combat::ui::CombatPoiseText>,
            Without<crate::core::combat::ui::CombatEnemyIntentDescription>,
            Without<crate::core::combat::ui::CombatEnemyCastText>,
        ),
    >,
    mut intent_desc_q: Query<
        &mut Text,
        (
            With<crate::core::combat::ui::CombatEnemyIntentDescription>,
            Without<crate::core::combat::ui::CombatEnemyIntentName>,
            Without<crate::core::combat::ui::CombatPoiseText>,
            Without<crate::core::combat::ui::CombatEnemyCastText>,
        ),
    >,
    mut cast_fill_q: Query<
        &mut Node,
        (
            With<crate::core::combat::ui::CombatEnemyCastFill>,
            Without<CombatCard>,
            Without<crate::core::combat::ui::CombatPoiseFill>,
            Without<crate::core::combat::ui::CombatPoiseBreakFill>,
            Without<crate::core::combat::ui::CombatEnemyRecoveryFill>,
        ),
    >,
    mut recovery_fill_q: Query<
        &mut Node,
        (
            With<crate::core::combat::ui::CombatEnemyRecoveryFill>,
            Without<CombatCard>,
            Without<crate::core::combat::ui::CombatPoiseFill>,
            Without<crate::core::combat::ui::CombatPoiseBreakFill>,
            Without<crate::core::combat::ui::CombatEnemyCastFill>,
        ),
    >,
    mut cast_text_q: Query<
        &mut Text,
        (
            With<crate::core::combat::ui::CombatEnemyCastText>,
            Without<crate::core::combat::ui::CombatEnemyIntentName>,
            Without<crate::core::combat::ui::CombatEnemyIntentDescription>,
            Without<crate::core::combat::ui::CombatPoiseText>,
        ),
    >,
) {
    let Some(state) = state else {
        return;
    };
    let language = settings.language;
    if let Ok(mut fill) = poise_fill_q.single_mut() {
        fill.width =
            Val::Percent((state.enemy_poise / state.enemy_max_poise).clamp(0.0, 1.0) * 100.0);
    }
    if let Ok(mut fill) = poise_break_fill_q.single_mut() {
        fill.width = Val::Percent(
            (state.enemy_break_remaining / BREAK_STUN_DURATION).clamp(0.0, 1.0) * 100.0,
        );
    }
    if let Ok(mut text) = poise_text_q.single_mut() {
        text.0 = if state.enemy_break_remaining > 0.0 {
            format!(
                "{} - {:.1}s",
                localization.get("combat.broken", language),
                state.enemy_break_remaining
            )
        } else {
            format!(
                "{:.0} / {:.0} {}",
                state.enemy_poise,
                state.enemy_max_poise,
                localization.get("combat.poise", language)
            )
        };
    }
    for (card, mut node, mut border, mut background, mut image) in &mut tactic_card_q {
        match card {
            CombatCard::Guard => {
                let affordable = state.player.mana >= guard_mana_cost(state.player_level);
                image.color = if affordable {
                    Color::WHITE
                } else {
                    Color::srgb(0.42, 0.42, 0.46)
                };
                node.border = UiRect::all(Val::Px(if state.guard_remaining > 0.0 {
                    4.0
                } else {
                    2.0
                }));
                *border = BorderColor::all(if state.guard_remaining > 0.0 {
                    Color::srgb(1.0, 0.82, 0.30)
                } else {
                    crate::core::constants::BUTTON_BORDER_COLOR
                });
                background.0 = if state.guard_remaining > 0.0 {
                    Color::srgba(0.34, 0.21, 0.04, 0.96)
                } else if !affordable {
                    Color::srgba(0.03, 0.03, 0.04, 0.82)
                } else {
                    Color::srgba(0.05, 0.05, 0.08, 0.72)
                };
            },
            CombatCard::Stance(stance) => {
                let selected = *stance == state.stance;
                node.border = UiRect::all(Val::Px(if selected {
                    4.0
                } else {
                    2.0
                }));
                image.color = if selected {
                    Color::WHITE
                } else {
                    Color::srgb(0.50, 0.50, 0.54)
                };
                *border = BorderColor::all(if selected {
                    Color::srgb(1.0, 0.72, 0.22)
                } else {
                    crate::core::constants::BUTTON_BORDER_COLOR
                });
                background.0 = Color::srgba(0.05, 0.05, 0.08, 0.72);
            },
            CombatCard::Ability(_) | CombatCard::Consumable(_) => {},
        }
    }

    let Some(tactics) = state.enemy_tactics.as_ref() else {
        return;
    };
    let preview = tactics.active_cast.map(|cast| cast.movement).or_else(|| {
        (!tactics.rotation.is_empty())
            .then(|| tactics.rotation[tactics.next_index % tactics.rotation.len()])
    });
    let Some(movement) = preview else {
        return;
    };
    if let Ok(mut image) = intent_image_q.single_mut() {
        image.image = assets.image(movement.kind.image_key());
    }
    if let Ok(mut text) = intent_name_q.single_mut() {
        text.0 = if tactics.active_cast.is_some() {
            localization.get(movement.kind.name_key(), language)
        } else {
            format!(
                "{}: {}",
                localization.get("combat.next", language),
                localization.get(movement.kind.name_key(), language)
            )
        };
    }
    if let Ok(mut text) = intent_desc_q.single_mut() {
        let target_key = match movement.target {
            EnemyMoveTarget::Player => "combat.target_player",
            EnemyMoveTarget::Pet if state.pet.as_ref().is_some_and(|pet| pet.alive) => {
                "combat.target_pet"
            },
            EnemyMoveTarget::Pet => "combat.target_player",
            EnemyMoveTarget::SelfSide => "combat.target_self",
        };
        text.0 = format!(
            "{}\n{}: {}",
            localization.get(movement.kind.description_key(), language),
            localization.get("general.target", language),
            localization.get(target_key, language)
        );
    }
    let (cast_progress, recovery_progress, cast_text) =
        if let Some(active_cast) = tactics.active_cast {
            let progress = (active_cast.elapsed / active_cast.movement.cast_time).clamp(0.0, 1.0);
            let remaining = (active_cast.movement.cast_time - active_cast.elapsed).max(0.0);
            (progress, 0.0, format!("{remaining:.1}s"))
        } else {
            let progress = if tactics.recovery_max > 0.0 {
                (tactics.recovery / tactics.recovery_max).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (0.0, progress, format!("{:.1}s", tactics.recovery.max(0.0)))
        };
    if let Ok(mut fill) = cast_fill_q.single_mut() {
        fill.width = Val::Percent(cast_progress * 100.0);
    }
    if let Ok(mut fill) = recovery_fill_q.single_mut() {
        fill.width = Val::Percent(recovery_progress * 100.0);
    }
    if let Ok(mut text) = cast_text_q.single_mut() {
        text.0 = cast_text;
    }
}

/// Spawns floating text.
fn spawn_floating_text(
    commands: &mut Commands,
    assets: &crate::core::assets::WorldAssets,
    fx: &CombatFx,
    player_portrait: Option<Entity>,
) {
    let mut rng = rng();
    let horizontal = match fx.side {
        FxSide::Player => rng.random_range(9.2..27.6),
        FxSide::Enemy => rng.random_range(72.4..90.8),
    };
    let start_top = rng.random_range(12.0..32.0);
    let is_xp_reward = fx.text.starts_with('+') && fx.text.contains(' ');

    // The XP reward is shown large, centered over the player portrait and fades
    // out slowly, rather than as a small drifting hit number.
    if is_xp_reward {
        if let Some(portrait) = player_portrait {
            commands.entity(portrait).with_children(|parent| {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(0.),
                        right: Val::Percent(0.),
                        top: Val::Percent(42.),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    GlobalZIndex(1200),
                    Pickable::IGNORE,
                    FloatingCombatText {
                        timer: 0.0,
                        start_top: 42.0,
                        life: XP_REWARD_TEXT_LIFE,
                        centered: true,
                    },
                    crate::core::combat::ui::CombatCmp,
                    crate::core::menu::utils::add_text(
                        fx.text.clone(),
                        "bold",
                        XP_REWARD_TEXT_SIZE,
                        assets,
                    ),
                    TextLayout::justify(Justify::Center),
                    TextColor(fx.color),
                ));
            });
            return;
        }
    }

    let font_size = if is_xp_reward {
        XP_REWARD_TEXT_SIZE
    } else {
        HIT_TEXT_SIZE
    };
    let life = if is_xp_reward {
        XP_REWARD_TEXT_LIFE
    } else {
        1.1
    };
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(horizontal),
            top: Val::Percent(start_top),
            ..default()
        },
        GlobalZIndex(1200),
        Pickable::IGNORE,
        FloatingCombatText {
            timer: 0.0,
            start_top,
            life,
            centered: false,
        },
        crate::core::combat::ui::CombatCmp,
        crate::core::menu::utils::add_text(fx.text.clone(), "bold", font_size, assets),
        TextColor(fx.color),
    ));
}

/// Performs the animate death skulls operation.
pub fn animate_death_skulls(
    time: Res<Time>,
    combat_speed: Res<CombatSpeed>,
    mut commands: Commands,
    state: Option<Res<CombatState>>,
    assets: Res<crate::core::assets::WorldAssets>,
    pending_hunt_pet: Option<Res<PendingHuntPet>>,
    player_portrait_q: Query<Entity, With<crate::core::combat::ui::CombatPlayerPortrait>>,
    enemy_portrait_q: Query<Entity, With<crate::core::combat::ui::CombatEnemyPortrait>>,
    pet_portrait_q: Query<Entity, With<crate::core::combat::ui::CombatPetPortrait>>,
    mut skull_q: Query<(&mut DeathSkullOverlay, &mut Node, &mut ImageNode)>,
) {
    let Some(state) = state else {
        return;
    };
    let dt = time.delta_secs() * combat_speed.0;

    let mut player_skull_exists = false;
    let mut enemy_skull_exists = false;
    let mut pet_skull_exists = false;
    for (mut skull, mut node, mut image) in &mut skull_q {
        match skull.side {
            DeathSkullSide::Player => player_skull_exists = true,
            DeathSkullSide::Enemy => enemy_skull_exists = true,
            DeathSkullSide::Pet => pet_skull_exists = true,
        }
        skull.timer = (skull.timer + dt).min(DEATH_SKULL_ANIM_DURATION);
        let frac = (skull.timer / DEATH_SKULL_ANIM_DURATION).clamp(0.0, 1.0);
        let size = DEATH_SKULL_START_SIZE + (DEATH_SKULL_END_SIZE - DEATH_SKULL_START_SIZE) * frac;
        node.width = Val::Percent(size);
        node.height = Val::Percent(size);
        node.left = Val::Percent(50.0 - size / 2.0);
        node.top = Val::Percent(50.0 - size / 2.0);
        image.color = Color::srgba(1.0, 1.0, 1.0, 0.15 + 0.45 * frac);
    }

    if !state.player.alive && !player_skull_exists {
        if let Ok(entity) = player_portrait_q.single() {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0 - DEATH_SKULL_START_SIZE / 2.0),
                        top: Val::Percent(50.0 - DEATH_SKULL_START_SIZE / 2.0),
                        width: Val::Percent(DEATH_SKULL_START_SIZE),
                        height: Val::Percent(DEATH_SKULL_START_SIZE),
                        ..default()
                    },
                    ImageNode {
                        image: assets.image("skull"),
                        image_mode: NodeImageMode::Stretch,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.15),
                        ..default()
                    },
                    Pickable::IGNORE,
                    DeathSkullOverlay {
                        side: DeathSkullSide::Player,
                        timer: 0.0,
                    },
                ));
            });
        }
    }
    if !state.enemy.alive && !enemy_skull_exists {
        // When the defeated enemy can be captured as a pet, show the capture
        // image over it instead of the death skull.
        let capture_available =
            pending_hunt_pet.as_ref().map(|pending| pending.offer_available).unwrap_or(false);
        let overlay_image = if capture_available {
            "capture"
        } else {
            "skull"
        };
        if let Ok(entity) = enemy_portrait_q.single() {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0 - DEATH_SKULL_START_SIZE / 2.0),
                        top: Val::Percent(50.0 - DEATH_SKULL_START_SIZE / 2.0),
                        width: Val::Percent(DEATH_SKULL_START_SIZE),
                        height: Val::Percent(DEATH_SKULL_START_SIZE),
                        ..default()
                    },
                    ImageNode {
                        image: assets.image(overlay_image),
                        image_mode: NodeImageMode::Stretch,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.15),
                        ..default()
                    },
                    Pickable::IGNORE,
                    DeathSkullOverlay {
                        side: DeathSkullSide::Enemy,
                        timer: 0.0,
                    },
                ));
            });
        }
    }
    // A dead pet keeps its portrait but gains a death skull; combat continues.
    if state.pet.as_ref().map(|pet| !pet.alive).unwrap_or(false) && !pet_skull_exists {
        if let Ok(entity) = pet_portrait_q.single() {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0 - DEATH_SKULL_START_SIZE / 2.0),
                        top: Val::Percent(50.0 - DEATH_SKULL_START_SIZE / 2.0),
                        width: Val::Percent(DEATH_SKULL_START_SIZE),
                        height: Val::Percent(DEATH_SKULL_START_SIZE),
                        ..default()
                    },
                    ImageNode {
                        image: assets.image("skull"),
                        image_mode: NodeImageMode::Stretch,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.15),
                        ..default()
                    },
                    Pickable::IGNORE,
                    DeathSkullOverlay {
                        side: DeathSkullSide::Pet,
                        timer: 0.0,
                    },
                ));
            });
        }
    }
}

/// Performs the animate floating text operation.
pub fn animate_floating_text(
    time: Res<Time>,
    combat_speed: Res<CombatSpeed>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut FloatingCombatText, &mut Node, &mut TextColor)>,
) {
    let dt = time.delta_secs() * combat_speed.0;
    for (entity, mut fct, mut node, mut color) in &mut q {
        fct.timer += dt;
        let frac = (fct.timer / fct.life).clamp(0.0, 1.0);
        // Centered XP text barely drifts so it stays over the portrait; hit
        // numbers float upward more noticeably.
        let drift = if fct.centered {
            4.0
        } else {
            10.0
        };
        node.top = Val::Percent(fct.start_top - frac * drift);
        let alpha = (1.0 - frac).clamp(0.0, 1.0);
        color.0 = color.0.with_alpha(alpha);
        if fct.timer >= fct.life {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawns consumable cards whose stock is exhausted.
pub fn sync_consumable_cards(
    mut commands: Commands,
    state: Option<Res<CombatState>>,
    q: Query<(Entity, &ConsumableCardRoot)>,
    mut count_q: Query<(&ConsumableCardCount, &mut Text)>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
) {
    let Some(state) = state else {
        return;
    };
    if !state.is_changed() {
        return;
    }

    for (card, mut text) in &mut count_q {
        let count = state.player_consumables.get(&card.key).copied().unwrap_or(0);
        **text = count.to_string();
    }

    let mut despawned_any = false;
    for (entity, card) in &q {
        if !card.is_player {
            continue;
        }
        let available = state.player_consumables.get(&card.key).copied().unwrap_or(0) > 0;
        if !available {
            commands.entity(entity).despawn();
            despawned_any = true;
        }
    }
    // A despawned card can leave its hover tooltip stuck open; clear it.
    if despawned_any {
        for entity in &tooltip_q {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Handles a click on the bottom combat button (forfeit while ongoing,
/// continue once combat is over).
pub fn handle_combat_end_button_click(
    _event: On<Pointer<Click>>,
    mut commands: Commands,
    state: Option<Res<CombatState>>,
    mut player: ResMut<Player>,
    duel: Option<Res<DuelActive>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut pending_hunt_xp: ResMut<PendingHuntXp>,
) {
    if let Some(ref s) = state {
        if s.status == CombatStatus::Over {
            if duel.is_none() {
                pending_hunt_xp.amount = s.xp_reward();
                maybe_queue_mutation_offer(&mut commands, s.mutation_candidate);
            }
            // A lost combat leaves the player severely injured: show the defeat
            // screen overlay on top of the playing page.
            if !s.player_won {
                commands.insert_resource(crate::core::ui::defeat::DefeatContext {
                    was_pvp: duel.is_some(),
                });
                // Clear any pending hunt / quest rewards/XP on loss so no XP is gained!
                // "dying means no xp gain at all."
                commands.insert_resource(crate::core::actions::hunt::PendingHuntXp::default());
                commands.insert_resource(crate::core::actions::hunt::PendingHuntLoot::default());
                commands.insert_resource(crate::core::actions::quest::PendingQuestXp::default());
                commands
                    .insert_resource(crate::core::actions::quest::PendingQuestRewards::default());

                play_audio_msg.write(PlayAudioMsg::new("button"));
                next_game_state.set(GameState::Playing);
                return;
            }
        } else {
            // Forfeiting active combat counts as a loss: reduce health to zero and transition
            player.set_health(0);
            if duel.is_none() {
                maybe_queue_mutation_offer(&mut commands, s.mutation_candidate);
            }
            commands.insert_resource(crate::core::ui::defeat::DefeatContext {
                was_pvp: duel.is_some(),
            });
            commands.insert_resource(crate::core::actions::hunt::PendingHuntXp::default());
            commands.insert_resource(crate::core::actions::hunt::PendingHuntLoot::default());
            commands.insert_resource(crate::core::actions::quest::PendingQuestXp::default());
            commands.insert_resource(crate::core::actions::quest::PendingQuestRewards::default());

            play_audio_msg.write(PlayAudioMsg::new("button"));
            next_game_state.set(GameState::Playing);
            return;
        }
    } else if duel.is_some() {
        return;
    }

    play_audio_msg.write(PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}

/// Handles continue with pet button click.
pub fn handle_continue_with_pet_button_click(
    _event: On<Pointer<Click>>,
    mut commands: Commands,
    state: Option<Res<CombatState>>,
    mut player: ResMut<Player>,
    pending_hunt_pet: Option<Res<PendingHuntPet>>,
    mut pending_hunt_xp: ResMut<PendingHuntXp>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let Some(pending_hunt_pet) = pending_hunt_pet else {
        return;
    };
    let Some(state) = state else {
        return;
    };
    if state.status != CombatStatus::Over || !state.player_won || !pending_hunt_pet.offer_available
    {
        return;
    }

    pending_hunt_xp.amount = state.xp_reward();
    maybe_queue_mutation_offer(&mut commands, state.mutation_candidate);
    player.set_pet(pending_hunt_pet.monster.clone());
    commands.remove_resource::<PendingHuntPet>();
    play_audio_msg.write(PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}

/// Queues the post-combat mutation choice for every eligible encounter.
fn maybe_queue_mutation_offer(commands: &mut Commands, candidate: Option<Mutation>) {
    if let Some(candidate) = candidate {
        commands.insert_resource(crate::core::ui::mutation::PendingMutationOffer(candidate));
    }
}

/// Performs the cleanup combat on exit operation.
pub fn cleanup_combat_on_exit(
    mut commands: Commands,
    combat_q: Query<Entity, With<CombatCmp>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    combat_menu_suspended: Res<CombatMenuSuspended>,
) {
    if combat_menu_suspended.0 {
        return;
    }
    for entity in &combat_q {
        commands.entity(entity).try_despawn();
    }
    for entity in &tooltip_q {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<CombatState>();
    commands.remove_resource::<PendingHuntPet>();
}

/// Performs the cleanup any combat artifacts operation.
pub fn cleanup_any_combat_artifacts(
    mut commands: Commands,
    combat_q: Query<Entity, With<CombatCmp>>,
    mut combat_menu_suspended: ResMut<CombatMenuSuspended>,
) {
    for entity in &combat_q {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<CombatState>();
    commands.remove_resource::<PendingHuntPet>();
    combat_menu_suspended.0 = false;
}

/// Synchronizes combat continue with pet button.
pub fn sync_combat_continue_with_pet_button(
    mut commands: Commands,
    state: Option<Res<CombatState>>,
    pending_hunt_pet: Option<Res<PendingHuntPet>>,
    assets: Res<crate::core::assets::WorldAssets>,
    localization: Res<crate::core::localization::Localization>,
    settings: Res<crate::core::settings::Settings>,
    mut slot_q: Query<(Entity, &mut Node), With<CombatContinueWithPetSlot>>,
    pet_btn_q: Query<Entity, With<CombatContinueWithPetButton>>,
    children_q: Query<&Children>,
) {
    let Some(state) = state else {
        return;
    };
    let should_show_pet_button = state.status == CombatStatus::Over
        && pending_hunt_pet.as_ref().map(|pending| pending.offer_available).unwrap_or(false);
    let Some((slot_entity, mut slot_node)) = slot_q.iter_mut().next() else {
        return;
    };

    let pet_button_exists = pet_btn_q.iter().next().is_some();
    if should_show_pet_button {
        slot_node.display = Display::Flex;
        if !pet_button_exists {
            spawn_continue_with_pet_button(
                &mut commands,
                slot_entity,
                &assets,
                &localization,
                settings.language,
            );
        }
    } else {
        slot_node.display = Display::None;
        if pet_button_exists {
            despawn_descendants_manual(&mut commands, slot_entity, &children_q);
        }
    }
}

/// Spawns continue with pet button.
fn spawn_continue_with_pet_button(
    commands: &mut Commands,
    parent: Entity,
    assets: &crate::core::assets::WorldAssets,
    localization: &crate::core::localization::Localization,
    lang: crate::core::settings::Language,
) {
    commands.entity(parent).with_children(|parent| {
        parent
            .spawn((
                Node {
                    min_width: Val::VMin(26.0),
                    height: Val::VMin(5.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::horizontal(Val::VMin(1.0)),
                    border: UiRect::all(Val::VMin(0.22)),
                    border_radius: BorderRadius::all(Val::VMin(0.44)),
                    ..default()
                },
                BackgroundColor(crate::core::constants::NORMAL_BUTTON_COLOR),
                BorderColor::all(crate::core::constants::BUTTON_BORDER_COLOR),
                Button,
                Interaction::default(),
                Pickable::default(),
                CombatContinueWithPetButton,
            ))
            .observe(crate::core::menu::utils::recolor::<Over>(
                crate::core::constants::HOVERED_BUTTON_COLOR,
            ))
            .observe(crate::core::menu::utils::recolor::<Out>(
                crate::core::constants::NORMAL_BUTTON_COLOR,
            ))
            .observe(crate::core::menu::utils::recolor::<Press>(
                crate::core::constants::PRESSED_BUTTON_COLOR,
            ))
            .observe(crate::core::menu::utils::recolor::<Release>(
                crate::core::constants::HOVERED_BUTTON_COLOR,
            ))
            .observe(crate::core::utils::cursor::<Over>(bevy::window::SystemCursorIcon::Pointer))
            .observe(crate::core::utils::cursor::<Out>(bevy::window::SystemCursorIcon::Default))
            .observe(crate::core::utils::cursor::<Release>(bevy::window::SystemCursorIcon::Default))
            .observe(handle_continue_with_pet_button_click)
            .with_children(|parent| {
                parent.spawn((
                    crate::core::menu::utils::add_text(
                        localization.get("general.continue_with_pet", lang),
                        "bold",
                        2.2,
                        assets,
                    ),
                    TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                ));
            });
    });
}

/// Keeps the portraits' debuff icon bars in sync with the active negative combat
/// effects. Shows one icon per distinct debuff currently affecting the fighter.
pub fn sync_combat_effect_icons(
    mut commands: Commands,
    state: Option<Res<CombatState>>,
    assets: Res<crate::core::assets::WorldAssets>,
    bar_q: Query<(Entity, &crate::core::combat::ui::CombatEffectsBar)>,
    icon_q: Query<&CombatEffectIcon>,
    children_q: Query<&Children>,
) {
    let Some(state) = state else {
        return;
    };

    for (bar_entity, bar) in &bar_q {
        let effects_opt = match bar.side {
            crate::core::combat::ui::CombatEffectsBarSide::Player => Some(&state.player.effects),
            crate::core::combat::ui::CombatEffectsBarSide::Enemy => Some(&state.enemy.effects),
            crate::core::combat::ui::CombatEffectsBarSide::Pet => {
                state.pet.as_ref().map(|p| &p.effects)
            },
        };

        // Distinct debuff effects currently on the target, preserving order.
        let mut desired: Vec<Effect> = Vec::new();
        let mut desired_keys: Vec<&'static str> = Vec::new();
        if let Some(effects) = effects_opt {
            for te in effects {
                if let Some(key) = te.effect.debuff_icon() {
                    if !desired_keys.contains(&key) {
                        desired_keys.push(key);
                        desired.push(te.effect.clone());
                    }
                }
            }
        }

        // Icon keys already rendered, in child order.
        let mut existing_keys: Vec<&'static str> = Vec::new();
        if let Ok(children) = children_q.get(bar_entity) {
            for child in children.iter() {
                if let Ok(icon) = icon_q.get(child) {
                    if let Some(key) = icon.effect.debuff_icon() {
                        existing_keys.push(key);
                    }
                }
            }
        }

        if existing_keys == desired_keys {
            continue;
        }

        despawn_descendants_manual(&mut commands, bar_entity, &children_q);

        commands.entity(bar_entity).with_children(|parent| {
            for effect in &desired {
                let Some(key) = effect.debuff_icon() else {
                    continue;
                };
                parent.spawn((
                    Node {
                        width: Val::VMin(5.5),
                        height: Val::VMin(5.5),
                        ..default()
                    },
                    ImageNode::new(assets.image(key)).with_mode(NodeImageMode::Stretch),
                    Interaction::default(),
                    Pickable::default(),
                    CombatEffectIcon {
                        effect: effect.clone(),
                    },
                ));
            }
        });
    }
}

/// Shows a tooltip (effect name + description) when hovering a debuff icon on
/// the player's portrait during combat.
pub fn combat_effect_tooltip_system(
    mut commands: Commands,
    assets: Res<crate::core::assets::WorldAssets>,
    localization: Res<crate::core::localization::Localization>,
    settings: Res<crate::core::settings::Settings>,
    icon_q: Query<(&Interaction, &CombatEffectIcon)>,
    changed_q: Query<(), (With<CombatEffectIcon>, Changed<Interaction>)>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if changed_q.is_empty() {
        return;
    }

    let mut hovered: Option<&Effect> = None;
    for (interaction, icon) in &icon_q {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            hovered = Some(&icon.effect);
            break;
        }
    }

    for entity in tooltip_q.iter() {
        commands.entity(entity).try_despawn();
    }

    if let Some(effect) = hovered {
        let (title, desc) = effect.title_and_description(settings.language, &localization);
        crate::core::ui::tooltip::spawn_item_tooltip(
            &mut commands,
            &assets,
            title,
            vec![desc],
            &windows,
            None,
            None,
            0.0,
        );
    }
}

/// Shows tactical descriptions when hovering Guard or an auto-attack stance.
pub fn combat_tactic_tooltip_system(
    mut commands: Commands,
    state: Option<Res<CombatState>>,
    assets: Res<crate::core::assets::WorldAssets>,
    localization: Res<crate::core::localization::Localization>,
    settings: Res<crate::core::settings::Settings>,
    card_q: Query<
        (&Interaction, &CombatCard),
        Without<crate::core::ui::playing::RightColumnTooltip>,
    >,
    changed_q: Query<
        (),
        (
            With<CombatCard>,
            Changed<Interaction>,
            Without<crate::core::ui::playing::RightColumnTooltip>,
        ),
    >,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if changed_q.is_empty() {
        return;
    }
    let Some(state) = state else {
        return;
    };
    let hovered = card_q.iter().find_map(|(interaction, card)| {
        matches!(*interaction, Interaction::Hovered | Interaction::Pressed).then_some(card)
    });
    for entity in tooltip_q.iter() {
        commands.entity(entity).try_despawn();
    }
    let Some(card) = hovered else {
        return;
    };
    let language = settings.language;
    let (title, description, image) = match card {
        CombatCard::Guard => (
            localization.get("combat.guard_name", language),
            localization
                .get("combat.guard_desc", language)
                .replace("{window}", &format!("{:.2}", state.stance.perfect_guard_window()))
                .replace("{mana}", &format!("{:.0}", guard_mana_cost(state.player_level))),
            Some("combat_guard".to_string()),
        ),
        CombatCard::Stance(stance) => (
            localization.get(stance.name_key(), language),
            localization.get(
                match stance {
                    CombatStance::Aggressive => "combat.stance_aggressive_desc",
                    CombatStance::Defensive => "combat.stance_defensive_desc",
                    CombatStance::Precise => "combat.stance_precise_desc",
                    CombatStance::Disruptive => "combat.stance_disruptive_desc",
                },
                language,
            ),
            Some(stance.image_key().to_string()),
        ),
        CombatCard::Ability(_) | CombatCard::Consumable(_) => return,
    };
    crate::core::ui::tooltip::spawn_item_tooltip(
        &mut commands,
        &assets,
        title,
        vec![description],
        &windows,
        None,
        image,
        0.0,
    );
}

/// Updates combat equipment slots.
pub fn update_combat_equipment_slots(
    player: Res<Player>,
    duel_state: Option<Res<crate::core::network::DuelState>>,
    assets: Res<crate::core::assets::WorldAssets>,
    mut slot_q: Query<(
        &CombatSlot,
        &crate::core::ui::playing::EquipSlot,
        &mut ImageNode,
        &mut Visibility,
    )>,
) {
    let opponent = duel_state.as_ref().and_then(|d| d.opponent.as_ref());

    let is_p_lh_two_hand = player
        .weapon_lh
        .as_deref()
        .and_then(get_equipment)
        .map(|eq| match eq {
            Equipment::Weapon(w) => w.hand == crate::core::catalog::weapons::Hand::TwoHand,
            _ => false,
        })
        .unwrap_or(false);

    let is_e_lh_two_hand = opponent
        .and_then(|opp| opp.weapon_lh.as_deref())
        .and_then(get_equipment)
        .map(|eq| match eq {
            Equipment::Weapon(w) => w.hand == crate::core::catalog::weapons::Hand::TwoHand,
            _ => false,
        })
        .unwrap_or(false);

    for (combat_slot, slot, mut image, mut vis) in &mut slot_q {
        if combat_slot.is_player {
            let equipped_key = match slot {
                crate::core::ui::playing::EquipSlot::Helmet => player.helmet.as_deref(),
                crate::core::ui::playing::EquipSlot::Accessory => player.accessory.as_deref(),
                crate::core::ui::playing::EquipSlot::Accessory2 => player.accessory2.as_deref(),
                crate::core::ui::playing::EquipSlot::WeaponLH => player.weapon_lh.as_deref(),
                crate::core::ui::playing::EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
                crate::core::ui::playing::EquipSlot::Chestplate => player.armor.as_deref(),
                crate::core::ui::playing::EquipSlot::Boots => player.boots.as_deref(),
                crate::core::ui::playing::EquipSlot::Gloves => player.gloves.as_deref(),
            };

            let img_handle = match equipped_key {
                Some(key) => get_equipment(key)
                    .map(|equipment| assets.image(equipment.image()))
                    .unwrap_or_else(|| assets.image("stone")),
                None => assets.image("stone"),
            };
            if image.image != img_handle {
                image.image = img_handle;
            }

            let visible = match slot {
                crate::core::ui::playing::EquipSlot::Helmet => player.helmet.is_some(),
                crate::core::ui::playing::EquipSlot::Accessory => player.accessory.is_some(),
                crate::core::ui::playing::EquipSlot::Accessory2 => player.accessory2.is_some(),
                crate::core::ui::playing::EquipSlot::WeaponLH => player.weapon_lh.is_some(),
                crate::core::ui::playing::EquipSlot::WeaponRH => {
                    player.weapon_rh.is_some() && !is_p_lh_two_hand
                },
                crate::core::ui::playing::EquipSlot::Chestplate => player.armor.is_some(),
                crate::core::ui::playing::EquipSlot::Boots => player.boots.is_some(),
                crate::core::ui::playing::EquipSlot::Gloves => player.gloves.is_some(),
            };

            let target_vis = if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *vis != target_vis {
                *vis = target_vis;
            }
        } else if let Some(opp) = opponent {
            let equipped_key = match slot {
                crate::core::ui::playing::EquipSlot::Helmet => opp.helmet.as_deref(),
                crate::core::ui::playing::EquipSlot::Accessory => opp.accessory.as_deref(),
                crate::core::ui::playing::EquipSlot::Accessory2 => opp.accessory2.as_deref(),
                crate::core::ui::playing::EquipSlot::WeaponLH => opp.weapon_lh.as_deref(),
                crate::core::ui::playing::EquipSlot::WeaponRH => opp.weapon_rh.as_deref(),
                crate::core::ui::playing::EquipSlot::Chestplate => opp.armor.as_deref(),
                crate::core::ui::playing::EquipSlot::Boots => opp.boots.as_deref(),
                crate::core::ui::playing::EquipSlot::Gloves => opp.gloves.as_deref(),
            };

            let img_handle = match equipped_key {
                Some(key) => get_equipment(key)
                    .map(|equipment| assets.image(equipment.image()))
                    .unwrap_or_else(|| assets.image("stone")),
                None => assets.image("stone"),
            };
            if image.image != img_handle {
                image.image = img_handle;
            }

            let visible = match slot {
                crate::core::ui::playing::EquipSlot::Helmet => opp.helmet.is_some(),
                crate::core::ui::playing::EquipSlot::Accessory => opp.accessory.is_some(),
                crate::core::ui::playing::EquipSlot::Accessory2 => opp.accessory2.is_some(),
                crate::core::ui::playing::EquipSlot::WeaponLH => opp.weapon_lh.is_some(),
                crate::core::ui::playing::EquipSlot::WeaponRH => {
                    opp.weapon_rh.is_some() && !is_e_lh_two_hand
                },
                crate::core::ui::playing::EquipSlot::Chestplate => opp.armor.is_some(),
                crate::core::ui::playing::EquipSlot::Boots => opp.boots.is_some(),
                crate::core::ui::playing::EquipSlot::Gloves => opp.gloves.is_some(),
            };

            let target_vis = if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *vis != target_vis {
                *vis = target_vis;
            }
        } else {
            let target_vis = Visibility::Hidden;
            if *vis != target_vis {
                *vis = target_vis;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::catalog::catalog::{all_consumables, all_weapons, all_wearables};
    use crate::core::catalog::wearables::WearableSlot;
    use crate::core::classes::Class;
    use crate::core::deities::Deity;

    #[test]
    /// Verifies combat selects five copies per type while preserving excess inventory stock.
    fn combat_consumable_selection_caps_each_type_at_five() {
        let key =
            all_consumables().first().expect("consumable catalog should not be empty").name.clone();
        let mut player = Player::default();
        player.equipped_consumables.push(key.clone());
        player.inventory.extend(std::iter::repeat_n(key.clone(), 8));

        let mut selected = select_combat_consumables(&player);

        assert_eq!(selected.get(&key), Some(&MAX_COMBAT_CONSUMABLES_PER_TYPE));
        assert_eq!(player.inventory.iter().filter(|item| *item == &key).count(), 8);
        for _ in 0..MAX_COMBAT_CONSUMABLES_PER_TYPE {
            assert!(take_combat_consumable(&mut selected, &key));
        }
        assert!(!take_combat_consumable(&mut selected, &key));
        assert_eq!(player.inventory.iter().filter(|item| *item == &key).count(), 8);
    }

    /// Returns a neutral fighter suitable for isolated combat-mechanics tests.
    fn test_fighter() -> Fighter {
        Fighter {
            max_health: 100.0,
            health: 100.0,
            display_health: 100.0,
            max_mana: 50.0,
            mana: 50.0,
            display_mana: 50.0,
            base_attack: 10.0,
            base_defense: 5.0,
            base_initiative: 10.0,
            base_attack_speed: 1.0,
            crit_chance: 0.05,
            health_regen: 1.0,
            mana_regen: 1.0,
            attack_timer: 0.0,
            effects: Vec::new(),
            weapon_effects: Vec::new(),
            attack_style: AttackStyle::Melee,
            intelligence_mod: 0.0,
            passive_modifiers: Vec::new(),
            mutation: None,
            alive: true,
            weapons: Vec::new(),
        }
    }

    /// Returns a minimal two-fighter combat state for effect tests.
    fn test_combat_state() -> CombatState {
        CombatState {
            player: test_fighter(),
            pet: None,
            enemy: test_fighter(),
            enemy_pet: None,
            abilities: Vec::new(),
            enemy_abilities: Vec::new(),
            player_consumables: HashMap::new(),
            enemy_consumables: HashMap::new(),
            stance: CombatStance::Aggressive,
            guard_remaining: 0.0,
            perfect_guard_remaining: 0.0,
            enemy_poise: 50.0,
            enemy_max_poise: 50.0,
            enemy_break_remaining: 0.0,
            enemy_tactics: None,
            status: CombatStatus::Ongoing,
            player_won: false,
            player_level: 1,
            enemy_level: 1,
            mutation_candidate: None,
            fx: Vec::new(),
            paused: false,
            dodge_word: "Dodge".to_string(),
            miss_word: "Miss".to_string(),
            xp_word: "XP".to_string(),
            guard_word: "Guard".to_string(),
            parry_word: "Parry".to_string(),
            break_word: "Break".to_string(),
            shatter_word: "Shatter".to_string(),
            detonate_word: "Detonate".to_string(),
            doom_word: "Doom".to_string(),
            exploit_word: "Exploit".to_string(),
            cleanse_word: "Cleanse".to_string(),
            phase_word: "Phase Two".to_string(),
        }
    }

    #[test]
    /// Verifies Guard spends Mana and cannot activate when its cost is unavailable.
    fn guard_requires_and_spends_mana() {
        let mut state = test_combat_state();
        state.player_level = 10;
        state.player.mana = 100.0;
        let starting_mana = state.player.mana;
        let mana_cost = guard_mana_cost(state.player_level);

        assert!(begin_guard(&mut state));
        assert_eq!(state.player.mana, starting_mana - mana_cost);
        assert_eq!(state.guard_remaining, GUARD_DURATION);

        state.guard_remaining = 0.0;
        state.perfect_guard_remaining = 0.0;
        state.player.mana = mana_cost - 1.0;
        assert!(!begin_guard(&mut state));
        assert_eq!(state.guard_remaining, 0.0);
    }

    #[test]
    /// Verifies listed regeneration values are the exact resources restored per second.
    fn combat_regeneration_uses_the_listed_per_second_values() {
        let mut fighter = test_fighter();
        fighter.health = 50.0;
        fighter.mana = 20.0;
        fighter.health_regen = 3.0;
        fighter.mana_regen = 4.0;

        regenerate_fighter(&mut fighter, 2.0);

        assert_eq!(fighter.health, 56.0);
        assert_eq!(fighter.mana, 28.0);
    }

    /// Advances an integration-test combat by a large deterministic time slice.
    fn advance_test_combat(
        mut state: ResMut<CombatState>,
        mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    ) {
        step_combat(&mut state, 3.0, &mut play_audio_msg);
    }

    #[test]
    /// Verifies a telegraphed move is announced before it resolves damage.
    fn enemy_moves_telegraph_before_resolving() {
        let mut state = test_combat_state();
        state.enemy_tactics = Some(EnemyTactics {
            archetype: MonsterArchetype::Berserker,
            rotation: vec![EnemyMove {
                kind: EnemyMoveKind::CrushingBlow,
                cast_time: 2.0,
                recovery: 3.0,
                target: EnemyMoveTarget::Player,
            }],
            next_index: 0,
            recovery: 0.0,
            recovery_max: 0.0,
            active_cast: None,
            phase_two: false,
        });
        let starting_health = state.player.health;
        let mut app = App::new();
        app.add_message::<PlayAudioMsg>()
            .insert_resource(state)
            .add_systems(Update, advance_test_combat);

        app.update();
        assert_eq!(app.world().resource::<CombatState>().player.health, starting_health);
        assert!(app
            .world()
            .resource::<CombatState>()
            .enemy_tactics
            .as_ref()
            .is_some_and(|tactics| tactics.active_cast.is_some()));

        app.update();
        assert!(app.world().resource::<CombatState>().player.health < starting_health);
        let tactics = app.world().resource::<CombatState>().enemy_tactics.as_ref().unwrap();
        assert_eq!(tactics.recovery, 3.0);
        assert_eq!(tactics.recovery_max, tactics.recovery);
    }

    #[test]
    /// Verifies the tactical HUD's mutable UI queries remain explicitly disjoint.
    fn combat_tactics_visual_queries_are_disjoint() {
        let mut world = World::new();
        let mut system = IntoSystem::into_system(update_combat_tactics_visuals);

        system.initialize(&mut world);
    }

    #[test]
    /// Verifies a physical payoff consumes Freeze and inflicts Shatter damage.
    fn physical_payoff_shatters_freeze() {
        let mut state = test_combat_state();
        state.enemy.effects.push(TimedEffect {
            effect: Effect::Freeze {
                attack_speed_pct: -20.0,
                duration: 4.0,
            },
            remaining: 4.0,
            tick_acc: 0.0,
            magnitude_multiplier: 1.0,
        });
        let ability = crate::core::catalog::abilities::Ability {
            name: "Shatter Test".to_string(),
            image: "test".to_string(),
            level: 1,
            kind: Kind::Physical,
            mana_cost: 1,
            cooldown: 2.0,
            on_self: false,
            is_aoe: false,
            effects: vec![Effect::Pierce {
                damage: 10,
            }],
        };
        let health_before = state.enemy.health;

        let bonus_poise = resolve_ability_combos(&mut state, &ability);

        assert!(state.enemy.health < health_before);
        assert!(bonus_poise >= 20.0);
        assert!(!state
            .enemy
            .effects
            .iter()
            .any(|timed| matches!(timed.effect, Effect::Freeze { .. })));
    }

    #[test]
    /// Verifies Purge converts removed pressure into health.
    fn purge_rewards_scale_with_removed_debuffs() {
        let mut state = test_combat_state();
        state.player.health = 50.0;

        reward_purge_combo(&mut state, 2);

        assert!(state.player.health > 50.0);
    }

    #[test]
    /// Verifies pet-targeting enemy moves fall back to the player when no pet lives.
    fn pet_target_falls_back_to_player() {
        let state = test_combat_state();
        assert_eq!(enemy_move_target(&state, EnemyMoveTarget::Pet), Who::Player);
    }

    #[test]
    /// Verifies that per-weapon attack and speed calculations remain stable.
    fn test_fighter_dual_wield_mechanics() {
        let fighter = test_fighter();

        // Test eff_attack_speed_for and attack_period_for with a base speed of 1.2
        let speed = fighter.eff_attack_speed_for(1.2);
        assert_eq!(speed, 1.2);
        let period = fighter.attack_period_for(1.2);
        assert!((period - 1.6666667).abs() < 0.0001);

        // Test eff_attack_for with a base attack of 15.0
        let attack = fighter.eff_attack_for(15.0);
        assert_eq!(attack, 15.0);
    }

    #[test]
    /// Verifies every percentage-based passive modifier reaches combat math.
    fn passive_combat_modifiers_are_applied() {
        let mut fighter = test_fighter();
        fighter.passive_modifiers = vec![
            Modifier::KindPowerMultiplier(Kind::Fire, 10.0),
            Modifier::CategoryPowerMultiplier(Category::Range, 5.0),
            Modifier::KindResistanceMultiplier(Kind::Ice, 20.0),
            Modifier::CategoryResistanceMultiplier(Category::Range, 5.0),
            Modifier::LifeSteal(7.0),
            Modifier::HealingMultiplier(12.0),
        ];

        assert!(
            (fighter.outgoing_damage_multiplier(Kind::Fire, Some(Category::Range)) - 1.15).abs()
                < f32::EPSILON
        );
        assert!(
            (fighter.incoming_damage_multiplier(Kind::Ice, Some(Category::Range)) - 0.75).abs()
                < f32::EPSILON
        );
        assert!((fighter.lifesteal() - 0.07).abs() < f32::EPSILON);
        assert!((fighter.healing_multiplier() - 1.12).abs() < f32::EPSILON);
    }

    #[test]
    /// Verifies direct scaling remains attached to periodic damage and healing.
    fn timed_effect_magnitude_is_applied() {
        let mut state = test_combat_state();
        push_timed(
            &mut state,
            Who::Player,
            Effect::Poison {
                damage: 2,
                duration: 2.0,
            },
            1.5,
        );

        tick_fighter_effects(&mut state.player, 1.0);

        assert!((state.player.health - 97.0).abs() < f32::EPSILON);
    }

    #[test]
    /// Verifies dual wielding keeps shared attack singular and applies identity speed and crit.
    fn dual_wielding_applies_identity_bonuses_once() {
        let weapons = all_weapons()
            .iter()
            .filter(|weapon| weapon.category == Category::Finesse && weapon.level == 1)
            .take(2)
            .collect::<Vec<_>>();
        assert_eq!(weapons.len(), 2);
        let player = Player {
            class: Class::Monk,
            deity: Deity::Aeloria,
            weapon_lh: Some(weapons[0].name.clone()),
            weapon_rh: Some(weapons[1].name.clone()),
            ..default()
        };

        let combat_weapons = player_combat_weapons(&player);
        assert_eq!(combat_weapons.len(), 2);
        assert!(
            (combat_weapons.iter().map(|weapon| weapon.attack).sum::<f32>()
                - player.attack() as f32)
                .abs()
                < f32::EPSILON
        );
        for (combat_weapon, catalog_weapon) in combat_weapons.iter().zip(weapons) {
            assert!(
                (combat_weapon.attack_speed - catalog_weapon.attack_speed * 1.1).abs()
                    < f32::EPSILON
            );
            assert!(
                (combat_weapon.crit_chance - (catalog_weapon.crit_chance + 0.12)).abs()
                    < f32::EPSILON
            );
        }
    }

    #[test]
    /// Verifies defensive shields and books never create a basic auto-attack.
    fn shields_and_books_do_not_auto_attack() {
        for category in [Category::Shield, Category::Book] {
            let weapon = all_weapons()
                .iter()
                .find(|weapon| weapon.category == category)
                .unwrap_or_else(|| panic!("catalog contains a {category:?} weapon"));
            let player = Player {
                weapon_lh: Some(weapon.name.clone()),
                ..default()
            };

            assert!(player_combat_weapons(&player).is_empty());
        }
    }

    #[test]
    /// Verifies that ability dodge chance uses caster intelligence.
    fn test_ability_dodge_chance_uses_caster_intelligence() {
        let without_intelligence = ability_dodge_chance(10.0, 14.0, 0.0);
        let with_intelligence = ability_dodge_chance(10.0, 14.0, 5.0);

        assert!(with_intelligence < without_intelligence);
        assert!((without_intelligence - 0.252).abs() < 0.0001);
        assert!((with_intelligence - 0.162).abs() < 0.0001);
    }

    #[test]
    /// Verifies that wearable passives are collected as on-being-hit effects.
    fn wearable_effects_are_active_in_combat() {
        let wearable = all_wearables()
            .iter()
            .find(|wearable| !wearable.effects.is_empty())
            .expect("the generated wearable catalog should contain passive effects");
        let mut player = Player::default();
        match wearable.slot {
            WearableSlot::Accessory => player.accessory = Some(wearable.name.clone()),
            WearableSlot::Helmet => player.helmet = Some(wearable.name.clone()),
            WearableSlot::Chestplate => player.armor = Some(wearable.name.clone()),
            WearableSlot::Gloves => player.gloves = Some(wearable.name.clone()),
            WearableSlot::Boots => player.boots = Some(wearable.name.clone()),
        }

        let collected = player_equipment_effects(&player);
        for effect in &wearable.effects {
            assert!(
                collected.iter().any(|item| !item.on_hit && item.effect == *effect),
                "{} passive is missing from combat",
                wearable.name
            );
        }
    }

    #[test]
    /// Verifies that repeated non-stackable effects refresh instead of multiplying.
    fn timed_effects_refresh_without_stacking() {
        let mut state = test_combat_state();
        push_timed(
            &mut state,
            Who::Player,
            Effect::Haste {
                initiative_pct: 10.0,
                duration: 5.0,
            },
            1.0,
        );
        push_timed(
            &mut state,
            Who::Player,
            Effect::Haste {
                initiative_pct: 20.0,
                duration: 3.0,
            },
            1.0,
        );

        assert_eq!(state.player.effects.len(), 1);
        assert_eq!(state.player.effects[0].remaining, 5.0);
        assert!(matches!(
            state.player.effects[0].effect,
            Effect::Haste {
                initiative_pct: 20.0,
                ..
            }
        ));
    }

    #[test]
    /// Verifies the vampire's damage bonus and matching Fire vulnerability.
    fn vampire_mutation_has_equal_damage_tradeoff() {
        let mut vampire = test_fighter();
        vampire.mutation = Some(Mutation::Vampire);

        assert!(
            (vampire.outgoing_damage_multiplier(Kind::Physical, None) - 1.15).abs() < f32::EPSILON
        );
        assert!((vampire.incoming_damage_multiplier(Kind::Fire, None) - 1.15).abs() < f32::EPSILON);
        assert!((vampire.incoming_damage_multiplier(Kind::Ice, None) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    /// Verifies undead negate Poison and Freeze without ignoring other effects.
    fn undead_mutation_blocks_poison_and_freeze() {
        let mut undead = test_fighter();
        undead.mutation = Some(Mutation::Undead);

        assert!(undead.is_immune_to_effect(&Effect::Poison {
            damage: 5,
            duration: 3.0,
        }));
        assert!(undead.is_immune_to_effect(&Effect::Freeze {
            attack_speed_pct: -20.0,
            duration: 3.0,
        }));
        assert!(!undead.is_immune_to_effect(&Effect::Burn {
            damage: 5,
            duration: 3.0,
        }));
    }

    #[test]
    /// Verifies every eligible combat queues its mutation offer without a random roll.
    fn eligible_combat_always_queues_mutation_offer() {
        let mut app = App::new();
        app.add_systems(Update, |mut commands: Commands| {
            maybe_queue_mutation_offer(&mut commands, Some(Mutation::Werewolf));
        });

        app.update();

        assert_eq!(
            app.world().resource::<crate::core::ui::mutation::PendingMutationOffer>().0,
            Mutation::Werewolf
        );
    }
}
