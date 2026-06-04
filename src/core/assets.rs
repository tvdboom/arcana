use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TextureInfo {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Clone)]
pub struct AtlasInfo {
    pub image: Handle<Image>,
    pub atlas: TextureAtlas,
    pub last_index: usize,
}

#[derive(Resource)]
pub struct WorldAssets {
    pub audio: HashMap<&'static str, Handle<AudioSource>>,
    pub fonts: HashMap<&'static str, Handle<Font>>,
    pub images: HashMap<&'static str, Handle<Image>>,
    pub textures: HashMap<&'static str, TextureInfo>,
    pub atlas: HashMap<&'static str, AtlasInfo>,
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

    pub fn texture(&self, name: impl Into<String>) -> TextureInfo {
        self.get_asset(&self.textures, name, "texture").clone()
    }

    pub fn atlas(&self, name: impl Into<String>) -> AtlasInfo {
        self.get_asset(&self.atlas, name, "atlas").clone()
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
            // Background
            ("bg", assets.load("images/bg/bg.png")),
            ("victory", assets.load("images/bg/victory.png")),
            ("defeat", assets.load("images/bg/defeat.png")),
        ]);

        let mut atlas: HashMap<&'static str, AtlasInfo> = HashMap::new();

        let textures = HashMap::new();

        Self {
            audio,
            fonts,
            images,
            textures,
            atlas,
        }
    }
}
