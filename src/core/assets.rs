use crate::core::catalog::catalog::{all_abilities, all_consumables, all_perks, all_weapons, all_wearables};
use bevy::asset::AssetServer;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;

/// Forces linear (smooth) filtering for an image, overriding the global
/// `ImagePlugin::default_nearest()` setting. Used for painted/photographic art
/// (item icons, action images) that would otherwise look pixelated when scaled.
fn linear_sampler(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::linear();
}

/// Loads an image with linear filtering (see [`linear_sampler`]).
fn load_linear(assets: &AssetServer, path: impl Into<bevy::asset::AssetPath<'static>>) -> Handle<Image> {
    assets.load_builder().with_settings(linear_sampler).load(path)
}

#[derive(Resource)]
pub struct WorldAssets {
    pub audio: HashMap<&'static str, Handle<AudioSource>>,
    pub fonts: HashMap<&'static str, Handle<Font>>,
    pub images: HashMap<&'static str, Handle<Image>>,
}

impl WorldAssets {
    fn get_asset<'a, T: Clone>(
        &self,
        map: &'a HashMap<&str, T>,
        name: impl Into<String>,
        asset_type: &str,
    ) -> &'a T {
        let name = name.into().clone();
        map.get(name.as_str()).unwrap_or_else(|| panic!("No asset for {asset_type} {name}."))
    }

    pub fn audio(&self, name: impl Into<String>) -> Handle<AudioSource> {
        self.get_asset(&self.audio, name, "audio").clone()
    }

    pub fn font(&self, name: impl Into<String>) -> Handle<Font> {
        self.get_asset(&self.fonts, name, "font").clone()
    }

    pub fn image(&self, name: impl Into<String>) -> Handle<Image> {
        self.get_asset(&self.images, name, "image").clone()
    }
}

