//! Loading and lookup of the images, fonts, audio, and catalogs used at runtime.

use crate::core::catalog::catalog::*;
use bevy::asset::AssetServer;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// Forces linear (smooth) filtering for an image, overriding the global
/// `ImagePlugin::default_nearest()` setting. Used for painted/photographic art
/// (item icons, action images) that would otherwise look pixelated when scaled.
fn linear_sampler(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::linear();
}

/// Loads an image with linear filtering (see [`linear_sampler`]).
fn load_linear(
    assets: &AssetServer,
    path: impl Into<bevy::asset::AssetPath<'static>>,
) -> Handle<Image> {
    assets.load_builder().with_settings(linear_sampler).load(path)
}

/// Performs the leak str operation.
fn leak_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

/// Inserts image aliases.
fn insert_image_aliases(
    image_paths: &mut HashMap<&'static str, String>,
    image_path: &str,
    aliases: impl IntoIterator<Item = String>,
) {
    for alias in aliases {
        image_paths.entry(leak_str(alias)).or_insert_with(|| image_path.to_string());
    }
}

/// Performs the catalog image aliases operation.
fn catalog_image_aliases(image: &str) -> Vec<String> {
    vec![image.to_string()]
}

/// Returns runtime keys and paths for expanded playable portraits and deity art.
fn expansion_portrait_paths() -> Vec<(String, String)> {
    let mut portraits =
        vec![("halfling".to_string(), "images/races/halfling_man.webp".to_string())];

    for mutation in ["werewolf", "wererat", "werebear", "vampire", "undead"] {
        portraits.push((mutation.to_string(), format!("images/races/{mutation}.webp")));
        portraits
            .push((format!("{mutation}_woman"), format!("images/races/{mutation}_woman.webp")));
        for race in ["elf", "dwarf", "orc", "halfling", "dragonborn"] {
            for sex in ["man", "woman"] {
                let key = format!("{mutation}_{race}_{sex}");
                portraits.push((key.clone(), format!("images/races/{key}.webp")));
            }
        }
    }

    for sex in ["man", "woman"] {
        portraits.push((format!("halfling_{sex}"), format!("images/races/halfling_{sex}.webp")));
    }

    for heritage in ["high", "dark", "wood"] {
        for sex in ["man", "woman"] {
            let key = format!("elf_{heritage}_{sex}");
            portraits.push((key.clone(), format!("images/races/{key}.webp")));
        }
    }

    for class in [
        "warrior",
        "assassin",
        "druid",
        "mage",
        "mage_black",
        "mage_red",
        "mage_green",
        "mage_white",
    ] {
        for sex in ["man", "woman"] {
            let key = format!("{class}_halfling_{sex}");
            portraits.push((key.clone(), format!("images/classes/{key}.webp")));
        }
    }

    for race in ["human", "elf", "dwarf", "orc", "halfling"] {
        for sex in ["man", "woman"] {
            let key = format!("monk_{race}_{sex}");
            portraits.push((key.clone(), format!("images/classes/{key}.webp")));
        }
    }

    for school in ["open_hand", "iron_body", "shadow_step", "spirit_fist"] {
        for race in ["human", "elf", "dwarf", "orc", "halfling", "dragonborn"] {
            for sex in ["man", "woman"] {
                let key = format!("monk_{school}_{race}_{sex}");
                portraits.push((key.clone(), format!("images/classes/{key}.webp")));
            }
        }
    }

    for (class, specializations) in [
        ("assassin", ["nightblade", "venomhand", "duelist", "phantom"]),
        ("bard", ["war_chant", "silver_ballad", "grave_dirge", "wild_rhythm"]),
    ] {
        for specialization in specializations {
            for race in ["human", "elf", "dwarf", "orc", "halfling", "dragonborn"] {
                for sex in ["man", "woman"] {
                    let key = format!("{class}_{specialization}_{race}_{sex}");
                    portraits.push((key.clone(), format!("images/classes/{key}.webp")));
                }
            }
        }
    }

    for specialization in ["paladin", "templar", "berserker", "warden"] {
        for race in ["human", "elf", "dwarf", "orc", "halfling", "dragonborn"] {
            for sex in ["man", "woman"] {
                let key = format!("warrior_{specialization}_{race}_{sex}");
                portraits.push((key.clone(), format!("images/classes/{key}.webp")));
            }
        }
    }

    portraits.push(("dragonborn".to_string(), "images/races/dragonborn_man.webp".to_string()));
    for sex in ["man", "woman"] {
        portraits
            .push((format!("dragonborn_{sex}"), format!("images/races/dragonborn_{sex}.webp")));
        for class in [
            "warrior",
            "assassin",
            "druid",
            "mage",
            "mage_black",
            "mage_red",
            "mage_green",
            "mage_white",
            "monk",
            "bard",
        ] {
            let key = format!("{class}_dragonborn_{sex}");
            portraits.push((key.clone(), format!("images/classes/{key}.webp")));
        }
    }

    for race in ["human", "dwarf", "orc", "halfling"] {
        for sex in ["man", "woman"] {
            portraits
                .push((format!("bard_{race}_{sex}"), format!("images/classes/bard_{sex}.webp")));
        }
    }
    for sex in ["man", "woman"] {
        let key = format!("bard_elf_{sex}");
        portraits.push((key.clone(), format!("images/classes/{key}.webp")));
    }
    for deity in
        ["aeloria", "serapha", "aurion", "vaelis", "tharos", "oryn", "kharos", "nyxara", "vhal"]
    {
        portraits.push((format!("deity_{deity}"), format!("images/deities/{deity}.webp")));
    }

    portraits
}

