use bevy::prelude::*;
use rand::{rng, RngExt};

use crate::core::actions::hunt::PendingHuntXp;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::{get_ability, get_equipment};
use crate::core::catalog::effects::Effect;
use crate::core::catalog::equipment::Equipment;
use crate::core::catalog::equipment::Kind;
use crate::core::catalog::weapons::{Category, Weapon};
use crate::core::combat::ui::{CombatCmp, CombatPortraitName, CombatPortraitLevel, CombatStatLabel, CombatPetName};
use crate::core::menu::systems::{CombatMenuSuspended, GameMenuOrigin};
use crate::core::monsters::ActiveMonster;
use crate::core::player::Player;
use crate::core::states::GameState;
use crate::core::ui::playing::TooltipNode;

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

const BAR_LERP_SPEED: f32 = 6.0;
const ATTACK_PERIOD_MULTIPLIER: f32 = 2.0;
const ABILITY_MANA_COST_MULTIPLIER: u32 = 2;
const HIT_TEXT_SIZE: f32 = 4.6;
const XP_REWARD_TEXT_SIZE: f32 = 9.0;
const XP_REWARD_TEXT_LIFE: f32 = 3.2;
const DEATH_SKULL_ANIM_DURATION: f32 = 0.9;
const DEATH_SKULL_START_SIZE: f32 = 6.0;
const DEATH_SKULL_END_SIZE: f32 = 50.0;
/// Scales down health/mana regeneration during combat so fights don't drag on
/// when regen would otherwise outpace incoming damage.
const COMBAT_REGEN_MULTIPLIER: f32 = 0.3;
/// Bounds and step for the adjustable combat speed.
const COMBAT_SPEED_MIN: f32 = 0.25;
const COMBAT_SPEED_MAX: f32 = 8.0;

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
    fn default() -> Self {
        Self(1.0)
    }
}

impl CombatSpeed {
    pub fn faster(&mut self) {
        self.0 = (self.0 * 2.0).min(COMBAT_SPEED_MAX);
    }

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
}

/// Dark overlay child of an ability card. Its height encodes cooldown progress.
#[derive(Component)]
pub struct AbilityCooldownOverlay {
    pub slot: usize,
}

/// Root node of a consumable card, tagged with its catalog key for despawn/sync.
#[derive(Component)]
pub struct ConsumableCardRoot(pub String);

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
    pub alive: bool,
    pub weapons: Vec<CombatWeapon>,
}

impl Fighter {
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

    pub fn attack_period_for(&self, base_speed: f32) -> f32 {
        (ATTACK_PERIOD_MULTIPLIER / self.eff_attack_speed_for(base_speed)).clamp(0.2, 10.0)
    }

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
        v.max(0.0)
    }

    #[allow(dead_code)]
    fn attack_period(&self) -> f32 {
        (ATTACK_PERIOD_MULTIPLIER / self.eff_attack_speed()).clamp(0.2, 10.0)
    }

    #[allow(dead_code)]
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
        v.max(0.0)
    }

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
        v.max(0.0)
    }

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

    fn can_dodge(&self) -> bool {
        !self.effects.iter().any(|te| matches!(te.effect, Effect::Immobilize { .. }))
    }

    fn can_act(&self) -> bool {
        !self
            .effects
            .iter()
            .any(|te| matches!(te.effect, Effect::Stun { .. } | Effect::MonarchShield { .. }))
    }

    fn can_cast(&self) -> bool {
        !self.effects.iter().any(|te| {
            matches!(
                te.effect,
                Effect::Silence { .. } | Effect::Stun { .. } | Effect::MonarchShield { .. }
            )
        })
    }

    fn lifesteal(&self) -> f32 {
        let mut v = 0.0;
        for te in &self.effects {
            if let Effect::Lifesteal {
                percentage,
                ..
            } = &te.effect
            {
                v += percentage / 100.0;
            }
        }
        v
    }

    fn take_damage(&mut self, dmg: f32) {
        self.health = (self.health - dmg).max(0.0);
        if self.health <= 0.0 {
            self.alive = false;
        }
    }

    fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

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
    pub abilities: Vec<AbilitySlot>,
    pub status: CombatStatus,
    pub player_won: bool,
    pub player_level: u32,
    pub enemy_level: u32,
    pub fx: Vec<CombatFx>,
    pub paused: bool,
    pub dodge_word: String,
    pub miss_word: String,
    pub xp_word: String,
}