impl FromWorld for WorldAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource::<AssetServer>().unwrap();

        let audio = HashMap::from([
            ("music", assets.load("audio/music.ogg")),
            ("message", assets.load("audio/message.ogg")),
            ("warning", assets.load("audio/warning.ogg")),
            ("button", assets.load("audio/button.ogg")),
            ("click", assets.load("audio/click.ogg")),
            ("error", assets.load("audio/error.ogg")),
            ("horn", assets.load("audio/horn.ogg")),
            ("defeat", assets.load("audio/defeat.ogg")),
            ("levelup", assets.load("audio/levelup.ogg")),
            ("inventory", assets.load("audio/inventory.ogg")),
            ("buy", assets.load("audio/buy.ogg")),
            ("sell", assets.load("audio/sell.ogg")),
            ("rest", assets.load("audio/rest.ogg")),
            ("work", assets.load("audio/work.ogg")),
            ("study", assets.load("audio/study.ogg")),
            ("train", assets.load("audio/train.ogg")),
            ("poof", assets.load("audio/poof.ogg")),
        ]);

        let fonts = HashMap::from([
            ("bold", assets.load("fonts/FiraSans-Bold.ttf")),
            ("medium", assets.load("fonts/FiraMono-Medium.ttf")),
        ]);

        let mut images: HashMap<&'static str, Handle<Image>> = HashMap::from([
            // Icons
            ("mute", assets.load("images/icons/mute.ktx2")),
            ("sound", assets.load("images/icons/sound.ktx2")),
            ("music", assets.load("images/icons/music.ktx2")),
            ("defense", assets.load("images/icons/defense.ktx2")),
            ("attack", assets.load("images/icons/attack.ktx2")),
            ("initiative", assets.load("images/icons/initiative.ktx2")),
            ("gold", assets.load("images/icons/gold.ktx2")),
            ("action_hunt", load_linear(assets, "images/icons/action_hunt.ktx2")),
            ("action_shop", load_linear(assets, "images/icons/action_shop.ktx2")),
            ("action_quest", load_linear(assets, "images/icons/action_quest.ktx2")),
            ("action_train", load_linear(assets, "images/icons/action_train.ktx2")),
            ("action_craft", load_linear(assets, "images/icons/action_craft.ktx2")),
            ("action_work", load_linear(assets, "images/icons/action_work.ktx2")),
            ("action_rest", load_linear(assets, "images/icons/action_rest.ktx2")),
            ("action_study", load_linear(assets, "images/icons/action_study.ktx2")),
            ("action_duel", load_linear(assets, "images/icons/action_duel.ktx2")),
            ("ap", assets.load("images/icons/ap.ktx2")),
            ("equipped", assets.load("images/icons/equipped.ktx2")),
            ("base", assets.load("images/icons/base.ktx2")),
            ("ability", assets.load("images/icons/ability.ktx2")),
            ("perk", assets.load("images/icons/perk.ktx2")),
            ("modifier", assets.load("images/icons/modifier.ktx2")),
            ("effect", assets.load("images/icons/effect.ktx2")),
            ("level", assets.load("images/icons/level.ktx2")),
            ("mana", assets.load("images/icons/mana.ktx2")),
            ("cooldown", assets.load("images/icons/cooldown.ktx2")),
            ("fire", assets.load("images/icons/fire.ktx2")),
            ("ice", assets.load("images/icons/ice.ktx2")),
            ("nature", assets.load("images/icons/nature.ktx2")),
            ("holy", assets.load("images/icons/holy.ktx2")),
            ("shadow", assets.load("images/icons/shadow.ktx2")),
            ("physical", assets.load("images/icons/physical.ktx2")),
            ("melee", assets.load("images/icons/melee.ktx2")),
            ("range", assets.load("images/icons/range.ktx2")),
            ("magical", assets.load("images/icons/magical.ktx2")),
            ("finesse", assets.load("images/icons/finesse.ktx2")),
            ("shield", assets.load("images/icons/shield.ktx2")),
            ("book", assets.load("images/icons/book.ktx2")),
            ("aoe", assets.load("images/icons/aoe.ktx2")),
            ("target", assets.load("images/icons/target.ktx2")),
            ("attack_speed", assets.load("images/icons/attack_speed.ktx2")),
            ("crit_chance", assets.load("images/icons/crit_chance.ktx2")),
            ("hand", assets.load("images/icons/hand.ktx2")),
            ("health", assets.load("images/icons/health.ktx2")),
            ("equipment", assets.load("images/icons/equipment.ktx2")),
            ("strength", assets.load("images/icons/strength.ktx2")),
            ("dexterity", assets.load("images/icons/dexterity.ktx2")),
            ("constitution", assets.load("images/icons/constitution.ktx2")),
            ("intelligence", assets.load("images/icons/intelligence.ktx2")),
            ("wisdom", assets.load("images/icons/wisdom.ktx2")),
            ("charisma", assets.load("images/icons/charisma.ktx2")),
            ("training", assets.load("images/icons/training.ktx2")),
            ("assassin", assets.load("images/icons/assassin.ktx2")),
            // Background
            ("bg", assets.load("images/bg/bg.ktx2")),
            ("bg2", assets.load("images/bg/bg2.ktx2")),
            ("basebg", assets.load("images/bg/base.ktx2")),
            ("bg_shop", assets.load("images/bg/shop.ktx2")),
            ("bg_work", assets.load("images/bg/work.ktx2")),
            ("bg_study", assets.load("images/bg/study.ktx2")),
            ("bg_train", assets.load("images/bg/train.ktx2")),
            ("bg_rest", assets.load("images/bg/rest.ktx2")),
            ("defeat", assets.load("images/bg/defeat.ktx2")),
            // UI
            ("border", assets.load("images/ui/border.ktx2")),
            ("border_hover", assets.load("images/ui/border hover.ktx2")),
            ("stone", assets.load("images/ui/stone.ktx2")),
            ("banner", assets.load("images/ui/banner.ktx2")),
            ("banner_large", assets.load("images/ui/banner large.ktx2")),
            // Races
            ("dwarf", assets.load("images/races/dwarf_man.ktx2")),
            ("dwarf_man", assets.load("images/races/dwarf_man.ktx2")),
            ("dwarf_woman", assets.load("images/races/dwarf_woman.ktx2")),
            ("elf", assets.load("images/races/elf_man.ktx2")),
            ("elf_man", assets.load("images/races/elf_man.ktx2")),
            ("elf_woman", assets.load("images/races/elf_woman.ktx2")),
            ("human", assets.load("images/races/human_man.ktx2")),
            ("human_man", assets.load("images/races/human_man.ktx2")),
            ("human_woman", assets.load("images/races/human_woman.ktx2")),
            ("orc", assets.load("images/races/orc_man.ktx2")),
            ("orc_man", assets.load("images/races/orc_man.ktx2")),
            ("orc_woman", assets.load("images/races/orc_woman.ktx2")),
            // Classes
            ("warrior_human_man", assets.load("images/classes/warrior_human_man.ktx2")),
            ("warrior_human_woman", assets.load("images/classes/warrior_human_woman.ktx2")),
            ("warrior_elf_man", assets.load("images/classes/warrior_elf_man.ktx2")),
            ("warrior_elf_woman", assets.load("images/classes/warrior_elf_woman.ktx2")),
            ("warrior_dwarf_man", assets.load("images/classes/warrior_dwarf_man.ktx2")),
            ("warrior_dwarf_woman", assets.load("images/classes/warrior_dwarf_woman.ktx2")),
            ("warrior_orc_man", assets.load("images/classes/warrior_orc_man.ktx2")),
            ("warrior_orc_woman", assets.load("images/classes/warrior_orc_woman.ktx2")),
            ("mage_human_man", assets.load("images/classes/mage_human_man.ktx2")),
            ("mage_human_woman", assets.load("images/classes/mage_human_woman.ktx2")),
            ("mage_elf_man", assets.load("images/classes/mage_elf_man.ktx2")),
            ("mage_elf_woman", assets.load("images/classes/mage_elf_woman.ktx2")),
            ("mage_dwarf_man", assets.load("images/classes/mage_dwarf_man.ktx2")),
            ("mage_dwarf_woman", assets.load("images/classes/mage_dwarf_woman.ktx2")),
            ("mage_orc_man", assets.load("images/classes/mage_orc_man.ktx2")),
            ("mage_orc_woman", assets.load("images/classes/mage_orc_woman.ktx2")),
            ("assassin_human_man", assets.load("images/classes/assassin_human_man.ktx2")),
            ("assassin_human_woman", assets.load("images/classes/assassin_human_woman.ktx2")),
            ("assassin_elf_man", assets.load("images/classes/assassin_elf_man.ktx2")),
            ("assassin_elf_woman", assets.load("images/classes/assassin_elf_woman.ktx2")),
            ("assassin_dwarf_man", assets.load("images/classes/assassin_dwarf_man.ktx2")),
            ("assassin_dwarf_woman", assets.load("images/classes/assassin_dwarf_woman.ktx2")),
            ("assassin_orc_man", assets.load("images/classes/assassin_orc_man.ktx2")),
            ("assassin_orc_woman", assets.load("images/classes/assassin_orc_woman.ktx2")),
            ("druid_human_man", assets.load("images/classes/druid_human_man.ktx2")),
            ("druid_human_woman", assets.load("images/classes/druid_human_woman.ktx2")),
            ("druid_elf_man", assets.load("images/classes/druid_elf_man.ktx2")),
            ("druid_elf_woman", assets.load("images/classes/druid_elf_woman.ktx2")),
            ("druid_dwarf_man", assets.load("images/classes/druid_dwarf_man.ktx2")),
            ("druid_dwarf_woman", assets.load("images/classes/druid_dwarf_woman.ktx2")),
            ("druid_orc_man", assets.load("images/classes/druid_orc_man.ktx2")),
            ("druid_orc_woman", assets.load("images/classes/druid_orc_woman.ktx2")),
            ("mage_black_human_man", assets.load("images/classes/mage_black_human_man.ktx2")),
            ("mage_black_human_woman", assets.load("images/classes/mage_black_human_woman.ktx2")),
            ("mage_black_elf_man", assets.load("images/classes/mage_black_elf_man.ktx2")),
            ("mage_black_elf_woman", assets.load("images/classes/mage_black_elf_woman.ktx2")),
            ("mage_black_dwarf_man", assets.load("images/classes/mage_black_dwarf_man.ktx2")),
            ("mage_black_dwarf_woman", assets.load("images/classes/mage_black_dwarf_woman.ktx2")),
            ("mage_black_orc_man", assets.load("images/classes/mage_black_orc_man.ktx2")),
            ("mage_black_orc_woman", assets.load("images/classes/mage_black_orc_woman.ktx2")),
            ("mage_red_human_man", assets.load("images/classes/mage_red_man.ktx2")),
            ("mage_red_human_woman", assets.load("images/classes/mage_red_human_woman.ktx2")),
            ("mage_red_elf_man", assets.load("images/classes/mage_red_elf_man.ktx2")),
            ("mage_red_elf_woman", assets.load("images/classes/mage_red_elf_woman.ktx2")),
            ("mage_red_dwarf_man", assets.load("images/classes/mage_red_dwarf_man.ktx2")),
            ("mage_red_dwarf_woman", assets.load("images/classes/mage_red_dwarf_woman.ktx2")),
            ("mage_red_orc_man", assets.load("images/classes/mage_red_orc_man.ktx2")),
            ("mage_red_orc_woman", assets.load("images/classes/mage_red_orc_woman.ktx2")),
            ("mage_green_human_man", assets.load("images/classes/mage_green_human_man.ktx2")),
            ("mage_green_human_woman", assets.load("images/classes/mage_green_human_woman.ktx2")),
            ("mage_green_elf_man", assets.load("images/classes/mage_green_elf_man.ktx2")),
            ("mage_green_elf_woman", assets.load("images/classes/mage_green_elf_woman.ktx2")),
            ("mage_green_dwarf_man", assets.load("images/classes/mage_green_dwarf_man.ktx2")),
            ("mage_green_dwarf_woman", assets.load("images/classes/mage_green_dwarf_woman.ktx2")),
            ("mage_green_orc_man", assets.load("images/classes/mage_green_orc_man.ktx2")),
            ("mage_green_orc_woman", assets.load("images/classes/mage_green_orc_woman.ktx2")),
            ("mage_white_human_man", assets.load("images/classes/mage_white_man.ktx2")),
            ("mage_white_human_woman", assets.load("images/classes/mage_white_human_woman.ktx2")),
            ("mage_white_elf_man", assets.load("images/classes/mage_white_elf_man.ktx2")),
            ("mage_white_elf_woman", assets.load("images/classes/mage_white_elf_woman.ktx2")),
            ("mage_white_dwarf_man", assets.load("images/classes/mage_white_dwarf_man.ktx2")),
            ("mage_white_dwarf_woman", assets.load("images/classes/mage_white_dwarf_woman.ktx2")),
            ("mage_white_orc_man", assets.load("images/classes/mage_white_orc_man.ktx2")),
            ("mage_white_orc_woman", assets.load("images/classes/mage_white_orc_woman.ktx2")),
            // Pets
            ("wolf", assets.load("images/pets/wolf.ktx2")),
            ("snake", assets.load("images/pets/snake.ktx2")),
            ("eagle", assets.load("images/pets/eagle.ktx2")),
            ("bear", assets.load("images/pets/bear.ktx2")),
            ("bat", assets.load("images/pets/bat.ktx2")),
            ("crocodile", assets.load("images/pets/crocodile.ktx2")),
            ("griffin", assets.load("images/pets/griffin.ktx2")),
            ("hyena", assets.load("images/pets/hyena.ktx2")),
            ("infernal can", assets.load("images/pets/infernal can.ktx2")),
            ("lizard", assets.load("images/pets/lizard.ktx2")),
            ("manticore", assets.load("images/pets/manticore.ktx2")),
            ("pegasus", assets.load("images/pets/pegasus.ktx2")),
            ("puma", assets.load("images/pets/puma.ktx2")),
            ("rat", assets.load("images/pets/rat.ktx2")),
            ("spider", assets.load("images/pets/spider.ktx2")),
            ("three headed dog", assets.load("images/pets/three headed dog.ktx2")),
            ("tiger", assets.load("images/pets/tiger.ktx2")),
            ("unicorn", assets.load("images/pets/unicorn.ktx2")),
            ("vulture", assets.load("images/pets/vulture.ktx2")),
            // Actions
            ("action_clerical_labor", load_linear(assets, "images/actions/clerical_labor.ktx2")),
            ("action_craft_labor", load_linear(assets, "images/actions/craft_labor.ktx2")),
            ("action_manual_labor", load_linear(assets, "images/actions/manual_labor.ktx2")),
            ("action_apprenticeship", load_linear(assets, "images/actions/apprenticeship.ktx2")),
            ("action_mentorship", load_linear(assets, "images/actions/mentorship.ktx2")),
            ("action_conditioning", load_linear(assets, "images/actions/conditioning.ktx2")),
            ("action_simple_rest", load_linear(assets, "images/actions/simple_rest.ktx2")),
            ("action_common_lodging", load_linear(assets, "images/actions/common_lodging.ktx2")),
            ("action_grand_accommodation", load_linear(assets, "images/actions/grand_accomodation.ktx2")),
            ("action_melee", load_linear(assets, "images/actions/melee.ktx2")),
            ("action_range", load_linear(assets, "images/actions/range.ktx2")),
            ("action_finesse", load_linear(assets, "images/actions/finesse.ktx2")),
        ]);

        for ability in all_abilities() {
            let key: &'static str = Box::leak(format!("build_{}", ability.name).into_boxed_str());
            images.insert(key, load_linear(assets, ability.image.clone()));
        }
        for perk in all_perks() {
            let key: &'static str = Box::leak(format!("build_{}", perk.name).into_boxed_str());
            images.insert(key, load_linear(assets, perk.image.clone()));
        }
        for weapon in all_weapons() {
            let key: &'static str = Box::leak(format!("build_{}", weapon.name).into_boxed_str());
            images.insert(key, load_linear(assets, weapon.image.clone()));
        }
        for wearable in all_wearables() {
            let key: &'static str = Box::leak(format!("build_{}", wearable.name).into_boxed_str());
            images.insert(key, load_linear(assets, wearable.image.clone()));
        }
        for consumable in all_consumables() {
            let key: &'static str = Box::leak(format!("build_{}", consumable.name).into_boxed_str());
            images.insert(key, load_linear(assets, consumable.image.clone()));
        }

        Self {
            audio,
            fonts,
            images,
        }
    }
}