const COMBAT_IMAGE_PATHS: [(&str, &str); 5] = [
    ("combat_guard", "images/combat/guard.webp"),
    ("combat_stance_aggressive", "images/combat/stance_aggressive.webp"),
    ("combat_stance_defensive", "images/combat/stance_defensive.webp"),
    ("combat_stance_precise", "images/combat/stance_precise.webp"),
    ("combat_stance_disruptive", "images/combat/stance_disruptive.webp"),
];

/// Records image paths without starting an asset request.
struct DeferredImagePaths;

impl DeferredImagePaths {
    /// Returns an owned path for the deferred image registry.
    fn load(&self, path: impl Into<String>) -> String {
        path.into()
    }
}

#[derive(Resource)]
pub struct WorldAssets {
    pub audio: HashMap<&'static str, Handle<AudioSource>>,
    pub fonts: HashMap<&'static str, Handle<Font>>,
    asset_server: AssetServer,
    image_paths: HashMap<&'static str, String>,
    lazy_images: Mutex<HashMap<String, Handle<Image>>>,
}

impl WorldAssets {
    /// Returns asset.
    fn get_asset<'a, T: Clone>(
        &self,
        map: &'a HashMap<&str, T>,
        name: impl Into<String>,
        asset_type: &str,
    ) -> &'a T {
        let name = name.into().clone();
        map.get(name.as_str()).unwrap_or_else(|| panic!("No asset for {asset_type} {name}."))
    }

    /// Performs the audio operation.
    pub fn audio(&self, name: impl Into<String>) -> Handle<AudioSource> {
        self.get_asset(&self.audio, name, "audio").clone()
    }

    /// Performs the font operation.
    pub fn font(&self, name: impl Into<String>) -> Handle<Font> {
        self.get_asset(&self.fonts, name, "font").clone()
    }

    /// Performs the image operation.
    pub fn image(&self, name: impl Into<String>) -> Handle<Image> {
        let name = name.into();
        let path = self
            .image_paths
            .get(name.as_str())
            .unwrap_or_else(|| panic!("No asset for image {name}."));
        let mut lazy_images = self.lazy_images.lock().expect("lazy image cache poisoned");
        lazy_images
            .entry(path.clone())
            .or_insert_with(|| load_linear(&self.asset_server, path.clone()))
            .clone()
    }
}

impl FromWorld for WorldAssets {
    /// Performs the from world operation.
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap().clone();
        let assets = &asset_server;

