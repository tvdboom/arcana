use crate::core::catalog::catalog::*;
use bevy::asset::AssetServer;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use std::collections::HashMap;
use std::path::Path;

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

fn leak_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn insert_image_aliases(
    images: &mut HashMap<&'static str, Handle<Image>>,
    image: &Handle<Image>,
    aliases: impl IntoIterator<Item = String>,
) {
    for alias in aliases {
        images.entry(leak_str(alias)).or_insert_with(|| image.clone());
    }
}

fn catalog_image_aliases(name: &str, image: &str) -> Vec<String> {
    let mut aliases = vec![format!("build_{}", name), image.to_string()];
    if let Some(stem) = Path::new(image).file_stem().and_then(|s| s.to_str()) {
        aliases.push(stem.to_string());
    }
    aliases
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
            ("skull", assets.load("images/icons/skull.ktx2")),
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
            ("bg_rest", assets.load("images/bg/rest.ktx2")),
            ("bg_study", assets.load("images/bg/study.ktx2")),
            ("bg_work", assets.load("images/bg/work.ktx2")),
            ("bg_train", assets.load("images/bg/train.ktx2")),
            ("bg_craft", assets.load("images/bg/craft.ktx2")),
            ("bg_hunt", assets.load("images/bg/hunt.ktx2")),
            ("bg_quest", assets.load("images/bg/quest.ktx2")),
            ("bg_duel", assets.load("images/bg/duel.ktx2")),
            ("bg_combat", assets.load("images/bg/combat.ktx2")),
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
            // Actions
            ("action_clerical_labor", load_linear(assets, "images/actions/clerical_labor.ktx2")),
            ("action_craft_labor", load_linear(assets, "images/actions/craft_labor.ktx2")),
            ("action_manual_labor", load_linear(assets, "images/actions/manual_labor.ktx2")),
            ("action_apprenticeship", load_linear(assets, "images/actions/apprenticeship.ktx2")),
            ("action_mentorship", load_linear(assets, "images/actions/mentorship.ktx2")),
            ("action_conditioning", load_linear(assets, "images/actions/conditioning.ktx2")),
            ("action_simple_rest", load_linear(assets, "images/actions/simple_rest.ktx2")),
            ("action_common_lodging", load_linear(assets, "images/actions/common_lodging.ktx2")),
            (
                "action_grand_accommodation",
                load_linear(assets, "images/actions/grand_accomodation.ktx2"),
            ),
            ("action_melee", load_linear(assets, "images/actions/melee.ktx2")),
            ("action_range", load_linear(assets, "images/actions/range.ktx2")),
            ("action_finesse", load_linear(assets, "images/actions/finesse.ktx2")),
            ("action_easy_hunt", load_linear(assets, "images/actions/easy_hunt.ktx2")),
            ("action_wild_hunt", load_linear(assets, "images/actions/wild_hunt.ktx2")),
            ("action_deadly_hunt", load_linear(assets, "images/actions/deadly_hunt.ktx2")),
            ("action_errand", load_linear(assets, "images/actions/errand.ktx2")),
            ("action_expedition", load_linear(assets, "images/actions/expedition.ktx2")),
            ("action_odyssey", load_linear(assets, "images/actions/odyssey.ktx2")),
        ]);

        for ability in all_abilities() {
            let image = load_linear(assets, ability.image.clone());
            insert_image_aliases(
                &mut images,
                &image,
                catalog_image_aliases(&ability.name, &ability.image),
            );
        }

        for perk in all_perks() {
            let image = load_linear(assets, perk.image.clone());
            insert_image_aliases(
                &mut images,
                &image,
                catalog_image_aliases(&perk.name, &perk.image),
            );
        }

        for weapon in all_weapons() {
            let image = load_linear(assets, weapon.image.clone());
            insert_image_aliases(
                &mut images,
                &image,
                catalog_image_aliases(&weapon.name, &weapon.image),
            );
        }

        for wearable in all_wearables() {
            let image = load_linear(assets, wearable.image.clone());
            insert_image_aliases(
                &mut images,
                &image,
                catalog_image_aliases(&wearable.name, &wearable.image),
            );
        }

        for consumable in all_consumables() {
            let image = load_linear(assets, consumable.image.clone());
            insert_image_aliases(
                &mut images,
                &image,
                catalog_image_aliases(&consumable.name, &consumable.image),
            );
        }

        for artifact in all_artifacts() {
            let image = load_linear(assets, artifact.image.clone());
            insert_image_aliases(
                &mut images,
                &image,
                catalog_image_aliases(&artifact.name, &artifact.image),
            );
        }

        for monster in all_monsters() {
            let image = load_linear(assets, monster.image.clone());
            insert_image_aliases(
                &mut images,
                &image,
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
            images,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::catalog_image_aliases;

    #[test]
    fn catalog_image_aliases_include_build_key_and_raw_path() {
        let aliases = catalog_image_aliases(
            "Mythic Alchemy Poisonousherbs",
            "images/catalog/consumable/Alchemy_40_poisonousherbs.ktx2",
        );

        assert!(aliases.iter().any(|alias| alias == "build_Mythic Alchemy Poisonousherbs"));
        assert!(aliases
            .iter()
            .any(|alias| alias == "images/catalog/consumable/Alchemy_40_poisonousherbs.ktx2"));
        assert!(aliases.iter().any(|alias| alias == "Alchemy_40_poisonousherbs"));
    }
}
