use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;

use crate::core::catalog::{GENERATED_ABILITIES, GENERATED_EQUIPMENT, GENERATED_PERKS};

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
            ("victory", assets.load("audio/victory.ogg")),
            ("explosion", assets.load("audio/explosion.ogg")),
            ("rest", assets.load("audio/rest.ogg")),
            ("work", assets.load("audio/work.ogg")),
            ("study", assets.load("audio/study.ogg")),
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
            ("armor_icon", assets.load("images/icons/armor.png")),
            ("attack_icon", assets.load("images/icons/attack.png")),
            ("initiative_icon", assets.load("images/icons/initiative.png")),
            ("gold", assets.load("images/icons/gold.png")),
            ("action_hunt", assets.load("images/icons/action_hunt.png")),
            ("action_shop", assets.load("images/icons/action_shop.png")),
            ("action_quest", assets.load("images/icons/action_quest.png")),
            ("action_train", assets.load("images/icons/action_train.png")),
            ("action_craft", assets.load("images/icons/action_craft.png")),
            ("action_work", assets.load("images/icons/action_work.png")),
            ("action_rest", assets.load("images/icons/action_rest.png")),
            ("action_study", assets.load("images/icons/action_study.png")),
            ("ap", assets.load("images/icons/ap.png")),
            ("equipped", assets.load("images/icons/equipped.png")),
            // Background
            ("bg", assets.load("images/bg/bg.png")),
            ("bg2", assets.load("images/bg/bg2.png")),
            ("bg3", assets.load("images/bg/bg3.png")),
            ("victory", assets.load("images/bg/victory.png")),
            ("defeat", assets.load("images/bg/defeat.png")),
            // UI Borders
            ("border", assets.load("images/ui/border.png")),
            ("border_hover", assets.load("images/ui/border hover.png")),
            ("stone", assets.load("images/ui/stone.png")),
            ("banner", assets.load("images/ui/banner.png")),
            ("banner_large", assets.load("images/ui/banner large.png")),
            // Races
            ("dwarf", assets.load("images/races/dwarf_male.png")),
            ("dwarf_male", assets.load("images/races/dwarf_male.png")),
            ("dwarf_female", assets.load("images/races/dwarf_female.png")),
            ("elf", assets.load("images/races/elf_male.png")),
            ("elf_male", assets.load("images/races/elf_male.png")),
            ("elf_female", assets.load("images/races/elf_female.png")),
            ("human", assets.load("images/races/human_male.png")),
            ("human_male", assets.load("images/races/human_male.png")),
            ("human_female", assets.load("images/races/human_female.png")),
            ("orc", assets.load("images/races/orc_male.png")),
            ("orc_male", assets.load("images/races/orc_male.png")),
            ("orc_female", assets.load("images/races/orc_female.png")),
            // Classes
            ("warrior", assets.load("images/classes/warrior_human_male.png")),
            ("warrior_human", assets.load("images/classes/warrior_human_male.png")),
            ("warrior_elf", assets.load("images/classes/warrior_elf_male.png")),
            ("warrior_dwarf", assets.load("images/classes/warrior_dwarf_male.png")),
            ("warrior_orc", assets.load("images/classes/warrior_orc_male.png")),
            ("mage", assets.load("images/classes/mage_human_male.png")),
            ("mage_human", assets.load("images/classes/mage_human_male.png")),
            ("mage_elf", assets.load("images/classes/mage_elf_male.png")),
            ("mage_dwarf", assets.load("images/classes/mage_dwarf_male.png")),
            ("mage_orc", assets.load("images/classes/mage_orc_male.png")),
            ("rogue", assets.load("images/classes/rogue_human_male.png")),
            ("rogue_human", assets.load("images/classes/rogue_human_male.png")),
            ("rogue_elf", assets.load("images/classes/rogue_elf_male.png")),
            ("rogue_dwarf", assets.load("images/classes/rogue_dwarf_male.png")),
            ("rogue_orc", assets.load("images/classes/rogue_orc_male.png")),
            ("druid", assets.load("images/classes/druid_human_male.png")),
            ("druid_human", assets.load("images/classes/druid_human_male.png")),
            ("druid_elf", assets.load("images/classes/druid_elf_male.png")),
            ("druid_dwarf", assets.load("images/classes/druid_dwarf_male.png")),
            ("druid_orc", assets.load("images/classes/druid_orc_male.png")),
            ("black", assets.load("images/classes/mage_black_human_male.png")),
            ("mage_black", assets.load("images/classes/mage_black_human_male.png")),
            ("mage_black_human", assets.load("images/classes/mage_black_human_male.png")),
            ("mage_black_elf", assets.load("images/classes/mage_black_elf_male.png")),
            ("mage_black_dwarf", assets.load("images/classes/mage_black_dwarf_male.png")),
            ("mage_black_orc", assets.load("images/classes/mage_black_orc_male.png")),
            ("red", assets.load("images/classes/mage_red_male.png")),
            ("mage_red", assets.load("images/classes/mage_red_male.png")),
            ("mage_red_human", assets.load("images/classes/mage_red_male.png")),
            ("mage_red_elf", assets.load("images/classes/mage_red_elf_male.png")),
            ("mage_red_dwarf", assets.load("images/classes/mage_red_dwarf_male.png")),
            ("mage_red_orc", assets.load("images/classes/mage_red_orc_male.png")),
            ("green", assets.load("images/classes/mage_green_human_male.png")),
            ("mage_green", assets.load("images/classes/mage_green_human_male.png")),
            ("mage_green_human", assets.load("images/classes/mage_green_human_male.png")),
            ("mage_green_elf", assets.load("images/classes/mage_green_elf_male.png")),
            ("mage_green_dwarf", assets.load("images/classes/mage_green_dwarf_male.png")),
            ("mage_green_orc", assets.load("images/classes/mage_green_orc_male.png")),
            ("white", assets.load("images/classes/mage_white_male.png")),
            ("mage_white", assets.load("images/classes/mage_white_male.png")),
            ("mage_white_human", assets.load("images/classes/mage_white_male.png")),
            ("mage_white_elf", assets.load("images/classes/mage_white_elf_male.png")),
            ("mage_white_dwarf", assets.load("images/classes/mage_white_dwarf_male.png")),
            ("mage_white_orc", assets.load("images/classes/mage_white_orc_male.png")),
            ("warrior_human_male", assets.load("images/classes/warrior_human_male.png")),
            ("warrior_human_female", assets.load("images/classes/warrior_human_female.png")),
            ("warrior_elf_male", assets.load("images/classes/warrior_elf_male.png")),
            ("warrior_elf_female", assets.load("images/classes/warrior_elf_female.png")),
            ("warrior_dwarf_male", assets.load("images/classes/warrior_dwarf_male.png")),
            ("warrior_dwarf_female", assets.load("images/classes/warrior_dwarf_female.png")),
            ("warrior_orc_male", assets.load("images/classes/warrior_orc_male.png")),
            ("warrior_orc_female", assets.load("images/classes/warrior_orc_female.png")),
            ("mage_human_male", assets.load("images/classes/mage_human_male.png")),
            ("mage_human_female", assets.load("images/classes/mage_human_female.png")),
            ("mage_elf_male", assets.load("images/classes/mage_elf_male.png")),
            ("mage_elf_female", assets.load("images/classes/mage_elf_female.png")),
            ("mage_dwarf_male", assets.load("images/classes/mage_dwarf_male.png")),
            ("mage_dwarf_female", assets.load("images/classes/mage_dwarf_female.png")),
            ("mage_orc_male", assets.load("images/classes/mage_orc_male.png")),
            ("mage_orc_female", assets.load("images/classes/mage_orc_female.png")),
            ("rogue_human_male", assets.load("images/classes/rogue_human_male.png")),
            ("rogue_human_female", assets.load("images/classes/rogue_human_female.png")),
            ("rogue_elf_male", assets.load("images/classes/rogue_elf_male.png")),
            ("rogue_elf_female", assets.load("images/classes/rogue_elf_female.png")),
            ("rogue_dwarf_male", assets.load("images/classes/rogue_dwarf_male.png")),
            ("rogue_dwarf_female", assets.load("images/classes/rogue_dwarf_female.png")),
            ("rogue_orc_male", assets.load("images/classes/rogue_orc_male.png")),
            ("rogue_orc_female", assets.load("images/classes/rogue_orc_female.png")),
            ("druid_human_male", assets.load("images/classes/druid_human_male.png")),
            ("druid_human_female", assets.load("images/classes/druid_human_female.png")),
            ("druid_elf_male", assets.load("images/classes/druid_elf_male.png")),
            ("druid_elf_female", assets.load("images/classes/druid_elf_female.png")),
            ("druid_dwarf_male", assets.load("images/classes/druid_dwarf_male.png")),
            ("druid_dwarf_female", assets.load("images/classes/druid_dwarf_female.png")),
            ("druid_orc_male", assets.load("images/classes/druid_orc_male.png")),
            ("druid_orc_female", assets.load("images/classes/druid_orc_female.png")),
            ("mage_black_human_male", assets.load("images/classes/mage_black_human_male.png")),
            ("mage_black_human_female", assets.load("images/classes/mage_black_human_female.png")),
            ("mage_black_elf_male", assets.load("images/classes/mage_black_elf_male.png")),
            ("mage_black_elf_female", assets.load("images/classes/mage_black_elf_female.png")),
            ("mage_black_dwarf_male", assets.load("images/classes/mage_black_dwarf_male.png")),
            ("mage_black_dwarf_female", assets.load("images/classes/mage_black_dwarf_female.png")),
            ("mage_black_orc_male", assets.load("images/classes/mage_black_orc_male.png")),
            ("mage_black_orc_female", assets.load("images/classes/mage_black_orc_female.png")),
            ("mage_red_human_male", assets.load("images/classes/mage_red_male.png")),
            ("mage_red_human_female", assets.load("images/classes/mage_red_human_female.png")),
            ("mage_red_elf_male", assets.load("images/classes/mage_red_elf_male.png")),
            ("mage_red_elf_female", assets.load("images/classes/mage_red_elf_female.png")),
            ("mage_red_dwarf_male", assets.load("images/classes/mage_red_dwarf_male.png")),
            ("mage_red_dwarf_female", assets.load("images/classes/mage_red_dwarf_female.png")),
            ("mage_red_orc_male", assets.load("images/classes/mage_red_orc_male.png")),
            ("mage_red_orc_female", assets.load("images/classes/mage_red_orc_female.png")),
            ("mage_green_human_male", assets.load("images/classes/mage_green_human_male.png")),
            ("mage_green_human_female", assets.load("images/classes/mage_green_human_female.png")),
            ("mage_green_elf_male", assets.load("images/classes/mage_green_elf_male.png")),
            ("mage_green_elf_female", assets.load("images/classes/mage_green_elf_female.png")),
            ("mage_green_dwarf_male", assets.load("images/classes/mage_green_dwarf_male.png")),
            ("mage_green_dwarf_female", assets.load("images/classes/mage_green_dwarf_female.png")),
            ("mage_green_orc_male", assets.load("images/classes/mage_green_orc_male.png")),
            ("mage_green_orc_female", assets.load("images/classes/mage_green_orc_female.png")),
            ("mage_white_human_male", assets.load("images/classes/mage_white_male.png")),
            ("mage_white_human_female", assets.load("images/classes/mage_white_human_female.png")),
            ("mage_white_elf_male", assets.load("images/classes/mage_white_elf_male.png")),
            ("mage_white_elf_female", assets.load("images/classes/mage_white_elf_female.png")),
            ("mage_white_dwarf_male", assets.load("images/classes/mage_white_dwarf_male.png")),
            ("mage_white_dwarf_female", assets.load("images/classes/mage_white_dwarf_female.png")),
            ("mage_white_orc_male", assets.load("images/classes/mage_white_orc_male.png")),
            ("mage_white_orc_female", assets.load("images/classes/mage_white_orc_female.png")),
            ("wolf", assets.load("images/pets/wolf.png")),
            ("snake", assets.load("images/pets/snake.png")),
            ("eagle", assets.load("images/pets/eagle.png")),
            ("bear", assets.load("images/pets/bear.png")),
            ("bat", assets.load("images/pets/bat.png")),
            ("crocodile", assets.load("images/pets/crocodile.png")),
            ("hyena", assets.load("images/pets/hyena.png")),
            ("infernal can", assets.load("images/pets/infernal can.png")),
            ("lizard", assets.load("images/pets/lizard.png")),
            ("pegasus", assets.load("images/pets/pegasus.png")),
            ("rat", assets.load("images/pets/rat.png")),
            ("spider", assets.load("images/pets/spider.png")),
            ("three headed dog", assets.load("images/pets/three headed dog.png")),
            ("tiger", assets.load("images/pets/tiger.png")),
            ("unicorn", assets.load("images/pets/unicorn.png")),
            ("vulture", assets.load("images/pets/vulture.png")),
        ]);

        for item in GENERATED_EQUIPMENT {
            images.insert(item.name, assets.load(item.icon_path));
        }
        for ability in GENERATED_ABILITIES {
            images.insert(ability.name, assets.load(ability.icon_path));
        }
        for perk in GENERATED_PERKS {
            images.insert(perk.name, assets.load(perk.icon_path));
        }

        Self {
            audio,
            fonts,
            images,
        }
    }
}