        let audio = HashMap::from([
            ("music", assets.load("audio/music.ogg")),
            ("message", assets.load("audio/message.ogg")),
            ("warning", assets.load("audio/warning.ogg")),
            ("button", assets.load("audio/button.ogg")),
            ("click", assets.load("audio/click.ogg")),
            ("error", assets.load("audio/error.ogg")),
            ("horn", assets.load("audio/horn.ogg")),
            ("victory", assets.load("audio/victory.ogg")),
            ("defeat", assets.load("audio/defeat.ogg")),
            ("levelup", assets.load("audio/levelup.ogg")),
            ("inventory", assets.load("audio/inventory.ogg")),
            ("cast", assets.load("audio/cast.ogg")),
            ("holy", assets.load("audio/holy.ogg")),
            ("drink", assets.load("audio/drink.ogg")),
            ("arrow_swish", assets.load("audio/arrow swish.ogg")),
            ("arrow_impact", assets.load("audio/arrow impact.ogg")),
            ("armor_impact", assets.load("audio/armor impact.ogg")),
            ("sword_clash", assets.load("audio/sword clash.ogg")),
            ("sword_slice", assets.load("audio/sword slice.ogg")),
            ("sword_slice_2", assets.load("audio/sword slice 2.ogg")),
            ("sword_slice_3", assets.load("audio/sword slice 3.ogg")),
            ("sword_slice_violent", assets.load("audio/sword slice violent.ogg")),
            ("buy", assets.load("audio/buy.ogg")),
            ("sell", assets.load("audio/sell.ogg")),
            ("rest", assets.load("audio/rest.ogg")),
            ("work", assets.load("audio/work.ogg")),
            ("study", assets.load("audio/study.ogg")),
            ("train", assets.load("audio/train.ogg")),
            ("craft", assets.load("audio/craft.ogg")),
            ("hunt", assets.load("audio/hunt.ogg")),
            ("quest", assets.load("audio/quest.ogg")),
            ("poof", assets.load("audio/poof.ogg")),
            ("curse", assets.load("audio/curse.ogg")),
        ]);

        let fonts = HashMap::from([
            ("bold", assets.load("fonts/FiraSans-Bold.ttf")),
            ("medium", assets.load("fonts/FiraMono-Medium.ttf")),
        ]);

