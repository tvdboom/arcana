use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;
use crate::core::catalog::{all_abilities, all_perks, all_weapons, all_wearables};

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
            ("coins", assets.load("audio/coins.ogg")),
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
            ("mute", assets.load("images/icons/mute.png")),
            ("sound", assets.load("images/icons/sound.png")),
            ("music", assets.load("images/icons/music.png")),
            ("defense", assets.load("images/icons/defense.png")),
            ("attack", assets.load("images/icons/attack.png")),
            ("initiative", assets.load("images/icons/initiative.png")),
            ("gold", assets.load("images/icons/gold.png")),
            ("action_hunt", assets.load("images/icons/action_hunt.png")),
            ("action_shop", assets.load("images/icons/action_shop.png")),
            ("action_quest", assets.load("images/icons/action_quest.png")),
            ("action_train", assets.load("images/icons/action_train.png")),
            ("action_craft", assets.load("images/icons/action_craft.png")),
            ("action_work", assets.load("images/icons/action_work.png")),
            ("action_rest", assets.load("images/icons/action_rest.png")),
            ("action_study", assets.load("images/icons/action_study.png")),
            ("action_duel", assets.load("images/icons/action_duel.png")),
            ("ap", assets.load("images/icons/ap.png")),
            ("equipped", assets.load("images/icons/equipped.png")),
            ("base", assets.load("images/icons/base.png")),
            ("ability", assets.load("images/icons/ability.png")),
            ("perk", assets.load("images/icons/perk.png")),
            ("modifier", assets.load("images/icons/modifier.png")),
            ("effect", assets.load("images/icons/effect.png")),
            ("level", assets.load("images/icons/level.png")),
            ("mana", assets.load("images/icons/mana.png")),
            ("cooldown", assets.load("images/icons/cooldown.png")),
            ("fire", assets.load("images/icons/fire.png")),
            ("ice", assets.load("images/icons/ice.png")),
            ("nature", assets.load("images/icons/nature.png")),
            ("holy", assets.load("images/icons/holy.png")),
            ("shadow", assets.load("images/icons/shadow.png")),
            ("physical", assets.load("images/icons/physical.png")),
            ("melee", assets.load("images/icons/melee.png")),
            ("range", assets.load("images/icons/range.png")),
            ("magical", assets.load("images/icons/magical.png")),
            ("finesse", assets.load("images/icons/finesse.png")),
            ("shield", assets.load("images/icons/shield.png")),
            ("book", assets.load("images/icons/book.png")),
            ("aoe", assets.load("images/icons/aoe.png")),
            ("target", assets.load("images/icons/target.png")),
            ("attack_speed", assets.load("images/icons/attack_speed.png")),
            ("crit_chance", assets.load("images/icons/crit_chance.png")),
            ("hand", assets.load("images/icons/hand.png")),
            ("health", assets.load("images/icons/health.png")),
            ("equipment", assets.load("images/icons/equipment.png")),
            ("strength", assets.load("images/icons/strength.png")),
            ("dexterity", assets.load("images/icons/dexterity.png")),
            ("constitution", assets.load("images/icons/constitution.png")),
            ("intelligence", assets.load("images/icons/intelligence.png")),
            ("wisdom", assets.load("images/icons/wisdom.png")),
            ("charisma", assets.load("images/icons/charisma.png")),
            ("assassin", assets.load("images/icons/assassin.png")),
            // Background
            ("bg", assets.load("images/bg/bg.png")),
            ("bg2", assets.load("images/bg/bg2.png")),
            ("basebg", assets.load("images/bg/base.png")),
            ("shop", assets.load("images/bg/shop.png")),
            ("victory", assets.load("images/bg/victory.png")),
            ("defeat", assets.load("images/bg/defeat.png")),
            // UI
            ("border", assets.load("images/ui/border.png")),
            ("border_hover", assets.load("images/ui/border hover.png")),
            ("stone", assets.load("images/ui/stone.png")),
            ("banner", assets.load("images/ui/banner.png")),
            ("banner_large", assets.load("images/ui/banner large.png")),
            // Races
            ("dwarf", assets.load("images/races/dwarf_man.png")),
            ("dwarf_man", assets.load("images/races/dwarf_man.png")),
            ("dwarf_woman", assets.load("images/races/dwarf_woman.png")),
            ("elf", assets.load("images/races/elf_man.png")),
            ("elf_man", assets.load("images/races/elf_man.png")),
            ("elf_woman", assets.load("images/races/elf_woman.png")),
            ("human", assets.load("images/races/human_man.png")),
            ("human_man", assets.load("images/races/human_man.png")),
            ("human_woman", assets.load("images/races/human_woman.png")),
            ("orc", assets.load("images/races/orc_man.png")),
            ("orc_man", assets.load("images/races/orc_man.png")),
            ("orc_woman", assets.load("images/races/orc_woman.png")),
            // Classes
            ("warrior_human_man", assets.load("images/classes/warrior_human_man.png")),
            ("warrior_human_woman", assets.load("images/classes/warrior_human_woman.png")),
            ("warrior_elf_man", assets.load("images/classes/warrior_elf_man.png")),
            ("warrior_elf_woman", assets.load("images/classes/warrior_elf_woman.png")),
            ("warrior_dwarf_man", assets.load("images/classes/warrior_dwarf_man.png")),
            ("warrior_dwarf_woman", assets.load("images/classes/warrior_dwarf_woman.png")),
            ("warrior_orc_man", assets.load("images/classes/warrior_orc_man.png")),
            ("warrior_orc_woman", assets.load("images/classes/warrior_orc_woman.png")),
            ("mage_human_man", assets.load("images/classes/mage_human_man.png")),
            ("mage_human_woman", assets.load("images/classes/mage_human_woman.png")),
            ("mage_elf_man", assets.load("images/classes/mage_elf_man.png")),
            ("mage_elf_woman", assets.load("images/classes/mage_elf_woman.png")),
            ("mage_dwarf_man", assets.load("images/classes/mage_dwarf_man.png")),
            ("mage_dwarf_woman", assets.load("images/classes/mage_dwarf_woman.png")),
            ("mage_orc_man", assets.load("images/classes/mage_orc_man.png")),
            ("mage_orc_woman", assets.load("images/classes/mage_orc_woman.png")),
            ("assassin_human_man", assets.load("images/classes/assassin_human_man.png")),
            ("assassin_human_woman", assets.load("images/classes/assassin_human_woman.png")),
            ("assassin_elf_man", assets.load("images/classes/assassin_elf_man.png")),
            ("assassin_elf_woman", assets.load("images/classes/assassin_elf_woman.png")),
            ("assassin_dwarf_man", assets.load("images/classes/assassin_dwarf_man.png")),
            ("assassin_dwarf_woman", assets.load("images/classes/assassin_dwarf_woman.png")),
            ("assassin_orc_man", assets.load("images/classes/assassin_orc_man.png")),
            ("assassin_orc_woman", assets.load("images/classes/assassin_orc_woman.png")),
            ("druid_human_man", assets.load("images/classes/druid_human_man.png")),
            ("druid_human_woman", assets.load("images/classes/druid_human_woman.png")),
            ("druid_elf_man", assets.load("images/classes/druid_elf_man.png")),
            ("druid_elf_woman", assets.load("images/classes/druid_elf_woman.png")),
            ("druid_dwarf_man", assets.load("images/classes/druid_dwarf_man.png")),
            ("druid_dwarf_woman", assets.load("images/classes/druid_dwarf_woman.png")),
            ("druid_orc_man", assets.load("images/classes/druid_orc_man.png")),
            ("druid_orc_woman", assets.load("images/classes/druid_orc_woman.png")),
            ("mage_black_human_man", assets.load("images/classes/mage_black_human_man.png")),
            ("mage_black_human_woman", assets.load("images/classes/mage_black_human_woman.png")),
            ("mage_black_elf_man", assets.load("images/classes/mage_black_elf_man.png")),
            ("mage_black_elf_woman", assets.load("images/classes/mage_black_elf_woman.png")),
            ("mage_black_dwarf_man", assets.load("images/classes/mage_black_dwarf_man.png")),
            ("mage_black_dwarf_woman", assets.load("images/classes/mage_black_dwarf_woman.png")),
            ("mage_black_orc_man", assets.load("images/classes/mage_black_orc_man.png")),
            ("mage_black_orc_woman", assets.load("images/classes/mage_black_orc_woman.png")),
            ("mage_red_human_man", assets.load("images/classes/mage_red_man.png")),
            ("mage_red_human_woman", assets.load("images/classes/mage_red_human_woman.png")),
            ("mage_red_elf_man", assets.load("images/classes/mage_red_elf_man.png")),
            ("mage_red_elf_woman", assets.load("images/classes/mage_red_elf_woman.png")),
            ("mage_red_dwarf_man", assets.load("images/classes/mage_red_dwarf_man.png")),
            ("mage_red_dwarf_woman", assets.load("images/classes/mage_red_dwarf_woman.png")),
            ("mage_red_orc_man", assets.load("images/classes/mage_red_orc_man.png")),
            ("mage_red_orc_woman", assets.load("images/classes/mage_red_orc_woman.png")),
            ("mage_green_human_man", assets.load("images/classes/mage_green_human_man.png")),
            ("mage_green_human_woman", assets.load("images/classes/mage_green_human_woman.png")),
            ("mage_green_elf_man", assets.load("images/classes/mage_green_elf_man.png")),
            ("mage_green_elf_woman", assets.load("images/classes/mage_green_elf_woman.png")),
            ("mage_green_dwarf_man", assets.load("images/classes/mage_green_dwarf_man.png")),
            ("mage_green_dwarf_woman", assets.load("images/classes/mage_green_dwarf_woman.png")),
            ("mage_green_orc_man", assets.load("images/classes/mage_green_orc_man.png")),
            ("mage_green_orc_woman", assets.load("images/classes/mage_green_orc_woman.png")),
            ("mage_white_human_man", assets.load("images/classes/mage_white_man.png")),
            ("mage_white_human_woman", assets.load("images/classes/mage_white_human_woman.png")),
            ("mage_white_elf_man", assets.load("images/classes/mage_white_elf_man.png")),
            ("mage_white_elf_woman", assets.load("images/classes/mage_white_elf_woman.png")),
            ("mage_white_dwarf_man", assets.load("images/classes/mage_white_dwarf_man.png")),
            ("mage_white_dwarf_woman", assets.load("images/classes/mage_white_dwarf_woman.png")),
            ("mage_white_orc_man", assets.load("images/classes/mage_white_orc_man.png")),
            ("mage_white_orc_woman", assets.load("images/classes/mage_white_orc_woman.png")),
            // Pets
            ("wolf", assets.load("images/pets/wolf.png")),
            ("snake", assets.load("images/pets/snake.png")),
            ("eagle", assets.load("images/pets/eagle.png")),
            ("bear", assets.load("images/pets/bear.png")),
            ("bat", assets.load("images/pets/bat.png")),
            ("crocodile", assets.load("images/pets/crocodile.png")),
            ("griffin", assets.load("images/pets/griffin.png")),
            ("hyena", assets.load("images/pets/hyena.png")),
            ("infernal can", assets.load("images/pets/infernal can.png")),
            ("lizard", assets.load("images/pets/lizard.png")),
            ("manticore", assets.load("images/pets/manticore.png")),
            ("pegasus", assets.load("images/pets/pegasus.png")),
            ("puma", assets.load("images/pets/puma.png")),
            ("rat", assets.load("images/pets/rat.png")),
            ("spider", assets.load("images/pets/spider.png")),
            ("three headed dog", assets.load("images/pets/three headed dog.png")),
            ("tiger", assets.load("images/pets/tiger.png")),
            ("unicorn", assets.load("images/pets/unicorn.png")),
            ("vulture", assets.load("images/pets/vulture.png")),
        ]);

        for ability in all_abilities() {
            let key: &'static str = Box::leak(format!("build_{}", ability.name).into_boxed_str());
            images.insert(key, assets.load(ability.image.clone()));
        }
        for perk in all_perks() {
            let key: &'static str = Box::leak(format!("build_{}", perk.name).into_boxed_str());
            images.insert(key, assets.load(perk.image.clone()));
        }
        for weapon in all_weapons() {
            let key: &'static str = Box::leak(format!("build_{}", weapon.name).into_boxed_str());
            images.insert(key, assets.load(weapon.image.clone()));
        }
        for wearable in all_wearables() {
            let key: &'static str = Box::leak(format!("build_{}", wearable.name).into_boxed_str());
            images.insert(key, assets.load(wearable.image.clone()));
        }

        Self {
            audio,
            fonts,
            images,
        }
    }
}