impl CombatState {
    pub fn xp_reward(&self) -> u32 {
        if !self.player_won {
            return 0;
        }
        let diff = self.enemy_level as i32 - self.player_level as i32;
        (2 + diff).max(0) as u32
    }
}

fn player_weapon_effects(player: &Player) -> Vec<WeaponEffect> {
    let mut out = Vec::new();
    for eq in player.equipped_equipment() {
        if let Equipment::Weapon(w) = eq {
            let on_hit = !matches!(w.category, Category::Shield | Category::Book);
            for e in &w.effects {
                out.push(WeaponEffect {
                    effect: e.clone(),
                    on_hit,
                });
            }
        }
    }
    out
}

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

pub fn setup_combat_state(
    mut commands: Commands,
    player: Res<Player>,
    active_monster: Option<Res<ActiveMonster>>,
    settings: Res<crate::core::settings::Settings>,
    localization: Res<crate::core::localization::Localization>,
    existing_state: Option<Res<CombatState>>,
) {
    if existing_state.is_some() {
        return;
    }
    let Some(active_monster) = active_monster else {
        return;
    };
    let monster = &active_monster.monster;

    let player_equipped = player.equipped_equipment();
    let attacking_weapons: Vec<&Weapon> = player_equipped
        .iter()
        .filter_map(|eq| {
            if let Equipment::Weapon(w) = eq {
                if !matches!(w.category, Category::Shield | Category::Book) {
                    Some(w)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let player_weapons = if attacking_weapons.len() == 2 {
        let base_player_attack = (player.attack() as i32
            - player_equipped.iter().map(|eq| eq.attack()).sum::<i32>())
        .max(0) as f32;

        attacking_weapons
            .into_iter()
            .map(|w| {
                let attack_style = match w.category {
                    Category::Range => AttackStyle::Range,
                    Category::Finesse => AttackStyle::Finesse,
                    Category::Melee => AttackStyle::Melee,
                    _ => AttackStyle::Other,
                };
                CombatWeapon {
                    name: w.name.clone(),
                    attack_speed: w.attack_speed,
                    attack_timer: 0.0,
                    attack: base_player_attack + w.attack as f32,
                    crit_chance: w.crit_chance,
                    effects: w
                        .effects
                        .iter()
                        .map(|e| WeaponEffect {
                            effect: e.clone(),
                            on_hit: true,
                        })
                        .collect(),
                    attack_style,
                }
            })
            .collect()
    } else {
        vec![CombatWeapon {
            name: "Primary Weapon".to_string(),
            attack_speed: player.attack_speed(),
            attack_timer: 0.0,
            attack: player.attack() as f32,
            crit_chance: player.crit_chance(),
            effects: player_weapon_effects(&player),
            attack_style: player_attack_style(&player),
        }]
    };

    let player_fighter = Fighter {
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
        weapon_effects: player_weapon_effects(&player),
        attack_style: player_attack_style(&player),
        alive: true,
        weapons: player_weapons,
    };

    let pet_fighter = player.pet.as_ref().map(|pet| Fighter {
        max_health: pet.max_health as f32,
        health: pet.health as f32,
        display_health: pet.health as f32,
        max_mana: 0.0,
        mana: 0.0,
        display_mana: 0.0,
        base_attack: pet.attack as f32,
        base_defense: pet.defense as f32,
        base_initiative: pet.initiative as f32,
        base_attack_speed: pet.attack_speed,
        crit_chance: 0.0,
        health_regen: pet.health_regen as f32,
        mana_regen: 0.0,
        attack_timer: 0.0,
        effects: Vec::new(),
        weapon_effects: pet
            .effects
            .iter()
            .map(|e| WeaponEffect {
                effect: e.clone(),
                on_hit: true,
            })
            .collect(),
        attack_style: AttackStyle::Other,
        alive: true,
        weapons: vec![CombatWeapon {
            name: "Basic Attack".to_string(),
            attack_speed: pet.attack_speed,
            attack_timer: 0.0,
            attack: pet.attack as f32,
            crit_chance: 0.0,
            effects: pet
                .effects
                .iter()
                .map(|e| WeaponEffect {
                    effect: e.clone(),
                    on_hit: true,
                })
                .collect(),
            attack_style: AttackStyle::Other,
        }],
    });

    let enemy_fighter = Fighter {
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
        weapon_effects: monster
            .effects
            .iter()
            .map(|e| WeaponEffect {
                effect: e.clone(),
                on_hit: true,
            })
            .collect(),
        attack_style: AttackStyle::Other,
        alive: true,
        weapons: vec![CombatWeapon {
            name: "Basic Attack".to_string(),
            attack_speed: monster.attack_speed,
            attack_timer: 0.0,
            attack: monster.attack as f32,
            crit_chance: 0.0,
            effects: monster
                .effects
                .iter()
                .map(|e| WeaponEffect {
                    effect: e.clone(),
                    on_hit: true,
                })
                .collect(),
            attack_style: AttackStyle::Other,
        }],
    };

    let abilities = player
        .active_abilities
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
        .collect();

    commands.insert_resource(CombatState {
        player: player_fighter,
        pet: pet_fighter,
        enemy: enemy_fighter,
        abilities,
        status: CombatStatus::Ongoing,
        player_won: false,
        player_level: player.level(),
        enemy_level: monster.level,
        fx: Vec::new(),
        paused: false,
        dodge_word: localization.get("general.dodge", settings.language),
        miss_word: localization.get("general.miss", settings.language),
        xp_word: localization.get("general.xp", settings.language),
    });
}

// ---------------------------------------------------------------------------
// Combat tick
// ---------------------------------------------------------------------------

fn dodge_chance(attacker_init: f32, defender_init: f32) -> f32 {
    (0.18 + (defender_init - attacker_init) * 0.018).clamp(0.08, 0.70)
}

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

#[derive(Clone, Copy)]
enum Who {
    Player,
    Pet,
    Enemy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AttackOutcome {
    Miss,
    Dodge,
    Hit,
}

fn random_sword_slice_key() -> &'static str {
    let mut rng = rng();
    match rng.random_range(0..4) {
        0 => "sword_slice",
        1 => "sword_slice_2",
        2 => "sword_slice_3",
        _ => "sword_slice_violent",
    }
}

fn on_attack_launch_sound(style: AttackStyle) -> Option<&'static str> {
    match style {
        AttackStyle::Range => Some("arrow_swish"),
        _ => None,
    }
}

fn on_attack_hit_sound(style: AttackStyle) -> &'static str {
    match style {
        AttackStyle::Range => "arrow_impact",
        AttackStyle::Melee | AttackStyle::Finesse => random_sword_slice_key(),
        AttackStyle::Other => "armor_impact",
    }
}

fn on_attack_dodge_sound(style: AttackStyle) -> &'static str {
    match style {
        AttackStyle::Melee | AttackStyle::Finesse => "sword_clash",
        _ => "armor_impact",
    }
}

impl CombatState {
    fn get(&self, who: Who) -> Option<&Fighter> {
        match who {
            Who::Player => Some(&self.player),
            Who::Pet => self.pet.as_ref(),
            Who::Enemy => Some(&self.enemy),
        }
    }

    fn get_mut(&mut self, who: Who) -> Option<&mut Fighter> {
        match who {
            Who::Player => Some(&mut self.player),
            Who::Pet => self.pet.as_mut(),
            Who::Enemy => Some(&mut self.enemy),
        }
    }
}

/// Resolves a basic attack from `attacker` to `defender`.
fn resolve_basic_attack(
    state: &mut CombatState,
    attacker: Who,
    defender: Who,
    weapon_index: usize,
) -> Option<(AttackStyle, AttackOutcome)> {
    let (atk, atk_init, crit_chance, extra_crit, miss, weapon_effects, lifesteal, attack_style) = {
        let a = match state.get(attacker) {
            Some(a) if a.alive => a,
            _ => return None,
        };
        let weapon = match a.weapons.get(weapon_index) {
            Some(w) => w,
            None => return None,
        };
        (
            a.eff_attack_for(weapon.attack),
            a.eff_initiative(),
            weapon.crit_chance,
            a.extra_crit(),
            a.miss_chance(),
            weapon.effects.clone(),
            a.lifesteal(),
            weapon.attack_style,
        )
    };

    let (def, def_init, can_dodge, incoming_mult, def_alive) = {
        let d = match state.get(defender) {
            Some(d) => d,
            None => return None,
        };
        (d.eff_defense(), d.eff_initiative(), d.can_dodge(), d.incoming_multiplier(), d.alive)
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

    let crit = rng.random_bool((crit_chance + extra_crit).clamp(0.0, 1.0) as f64);

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

    let mut dmg = compute_damage(atk, def, crit, incoming_mult);
    dmg *= 1.0 + bonus_pct;

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

    // Lifesteal heals the attacker.
    if lifesteal > 0.0 {
        if let Some(a) = state.get_mut(attacker) {
            a.heal(dmg * lifesteal);
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

    // Monsters do not cast abilities in combat; keep their basic attack to a
    // single hit by skipping on-hit effect chains for enemy auto-attacks.
    if !matches!(attacker, Who::Enemy) {
        for we in weapon_effects.iter().filter(|w| w.on_hit) {
            apply_effect(state, attacker, defender, &we.effect);
        }
    }
    // Apply the defender's on-being-hit weapon effects back to the attacker.
    let def_when_hit: Vec<Effect> = state
        .get(defender)
        .map(|d| d.weapon_effects.iter().filter(|w| !w.on_hit).map(|w| w.effect.clone()).collect())
        .unwrap_or_default();
    for e in def_when_hit {
        apply_effect(state, defender, attacker, &e);
    }
    Some((attack_style, AttackOutcome::Hit))
}

fn side_of(who: Who) -> FxSide {
    match who {
        Who::Player | Who::Pet => FxSide::Player,
        Who::Enemy => FxSide::Enemy,
    }
}

/// Applies a single effect from `source` onto `target`.
fn apply_effect(state: &mut CombatState, source: Who, target: Who, effect: &Effect) {
    match effect {
        Effect::Heal {
            heal_pct,
        } => {
            if let Some(t) = state.get_mut(target) {
                let missing = t.max_health - t.health;
                t.heal(missing * (*heal_pct as f32 / 100.0));
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
            if matches!(effect, Effect::Pierce { .. }) {
                if let Some(t) = state.get_mut(target) {
                    t.take_damage(*damage as f32);
                }
                state.fx.push(CombatFx {
                    side: side_of(target),
                    text: format!("-{}", damage),
                    color: Color::srgb(1.0, 0.5, 0.3),
                });
            }
            push_timed(state, target, effect.clone());
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
                t.mana = (t.mana - *amount as f32).max(0.0);
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
        // Timed buffs / debuffs / damage-over-time / heal-over-time.
        _ => push_timed(state, target, effect.clone()),
    }
}

fn push_timed(state: &mut CombatState, target: Who, effect: Effect) {
    let duration = effect_duration(&effect);
    if let Some(t) = state.get_mut(target) {
        t.effects.push(TimedEffect {
            effect,
            remaining: duration,
            tick_acc: 0.0,
        });
    }
}

fn is_positive(effect: &Effect) -> bool {
    matches!(
        effect,
        Effect::Berserk { .. }
            | Effect::Empower { .. }
            | Effect::Fortify { .. }
            | Effect::Haste { .. }
            | Effect::Focus { .. }
            | Effect::Regen { .. }
            | Effect::ManaFlow { .. }
            | Effect::Lifesteal { .. }
            | Effect::Thorns { .. }
            | Effect::BeastFrenzy { .. }
            | Effect::MonarchShield { .. }
            | Effect::Clearcasting { .. }
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
            | Effect::Cleave { .. }
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

fn tick_fighter_effects(fighter: &mut Fighter, dt: f32) -> Vec<(FxSide, String, Color)> {
    let mut fx = Vec::new();
    let mut curse_damage = Vec::new();
    for te in fighter.effects.iter_mut() {
        let (hp_s, mp_s) = effect_per_second(&te.effect);
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
                curse_damage.push(*damage as f32);
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

    // Ability cooldowns.
    for slot in state.abilities.iter_mut() {
        if slot.remaining > 0.0 {
            slot.remaining = (slot.remaining - dt).max(0.0);
        }
    }

    // Regen.
    for who in [Who::Player, Who::Pet, Who::Enemy] {
        if let Some(f) = state.get_mut(who) {
            if f.alive {
                f.health =
                    (f.health + f.health_regen * COMBAT_REGEN_MULTIPLIER * dt).min(f.max_health);
                f.mana = (f.mana + f.mana_regen * COMBAT_REGEN_MULTIPLIER * dt).min(f.max_mana);
            }
        }
    }

    // Damage/heal over time + effect expiry.
    for who in [Who::Player, Who::Pet, Who::Enemy] {
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

    // Basic attacks paced by attack speed.
    for (attacker, defender) in
        [(Who::Player, Who::Enemy), (Who::Pet, Who::Enemy), (Who::Enemy, Who::Player)]
    {
        let num_weapons = {
            let Some(f) = state.get(attacker) else {
                continue;
            };
            if !f.alive || !f.can_act() {
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
                let Some(f) = state.get_mut(attacker) else {
                    continue;
                };
                let Some(w) = f.weapons.get(weapon_index) else {
                    continue;
                };
                let speed = w.attack_speed;
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

                if let Some((style, outcome)) = resolve_basic_attack(state, attacker, defender, weapon_index) {
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
        state.player_won = enemy_dead && (state.player.alive && state.player.display_health.round() as i32 > 0);
        if state.player_won {
            play_audio_msg.write(PlayAudioMsg::new("levelup").volume(-10.));
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

pub fn combat_tick(
    time: Res<Time>,
    combat_speed: Res<CombatSpeed>,
    mut state: Option<ResMut<CombatState>>,
    mut player: ResMut<Player>,
    active_monster: Option<ResMut<ActiveMonster>>,
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
    if slot.remaining > 0.0 || !state.player.can_cast() {
        play_audio_msg.write(PlayAudioMsg::new("error"));
        return;
    }
    if state.player.mana < slot.mana_cost as f32 {
        play_audio_msg.write(PlayAudioMsg::new("error"));
        return;
    }
    let Some(ability) = get_ability(&key) else {
        return;
    };

    state.player.mana -= slot.mana_cost as f32;

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
            && rng.random_bool(dodge_chance(
                state.player.eff_initiative(),
                state.enemy.eff_initiative(),
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

    // Route each effect to its proper target based on the effect's nature so a
    // bundled self-buff never benefits the enemy (and vice versa).
    for effect in &ability.effects {
        if effect_targets_self(effect) {
            for &ally in &allies {
                if state.get(ally).is_some() {
                    apply_effect(state, Who::Player, ally, effect);
                }
            }
        } else if !enemy_dodged && state.enemy.alive {
            apply_effect(state, Who::Player, Who::Enemy, effect);
        }
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

    for effect in &consumable.effects {
        // Beneficial effects buff the player; any offensive effect is thrown at
        // the enemy so a consumable never debuffs its own user.
        if effect_targets_self(effect) {
            apply_effect(state, Who::Player, Who::Player, effect);
        } else if state.enemy.alive {
            apply_effect(state, Who::Player, Who::Enemy, effect);
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

    // The host player can dodge offensive effects.
    let has_offensive = ability.effects.iter().any(|e| !effect_targets_self(e));
    let player_dodged = if has_offensive {
        let mut rng = rng();
        state.player.can_dodge()
            && rng.random_bool(dodge_chance(
                state.enemy.eff_initiative(),
                state.player.eff_initiative(),
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
            apply_effect(state, Who::Enemy, Who::Enemy, effect);
        } else if !player_dodged && state.player.alive {
            apply_effect(state, Who::Enemy, Who::Player, effect);
        }
    }

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

    for effect in &consumable.effects {
        if effect_targets_self(effect) {
            apply_effect(state, Who::Enemy, Who::Enemy, effect);
        } else if state.player.alive {
            apply_effect(state, Who::Enemy, Who::Player, effect);
        }
    }

    state.fx.push(CombatFx {
        side: FxSide::Enemy,
        text: "Used!".to_string(),
        color: Color::srgb(0.5, 0.9, 0.6),
    });
    play_audio_msg.write(PlayAudioMsg::new("drink"));
}

pub fn handle_combat_card_click(
    event: On<Pointer<Click>>,
    card_q: Query<&CombatCard>,
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
            try_use_consumable(state, &mut player, &key, &mut play_audio_msg)
        },
    }
}

pub fn combat_input(
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

    let equipped: Vec<String> = consumable_card_order(&player);
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
pub fn consumable_card_order(player: &Player) -> Vec<String> {
    let mut consumables: Vec<(String, u32, String)> = player
        .equipped_consumables
        .iter()
        .filter(|key| player.inventory.iter().any(|inv| inv == *key))
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
    if let Ok(mut text) = text_q.single_mut() {
        let label = combat_speed.label();
        if text.0 != label {
            text.0 = label;
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct CombatTranslationParams<'w, 's> {
    pub name_q: Query<'w, 's, (&'static mut Text, &'static CombatPortraitName), (Without<crate::core::ui::playing::StatLabel>, Without<crate::core::combat::ui::CombatMonsterHealthText>, Without<CombatEndButtonText>)>,
    pub level_q: Query<'w, 's, (&'static mut Text, &'static CombatPortraitLevel), (Without<crate::core::ui::playing::StatLabel>, Without<crate::core::combat::ui::CombatMonsterHealthText>, Without<CombatEndButtonText>, Without<CombatPortraitName>)>,
    pub stat_label_q: Query<'w, 's, (&'static mut Text, &'static CombatStatLabel), (Without<crate::core::ui::playing::StatLabel>, Without<crate::core::combat::ui::CombatMonsterHealthText>, Without<CombatEndButtonText>, Without<CombatPortraitName>, Without<CombatPortraitLevel>)>,
    pub pet_name_q: Query<'w, 's, &'static mut Text, (With<CombatPetName>, Without<crate::core::ui::playing::StatLabel>, Without<crate::core::combat::ui::CombatMonsterHealthText>, Without<CombatEndButtonText>, Without<CombatPortraitName>, Without<CombatPortraitLevel>, Without<CombatStatLabel>)>,
    pub enemy_mana_label_q: Query<'w, 's, &'static mut Text, (With<crate::core::combat::ui::CombatEnemyManaText>, Without<crate::core::ui::playing::StatLabel>, Without<crate::core::combat::ui::CombatMonsterHealthText>, Without<CombatEndButtonText>, Without<CombatPortraitName>, Without<CombatPortraitLevel>, Without<CombatStatLabel>, Without<CombatPetName>)>,
}

pub fn localize_monster_name(
    name: &str,
    kind: crate::core::monsters::MonsterKind,
    localization: &crate::core::localization::Localization,
    lang: crate::core::settings::Language,
) -> String {
    if let Some(loc_name) = localization.get_opt(name, lang) {
        return loc_name;
    }

    if kind == crate::core::monsters::MonsterKind::Dragon {
        let name_cap = crate::utils::capitalize_words(name);
        let mut parts = name_cap.split_whitespace();
        if let Some(color) = parts.next() {
            let stage = parts.collect::<Vec<_>>().join(" ");
            let color_key = format!("general.{}", color.to_lowercase());
            let color_loc = localization.get_opt(&color_key, lang).unwrap_or_else(|| {
                localization.get_opt(color, lang).unwrap_or_else(|| color.to_string())
            });
            let dragon_loc = localization.get_opt("Dragon", lang).unwrap_or_else(|| "Dragon".to_string());
            if stage.is_empty() {
                return format!("{} {}", color_loc, dragon_loc);
            } else {
                let stage_loc = localization.get_opt(&stage, lang).unwrap_or_else(|| stage.to_string());
                return format!("{} {} ({})", color_loc, dragon_loc, stage_loc);
            }
        }
    }

    crate::utils::capitalize_words(name)
}

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
            let pet_name = localization.get_opt(&pet.name, lang).unwrap_or_else(|| crate::utils::capitalize_words(&pet.name));
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
            "{} / {} {}",
            state.enemy.mana.round().max(0.0) as i32,
            state.enemy.max_mana.round() as i32,
            mana_word
        );
    }

    // Ability cooldown / disabled overlays.
    for (overlay, mut node) in &mut overlay_q {
        let frac = state
            .abilities
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
    for (ability, mut image) in &mut ability_image_q {
        let out_of_mana = state
            .abilities
            .get(ability.slot)
            .map(|slot| {
                slot.key.is_some()
                    && slot.remaining <= 0.0
                    && state.player.mana < slot.mana_cost as f32
            })
            .unwrap_or(false);
        image.color = if out_of_mana {
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

pub fn animate_death_skulls(
    time: Res<Time>,
    combat_speed: Res<CombatSpeed>,
    mut commands: Commands,
    state: Option<Res<CombatState>>,
    assets: Res<crate::core::assets::WorldAssets>,
    player_portrait_q: Query<Entity, With<crate::core::combat::ui::CombatPlayerPortrait>>,
    enemy_portrait_q: Query<Entity, With<crate::core::combat::ui::CombatEnemyPortrait>>,
    mut skull_q: Query<(&mut DeathSkullOverlay, &mut Node, &mut ImageNode)>,
) {
    let Some(state) = state else {
        return;
    };
    let dt = time.delta_secs() * combat_speed.0;

    let mut player_skull_exists = false;
    let mut enemy_skull_exists = false;
    for (mut skull, mut node, mut image) in &mut skull_q {
        match skull.side {
            DeathSkullSide::Player => player_skull_exists = true,
            DeathSkullSide::Enemy => enemy_skull_exists = true,
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
                        image: assets.image("skull"),
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
}

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
        let drift = if fct.centered { 4.0 } else { 10.0 };
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
    player: Res<Player>,
    q: Query<(Entity, &ConsumableCardRoot)>,
) {
    if !player.is_changed() {
        return;
    }
    for (entity, card) in &q {
        let available = player.inventory.iter().any(|k| *k == card.0)
            && player.equipped_consumables.iter().any(|k| *k == card.0);
        if !available {
            commands.entity(entity).despawn();
        }
    }
}

/// Handles a click on the bottom combat button (forfeit while ongoing,
/// continue once combat is over).
pub fn handle_combat_end_button_click(
    _event: On<Pointer<Click>>,
    state: Option<Res<CombatState>>,
    duel: Option<Res<DuelActive>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut pending_hunt_xp: ResMut<PendingHuntXp>,
) {
    if let Some(ref s) = state {
        if s.status == CombatStatus::Over {
            if duel.is_none() {
                pending_hunt_xp.amount = s.xp_reward();
            }
        } else if duel.is_some() {
            // During a networked duel, we cannot forfeit/leave combat mid-fight.
            return;
        }
    } else if duel.is_some() {
        return;
    }

    play_audio_msg.write(PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}

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
}

pub fn cleanup_any_combat_artifacts(
    mut commands: Commands,
    combat_q: Query<Entity, With<CombatCmp>>,
    mut combat_menu_suspended: ResMut<CombatMenuSuspended>,
) {
    for entity in &combat_q {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<CombatState>();
    combat_menu_suspended.0 = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fighter_dual_wield_mechanics() {
        let fighter = Fighter {
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
            alive: true,
            weapons: Vec::new(),
        };

        // Test eff_attack_speed_for and attack_period_for with a base speed of 1.2
        let speed = fighter.eff_attack_speed_for(1.2);
        assert_eq!(speed, 1.2);
        let period = fighter.attack_period_for(1.2);
        assert!((period - 1.6666667).abs() < 0.0001);

        // Test eff_attack_for with a base attack of 15.0
        let attack = fighter.eff_attack_for(15.0);
        assert_eq!(attack, 15.0);
    }
}