        // Keep the registration table path-only. Calling `AssetServer::load` for
        // every portrait here decoded hundreds of unused textures during web startup.
        let deferred_images = DeferredImagePaths;
        let assets = &deferred_images;
        let load_linear = |_: &DeferredImagePaths, path: &str| path.to_string();
        let mut image_paths: HashMap<&'static str, String> = HashMap::from([
            // Icons
            ("mute", assets.load("images/icons/mute.webp")),
            ("sound", assets.load("images/icons/sound.webp")),
            ("music", assets.load("images/icons/music.webp")),
            ("defense", assets.load("images/icons/defense.webp")),
            ("attack", assets.load("images/icons/attack.webp")),
            ("initiative", assets.load("images/icons/initiative.webp")),
            ("gold", assets.load("images/icons/gold.webp")),
            ("skull", assets.load("images/icons/skull.webp")),
            ("capture", assets.load("images/icons/capture.webp")),
            ("action_hunt", load_linear(assets, "images/icons/action_hunt.webp")),
            ("action_shop", load_linear(assets, "images/icons/action_shop.webp")),
            ("action_quest", load_linear(assets, "images/icons/action_quest.webp")),
            ("action_train", load_linear(assets, "images/icons/action_train.webp")),
            ("action_craft", load_linear(assets, "images/icons/action_craft.webp")),
            ("action_work", load_linear(assets, "images/icons/action_work.webp")),
            ("action_rest", load_linear(assets, "images/icons/action_rest.webp")),
            ("action_study", load_linear(assets, "images/icons/action_study.webp")),
            ("action_duel", load_linear(assets, "images/icons/action_duel.webp")),
            ("ap", assets.load("images/icons/ap.webp")),
            ("equipped", assets.load("images/icons/equipped.webp")),
            ("base", assets.load("images/icons/base.webp")),
            ("ability", assets.load("images/icons/ability.webp")),
            ("perk", assets.load("images/icons/perk.webp")),
            ("modifier", assets.load("images/icons/modifier.webp")),
            ("effect", assets.load("images/icons/effect.webp")),
            ("level", assets.load("images/icons/level.webp")),
            ("mana", assets.load("images/icons/mana.webp")),
            ("cooldown", assets.load("images/icons/cooldown.webp")),
            ("fire", assets.load("images/icons/fire.webp")),
            ("ice", assets.load("images/icons/ice.webp")),
            ("nature", assets.load("images/icons/nature.webp")),
            ("holy", assets.load("images/icons/holy.webp")),
            ("shadow", assets.load("images/icons/shadow.webp")),
            ("physical", assets.load("images/icons/physical.webp")),
            ("melee", assets.load("images/icons/melee.webp")),
            ("range", assets.load("images/icons/range.webp")),
            ("magical", assets.load("images/icons/magical.webp")),
            ("finesse", assets.load("images/icons/finesse.webp")),
            ("shield", assets.load("images/icons/shield.webp")),
            ("book", assets.load("images/icons/book.webp")),
            ("aoe", assets.load("images/icons/aoe.webp")),
            ("target", assets.load("images/icons/target.webp")),
            ("attack_speed", assets.load("images/icons/attack_speed.webp")),
            ("crit_chance", assets.load("images/icons/crit_chance.webp")),
            ("hand", assets.load("images/icons/hand.webp")),
            ("health", assets.load("images/icons/health.webp")),
            ("equipment", assets.load("images/icons/equipment.webp")),
            ("strength", assets.load("images/icons/strength.webp")),
            ("dexterity", assets.load("images/icons/dexterity.webp")),
            ("constitution", assets.load("images/icons/constitution.webp")),
            ("intelligence", assets.load("images/icons/intelligence.webp")),
            ("wisdom", assets.load("images/icons/wisdom.webp")),
            ("charisma", assets.load("images/icons/charisma.webp")),
            ("training", assets.load("images/icons/training.webp")),
            ("race", assets.load("images/icons/race.webp")),
            ("class", assets.load("images/icons/class.webp")),
            ("deity", assets.load("images/icons/deity.webp")),
            ("assassin", assets.load("images/icons/assassin.webp")),
            // Effects
            ("blind", assets.load("images/icons/blind.webp")),
            ("burn", assets.load("images/icons/burn.webp")),
            ("curse", assets.load("images/icons/curse.webp")),
            ("freeze", assets.load("images/icons/freeze.webp")),
            ("immobilize", assets.load("images/icons/immobilize.webp")),
            ("poison", assets.load("images/icons/poison.webp")),
            ("paranoia", assets.load("images/icons/paranoia.webp")),
            ("silence", assets.load("images/icons/silence.webp")),
            ("stun", assets.load("images/icons/stun.webp")),
            ("taunt", assets.load("images/icons/taunt.webp")),
            ("vulnerability", assets.load("images/icons/vulnerability.webp")),
            // Background
            ("bg", assets.load("images/bg/bg.webp")),
            ("bg2", assets.load("images/bg/bg2.webp")),
            ("basebg", assets.load("images/bg/base.webp")),
            ("bg_shop", assets.load("images/bg/shop.webp")),
            ("bg_rest", assets.load("images/bg/rest.webp")),
            ("bg_study", assets.load("images/bg/study.webp")),
            ("bg_work", assets.load("images/bg/work.webp")),
            ("bg_train", assets.load("images/bg/train.webp")),
            ("bg_craft", assets.load("images/bg/craft.webp")),
            ("bg_hunt", assets.load("images/bg/hunt.webp")),
            ("bg_quest", assets.load("images/bg/quest.webp")),
            ("bg_duel", assets.load("images/bg/duel.webp")),
            ("bg_combat", assets.load("images/bg/combat.webp")),
            ("defeat", assets.load("images/bg/defeat.webp")),
            ("bg_mutation", assets.load("images/bg/mutation.webp")),
            // UI
            ("border", assets.load("images/ui/border.webp")),
            ("border_hover", assets.load("images/ui/border hover.webp")),
            ("stone", assets.load("images/ui/stone.webp")),
            ("banner", assets.load("images/ui/banner.webp")),
            ("banner_large", assets.load("images/ui/banner large.webp")),
            // Races
            ("dwarf", assets.load("images/races/dwarf_man.webp")),
            ("dwarf_man", assets.load("images/races/dwarf_man.webp")),
            ("dwarf_woman", assets.load("images/races/dwarf_woman.webp")),
            ("elf", assets.load("images/races/elf_man.webp")),
            ("elf_man", assets.load("images/races/elf_man.webp")),
            ("elf_woman", assets.load("images/races/elf_woman.webp")),
            ("human", assets.load("images/races/human_man.webp")),
            ("human_man", assets.load("images/races/human_man.webp")),
            ("human_woman", assets.load("images/races/human_woman.webp")),
            ("orc", assets.load("images/races/orc_man.webp")),
            ("orc_man", assets.load("images/races/orc_man.webp")),
            ("orc_woman", assets.load("images/races/orc_woman.webp")),
            // Classes
            ("warrior_human_man", assets.load("images/classes/warrior_human_man.webp")),
            ("warrior_human_woman", assets.load("images/classes/warrior_human_woman.webp")),
            ("warrior_elf_man", assets.load("images/classes/warrior_elf_man.webp")),
            ("warrior_elf_woman", assets.load("images/classes/warrior_elf_woman.webp")),
            ("warrior_dwarf_man", assets.load("images/classes/warrior_dwarf_man.webp")),
            ("warrior_dwarf_woman", assets.load("images/classes/warrior_dwarf_woman.webp")),
            ("warrior_orc_man", assets.load("images/classes/warrior_orc_man.webp")),
            ("warrior_orc_woman", assets.load("images/classes/warrior_orc_woman.webp")),
            ("mage_human_man", assets.load("images/classes/mage_human_man.webp")),
            ("mage_human_woman", assets.load("images/classes/mage_human_woman.webp")),
            ("mage_elf_man", assets.load("images/classes/mage_elf_man.webp")),
            ("mage_elf_woman", assets.load("images/classes/mage_elf_woman.webp")),
            ("mage_dwarf_man", assets.load("images/classes/mage_dwarf_man.webp")),
            ("mage_dwarf_woman", assets.load("images/classes/mage_dwarf_woman.webp")),
            ("mage_orc_man", assets.load("images/classes/mage_orc_man.webp")),
            ("mage_orc_woman", assets.load("images/classes/mage_orc_woman.webp")),
            ("assassin_human_man", assets.load("images/classes/assassin_human_man.webp")),
            ("assassin_human_woman", assets.load("images/classes/assassin_human_woman.webp")),
            ("assassin_elf_man", assets.load("images/classes/assassin_elf_man.webp")),
            ("assassin_elf_woman", assets.load("images/classes/assassin_elf_woman.webp")),
            ("assassin_dwarf_man", assets.load("images/classes/assassin_dwarf_man.webp")),
            ("assassin_dwarf_woman", assets.load("images/classes/assassin_dwarf_woman.webp")),
            ("assassin_orc_man", assets.load("images/classes/assassin_orc_man.webp")),
            ("assassin_orc_woman", assets.load("images/classes/assassin_orc_woman.webp")),
            ("druid_human_man", assets.load("images/classes/druid_human_man.webp")),
            ("druid_human_woman", assets.load("images/classes/druid_human_woman.webp")),
            ("druid_elf_man", assets.load("images/classes/druid_elf_man.webp")),
            ("druid_elf_woman", assets.load("images/classes/druid_elf_woman.webp")),
            ("druid_dwarf_man", assets.load("images/classes/druid_dwarf_man.webp")),
            ("druid_dwarf_woman", assets.load("images/classes/druid_dwarf_woman.webp")),
            ("druid_orc_man", assets.load("images/classes/druid_orc_man.webp")),
            ("druid_orc_woman", assets.load("images/classes/druid_orc_woman.webp")),
            ("mage_black_human_man", assets.load("images/classes/mage_black_human_man.webp")),
            ("mage_black_human_woman", assets.load("images/classes/mage_black_human_woman.webp")),
            ("mage_black_elf_man", assets.load("images/classes/mage_black_elf_man.webp")),
            ("mage_black_elf_woman", assets.load("images/classes/mage_black_elf_woman.webp")),
            ("mage_black_dwarf_man", assets.load("images/classes/mage_black_dwarf_man.webp")),
            ("mage_black_dwarf_woman", assets.load("images/classes/mage_black_dwarf_woman.webp")),
            ("mage_black_orc_man", assets.load("images/classes/mage_black_orc_man.webp")),
            ("mage_black_orc_woman", assets.load("images/classes/mage_black_orc_woman.webp")),
            ("mage_red_human_man", assets.load("images/classes/mage_red_man.webp")),
            ("mage_red_human_woman", assets.load("images/classes/mage_red_human_woman.webp")),
            ("mage_red_elf_man", assets.load("images/classes/mage_red_elf_man.webp")),
            ("mage_red_elf_woman", assets.load("images/classes/mage_red_elf_woman.webp")),
            ("mage_red_dwarf_man", assets.load("images/classes/mage_red_dwarf_man.webp")),
            ("mage_red_dwarf_woman", assets.load("images/classes/mage_red_dwarf_woman.webp")),
            ("mage_red_orc_man", assets.load("images/classes/mage_red_orc_man.webp")),
            ("mage_red_orc_woman", assets.load("images/classes/mage_red_orc_woman.webp")),
            ("mage_green_human_man", assets.load("images/classes/mage_green_human_man.webp")),
            ("mage_green_human_woman", assets.load("images/classes/mage_green_human_woman.webp")),
            ("mage_green_elf_man", assets.load("images/classes/mage_green_elf_man.webp")),
            ("mage_green_elf_woman", assets.load("images/classes/mage_green_elf_woman.webp")),
            ("mage_green_dwarf_man", assets.load("images/classes/mage_green_dwarf_man.webp")),
            ("mage_green_dwarf_woman", assets.load("images/classes/mage_green_dwarf_woman.webp")),
            ("mage_green_orc_man", assets.load("images/classes/mage_green_orc_man.webp")),
            ("mage_green_orc_woman", assets.load("images/classes/mage_green_orc_woman.webp")),
            ("mage_white_human_man", assets.load("images/classes/mage_white_man.webp")),
            ("mage_white_human_woman", assets.load("images/classes/mage_white_human_woman.webp")),
            ("mage_white_elf_man", assets.load("images/classes/mage_white_elf_man.webp")),
            ("mage_white_elf_woman", assets.load("images/classes/mage_white_elf_woman.webp")),
            ("mage_white_dwarf_man", assets.load("images/classes/mage_white_dwarf_man.webp")),
            ("mage_white_dwarf_woman", assets.load("images/classes/mage_white_dwarf_woman.webp")),
            ("mage_white_orc_man", assets.load("images/classes/mage_white_orc_man.webp")),
            ("mage_white_orc_woman", assets.load("images/classes/mage_white_orc_woman.webp")),
            // Actions
            ("action_clerical_labor", load_linear(assets, "images/actions/clerical_labor.webp")),
            ("action_craft_labor", load_linear(assets, "images/actions/craft_labor.webp")),
            ("action_manual_labor", load_linear(assets, "images/actions/manual_labor.webp")),
            ("action_apprenticeship", load_linear(assets, "images/actions/apprenticeship.webp")),
            ("action_mentorship", load_linear(assets, "images/actions/mentorship.webp")),
            ("action_conditioning", load_linear(assets, "images/actions/conditioning.webp")),
            ("action_simple_rest", load_linear(assets, "images/actions/simple_rest.webp")),
            ("action_common_lodging", load_linear(assets, "images/actions/common_lodging.webp")),
            (
                "action_grand_accommodation",
                load_linear(assets, "images/actions/grand_accomodation.webp"),
            ),
            ("action_melee", load_linear(assets, "images/actions/melee.webp")),
            ("action_range", load_linear(assets, "images/actions/range.webp")),
            ("action_finesse", load_linear(assets, "images/actions/finesse.webp")),
            ("action_easy_hunt", load_linear(assets, "images/actions/easy_hunt.webp")),
            ("action_wild_hunt", load_linear(assets, "images/actions/wild_hunt.webp")),
            ("action_deadly_hunt", load_linear(assets, "images/actions/deadly_hunt.webp")),
            ("action_errand", load_linear(assets, "images/actions/errand.webp")),
            ("action_expedition", load_linear(assets, "images/actions/expedition.webp")),
            ("action_odyssey", load_linear(assets, "images/actions/odyssey.webp")),
        ]);

        for (key, path) in COMBAT_IMAGE_PATHS {
            image_paths.insert(key, load_linear(assets, path));
        }

        for (key, path) in expansion_portrait_paths() {
            image_paths.insert(leak_str(key), assets.load(path));
        }

        for ability in all_abilities() {
            insert_image_aliases(
                &mut image_paths,
                &ability.image,
                catalog_image_aliases(&ability.image),
            );
        }

        for perk in all_perks() {
            insert_image_aliases(&mut image_paths, &perk.image, catalog_image_aliases(&perk.image));
        }

        for weapon in all_weapons() {
            insert_image_aliases(
                &mut image_paths,
                &weapon.image,
                catalog_image_aliases(&weapon.image),
            );
        }

        for wearable in all_wearables() {
            insert_image_aliases(
                &mut image_paths,
                &wearable.image,
                catalog_image_aliases(&wearable.image),
            );
        }

        for consumable in all_consumables() {
            insert_image_aliases(
                &mut image_paths,
                &consumable.image,
                catalog_image_aliases(&consumable.image),
            );
        }

        for artifact in all_artifacts() {
            insert_image_aliases(
                &mut image_paths,
                &artifact.image,
                catalog_image_aliases(&artifact.image),
            );
        }

        for monster in all_monsters() {
            insert_image_aliases(
                &mut image_paths,
                &monster.image,
                [
                    monster.name.to_lowercase(),
                    monster.name.to_lowercase().replace(" ", "_"),
                    monster.image.clone(),
                    Path::new(&monster.image)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| monster.image.clone()),
                ],
            );
        }

        Self {
            audio,
            fonts,
            asset_server,
            image_paths,
            lazy_images: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{catalog_image_aliases, expansion_portrait_paths};
    use crate::core::catalog::catalog::{all_perks, all_weapons};
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    /// Verifies catalog image aliases preserve the catalog image path.
    fn catalog_image_aliases_include_raw_path() {
        let image = "images/catalog/consumable/Alchemy_40_poisonousherbs.webp";

        assert_eq!(catalog_image_aliases(image), [image]);
    }

    #[test]
    /// Verifies catalog entries with the same display name retain distinct image paths.
    fn duplicate_catalog_names_keep_distinct_images() {
        let perk =
            all_perks().iter().find(|perk| perk.name == "Dagger").expect("Dagger perk must exist");
        let weapon = all_weapons()
            .iter()
            .find(|weapon| weapon.name == "Dagger")
            .expect("Dagger weapon must exist");

        assert_ne!(perk.image, weapon.image);
        assert_eq!(catalog_image_aliases(&perk.image), vec![perk.image.clone()]);
        assert_eq!(catalog_image_aliases(&weapon.image), vec![weapon.image.clone()]);
    }

    #[test]
    /// Verifies that every expanded playable portrait has a unique runtime key and built asset.
    fn expansion_portraits_have_unique_keys_and_built_assets() {
        let portraits = expansion_portrait_paths();
        let unique_keys = portraits.iter().map(|(key, _)| key).collect::<HashSet<_>>();
        assert_eq!(unique_keys.len(), portraits.len());

        for (_, path) in portraits {
            assert!(Path::new("assets").join(&path).is_file(), "missing portrait: {path}");
        }
    }

    #[test]
    /// Verifies requested heritage and specialized character keys use dedicated files.
    fn expanded_identity_portraits_are_not_aliases() {
        for (key, path) in expansion_portrait_paths() {
            let is_heritage = key.starts_with("elf_high_")
                || key.starts_with("elf_dark_")
                || key.starts_with("elf_wood_");
            let is_monk_school = key.starts_with("monk_open_hand_")
                || key.starts_with("monk_iron_body_")
                || key.starts_with("monk_shadow_step_")
                || key.starts_with("monk_spirit_fist_");
            let is_assassin_path = key.starts_with("assassin_nightblade_")
                || key.starts_with("assassin_venomhand_")
                || key.starts_with("assassin_duelist_")
                || key.starts_with("assassin_phantom_");
            let is_bard_style = key.starts_with("bard_war_chant_")
                || key.starts_with("bard_silver_ballad_")
                || key.starts_with("bard_grave_dirge_")
                || key.starts_with("bard_wild_rhythm_");
            let is_warrior_calling = key.starts_with("warrior_paladin_")
                || key.starts_with("warrior_templar_")
                || key.starts_with("warrior_berserker_")
                || key.starts_with("warrior_warden_");
            let is_elf_bard = key.starts_with("bard_elf_");
            let is_dragonborn_class = key.contains("_dragonborn_");
            if is_heritage
                || is_monk_school
                || is_assassin_path
                || is_bard_style
                || is_warrior_calling
                || is_elf_bard
                || is_dragonborn_class
            {
                assert_eq!(
                    Path::new(&path).file_stem().and_then(|stem| stem.to_str()),
                    Some(key.as_str())
                );
            }
        }
    }
}
