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

        let mut images: HashMap<&'static str, Handle<Image>> = HashMap::from([
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
            "images/catalog/consumable/Alchemy_40_poisonousherbs.webp",
        );

        assert!(aliases.iter().any(|alias| alias == "build_Mythic Alchemy Poisonousherbs"));
        assert!(aliases
            .iter()
            .any(|alias| alias == "images/catalog/consumable/Alchemy_40_poisonousherbs.webp"));
        assert!(aliases.iter().any(|alias| alias == "Alchemy_40_poisonousherbs"));
    }
}
