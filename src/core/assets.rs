use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;

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
        ]);

        let fonts = HashMap::from([
            ("bold", assets.load("fonts/FiraSans-Bold.ttf")),
            ("medium", assets.load("fonts/FiraMono-Medium.ttf")),
        ]);

        let images: HashMap<&'static str, Handle<Image>> = HashMap::from([
            // Icons
            ("mute", assets.load("images/icons/mute.png")),
            ("sound", assets.load("images/icons/sound.png")),
            ("music", assets.load("images/icons/music.png")),
            ("sword", assets.load("images/icons/sword.png")),
            ("shield", assets.load("images/icons/shield.png")),
            ("armor_icon", assets.load("images/icons/armor.png")),
            ("attack_icon", assets.load("images/icons/attack.png")),
            ("initiative_icon", assets.load("images/icons/initiative.png")),
            ("helmet_icon", assets.load("images/icons/Icon_10.png")),
            ("boots_icon", assets.load("images/icons/Icon_11.png")),
            ("gold", assets.load("images/icons/gold.png")),
            ("action_hunt", assets.load("images/icons/Icon_01.png")),
            ("action_shop", assets.load("images/icons/Icon_02.png")),
            ("action_quest", assets.load("images/icons/Icon_03.png")),
            ("action_train", assets.load("images/icons/Icon_04.png")),
            ("action_craft", assets.load("images/icons/Icon_08.png")),
            ("action_rest", assets.load("images/icons/Icon_09.png")),
            ("action_inventory", assets.load("images/icons/Icon_10.png")),
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
            ("wolf", assets.load("images/pets/wolf.png")),
            ("snake", assets.load("images/pets/snake.png")),
            ("eagle", assets.load("images/pets/eagle.png")),
            ("bear", assets.load("images/pets/bear.png")),
        ]);

        Self {
            audio,
            fonts,
            images,
        }
    }
}
