//! Deterministic generator for Arcana's RON catalogs and catalog-backed progression.
//!
//! It derives items and monsters from the available source artwork.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const QUALITY_PREFIXES: [[&str; 5]; 5] = [
    ["Crude", "Weathered", "Training", "Simple", "Makeshift"],
    ["Sturdy", "Balanced", "Hardened", "Tempered", "Soldier's"],
    ["Fine", "Runed", "Gilded", "Veteran's", "Artisan"],
    ["Masterwork", "Royal", "Vanguard", "Champion's", "Ornate"],
    ["Mythic", "Ancient", "Celestial", "Relic", "Ascendant"],
];

const CONSUMABLE_PREFIXES: [[&str; 6]; 5] = [
    ["Dilute", "Lesser", "Simple", "Mild", "Stable", "Filtered"],
    ["Balanced", "Steady", "Refined", "Concentrated", "Strong", "Pure"],
    ["Potent", "Superior", "Distilled", "Artisan", "Greater", "Enriched"],
    ["Masterwork", "Royal", "Perfected", "Sovereign", "Exalted", "Peerless"],
    ["Mythic", "Ancient", "Celestial", "Transcendent", "Eternal", "Primal"],
];

const PHYSICAL_SUFFIXES: &[&str] = &[
    "of the Vanguard",
    "of the Duelist",
    "of the Sentinel",
    "of the Hunt",
    "of Iron Resolve",
    "of the Warlord",
];

const FIRE_SUFFIXES: &[&str] = &[
    "of Embers",
    "of the Furnace",
    "of Cinders",
    "of the Phoenix",
    "of the Inferno",
    "of Sunfire",
];

const ICE_SUFFIXES: &[&str] = &[
    "of Rime",
    "of the Glacier",
    "of Winter",
    "of Hoarfrost",
    "of the Northwind",
    "of Permafrost",
];

const NATURE_SUFFIXES: &[&str] = &[
    "of the Grove",
    "of Briars",
    "of Wildwood",
    "of the Verdant Path",
    "of the Great Hunt",
    "of Deep Roots",
];

const HOLY_SUFFIXES: &[&str] =
    &["of Dawn", "of Grace", "of the Beacon", "of the Sacred Oath", "of the Sun", "of the Seraph"];

const SHADOW_SUFFIXES: &[&str] =
    &["of Dusk", "of the Veil", "of Whispers", "of the Grave", "of the Void", "of Nightfall"];

const UNIQUE_MODIFIERS: &[&str] = &[
    "of the dawn",
    "of the night",
    "of the storm",
    "of the sun",
    "of the moon",
    "of the stars",
    "of the void",
    "of the wind",
    "of the sea",
    "of the flame",
    "of the frost",
    "of the shadow",
    "of the grove",
    "of the peak",
    "of the deep",
    "of the wild",
    "of the desert",
    "of the tundra",
    "of the mist",
    "of the abyss",
    "of the marsh",
    "of the vale",
    "of the forest",
    "of the hearth",
    "of the skies",
];

/// Ability schools that must have a level-one option for character creation.
const STARTING_ABILITY_KINDS: &[&str] = &["Physical", "Fire", "Ice", "Nature", "Holy", "Shadow"];

const SHADOW_POOL: &[&str] = &[
    "Vampiric Touch",
    "Soul Drain",
    "Agony Hex",
    "Dark Covenant",
    "Withering Curse",
    "Shadow Bolt",
    "Death Grasp",
    "Grave Chill",
    "Abyssal Maw",
    "Void Tendril",
    "Haunting Echo",
    "Spectral Shield",
    "Nightmare Plague",
    "Torment Spike",
    "Wraith Walk",
    "Doom Curse",
    "Corpse Explosion",
    "Necrotic Rot",
    "Shadow Cloak",
    "Demonic Pact",
    "Reaper Slash",
    "Eldritch Terror",
    "Lich Touch",
    "Siphon Life",
    "Dark Whispers",
];

const HOLY_POOL: &[&str] = &[
    "Smite Evil",
    "Divine Radiance",
    "Lay on Hands",
    "Judgment Crest",
    "Sacred Bastion",
    "Holy Nova",
    "Angelic Blessing",
    "Solar Wrath",
    "Seraphic Shield",
    "Guiding Light",
    "Sanctuary Dome",
    "Heavenly Hammer",
    "Redeemer Grace",
    "Purifying Flame",
    "Sacred Ground",
    "Aura of Hope",
    "Beacon of Light",
    "Devout Prayer",
    "Pious Guard",
    "Righteous Fury",
    "Ascension Light",
    "Clerics Ward",
    "Sunburst Strike",
    "Graceful Touch",
    "Solomon Shield",
];

const NATURE_POOL: &[&str] = &[
    "Natures Touch",
    "Bramble Growth",
    "Wild Growth",
    "Thornspire Aura",
    "Toxic Bloom",
    "Oakskin Guard",
    "Hurricane Gust",
    "Stone Barrier",
    "Earthquake Tremor",
    "Vine Snare",
    "Root Grasp",
    "Floral Gale",
    "Leaf Blade",
    "Tornado Spin",
    "Thistle Armor",
    "Serpent Venom",
    "Wolf Pack Call",
    "Bear Swipe",
    "Eagle Screech",
    "Forest Harmony",
    "Primal Roar",
    "Tectonic Wave",
    "Solar Synthesis",
    "Spore Cloud",
    "Ivy Lash",
];

const FROST_POOL: &[&str] = &[
    "Glacial Spike",
    "Ice Shackle",
    "Blizzard Veil",
    "Frostbite Touch",
    "Frozen Tomb",
    "Ice Nova",
    "Hailstone Shower",
    "Chill Wave",
    "Glacier Wall",
    "Frost Shield",
    "Winter Breath",
    "Polar Blast",
    "Snowstorm Gale",
    "Icicle Spear",
    "Frozen Heart",
    "Deep Freeze",
    "Cryo Blast",
    "Iceberg Smash",
    "Frost Wave",
    "Crystal Shard",
    "Glacial Aegis",
    "Cold Snap",
    "Snow Drift",
    "Chilling Touch",
    "Permafrost Touch",
];

const FIRE_POOL: &[&str] = &[
    "Fire",
    "Pyroblast Barrage",
    "Flame Wreath",
    "Combustion Spark",
    "Infernal Cleave",
    "Cinder Shield",
    "Fireball",
    "Fire Wall",
    "Fire Wave",
    "Fire Rain",
    "Flame Burst",
    "Magma Eruption",
    "Ignite Touch",
    "Scorching Beam",
    "Cinder Blast",
    "Blazing Dash",
    "Incinerate Burst",
    "Phoenix Rebirth",
    "Lava Shield",
    "Volcano Erupt",
    "Dragon Breath",
    "Searing Heat",
    "Crimson Flare",
    "Conflagration Flame",
    "Sunfire Spike",
];

const PHYSICAL_POOL: &[&str] = &[
    "Heavy Strike",
    "Blade Rush",
    "Furious Slash",
    "Savage Rend",
    "Crushing Blow",
    "Sweeping Cleave",
    "Overpower",
    "Decimate",
    "Whirlwind",
    "Shield Breaker",
    "Heroic Strike",
    "Concussive Blow",
    "Rend Flesh",
    "Battle Cry",
    "Mortal Strike",
    "Iron Wall",
    "Shield Block",
    "Unbreakable Guard",
    "Stalwart Defense",
    "Shield Bash",
    "Fortress Stance",
    "Absorb Shield",
    "Sturdy Wall",
    "Ironclad Resolve",
    "Stone Aegis",
    "Defenders Oath",
    "Unyielding Will",
    "Guardian Barrier",
    "Last Stand",
    "Vanguard Aegis",
    "Viper Venom",
    "Sneak Attack",
    "Backstab",
    "Shadowstrike",
    "Lethal Poison",
    "Silent Cut",
    "Eviscerate",
    "Garrote",
    "Deadly Toxins",
    "Noxious Wound",
    "Ambush",
    "Fatal Strike",
    "Venomous Bite",
    "Agile Dagger",
    "Assassins Mark",
    "Swift Step",
    "Dodge Roll",
    "Quick Dash",
    "Evasive Maneuver",
    "Wind Runner",
    "Throwing Axe",
    "Double Jump",
    "Flank Strike",
    "Skirmish Dodge",
    "Acrobatic Leap",
    "Fleet Footed",
    "Agile Shot",
    "Quick Reflexes",
    "Fast Step",
    "Sidestep",
];

/// Returns last number.
fn get_last_number(s: &str) -> Option<f64> {
    let mut current_num = String::new();
    let mut numbers = Vec::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else {
            if !current_num.is_empty() {
                if let Ok(val) = current_num.parse::<f64>() {
                    numbers.push(val);
                }
                current_num.clear();
            }
        }
    }
    if !current_num.is_empty() {
        if let Ok(val) = current_num.parse::<f64>() {
            numbers.push(val);
        }
    }
    numbers.last().copied()
}

/// Capitalizes words.
fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Scores artwork for deterministic placement along the 20-level progression.
fn get_image_score(filename: &str) -> f64 {
    let mut score = 50.0;
    let lower = filename.to_lowercase();

    if let Some(num) = get_last_number(filename) {
        score += num * 1.5;
    }

    let cool_words = [
        "dragon",
        "fire",
        "ice",
        "magic",
        "demon",
        "shadow",
        "dark",
        "gold",
        "golden",
        "royal",
        "lord",
        "legendary",
        "crystal",
        "cosmic",
        "lightning",
        "light",
        "holy",
        "unholy",
        "death",
        "doom",
        "chaos",
        "blood",
        "hell",
        "vampire",
        "sun",
        "moon",
        "star",
        "arch",
        "grand",
        "elder",
        "master",
        "championship",
        "mythic",
        "epic",
    ];
    for w in cool_words {
        if lower.contains(w) {
            score += 25.0;
        }
    }

    let basic_words =
        ["old", "broken", "rusty", "crude", "wood", "stone", "simple", "training", "weak", "basic"];
    for w in basic_words {
        if lower.contains(w) {
            score -= 30.0;
        }
    }

    score
}

/// Converts an asset filename into a corrected, human-readable base name.
fn clean_name(filename: &str) -> String {
    let name_without_ext =
        Path::new(filename).file_stem().and_then(|s| s.to_str()).unwrap_or(filename);

    let lower_stem = name_without_ext.to_lowercase();
    if lower_stem == "skill_217" {
        return "Break Chains".to_string();
    }
    if lower_stem == "skill_100" {
        return "Beast Vortex".to_string();
    }
    if lower_stem == "skill_101" {
        return "Frost Current".to_string();
    }
    if lower_stem == "skill_102" {
        return "Torrential Geyser".to_string();
    }

    let mut cleaned = String::new();
    let mut prev_char: Option<char> = None;
    for c in name_without_ext.chars() {
        if c.is_ascii_digit() {
            prev_char = None;
            continue;
        }
        if c == '_' || c == '-' || c == ' ' {
            if !cleaned.ends_with(' ') && !cleaned.is_empty() {
                cleaned.push(' ');
            }
            prev_char = Some(' ');
        } else {
            if c.is_uppercase() {
                if let Some(p) = prev_char {
                    if p.is_lowercase() && !cleaned.ends_with(' ') {
                        cleaned.push(' ');
                    }
                }
            }
            cleaned.push(c);
            prev_char = Some(c);
        }
    }

    let mut cleaned = cleaned.trim().to_string();
    while cleaned.contains("  ") {
        cleaned = cleaned.replace("  ", " ");
    }
    let cleaned = cleaned.trim().to_string();

    let words: Vec<String> = cleaned
        .split_whitespace()
        .filter(|word| {
            let w = word.to_lowercase();
            w != "v" && w != "skill" && w != "skills" && w != "ability" && w != "abilities"
        })
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut s = first.to_uppercase().to_string();
                    s.push_str(&chars.as_str().to_lowercase());
                    s
                },
            }
        })
        .collect();

    let mut name = words.join(" ");
    let noisy_prefixes = [
        "Archerskill",
        "Assassinskill",
        "Druideskill",
        "Engineerskill",
        "Mageskill",
        "Paladinskill",
        "Priestskill",
        "Shamanskill",
        "Warriorskill",
        "Warlock",
        "Tech",
    ];
    for prefix in noisy_prefixes {
        if let Some(rest) = name.strip_prefix(prefix).map(str::trim) {
            if !rest.is_empty() {
                name = rest.to_string();
                break;
            }
        }
    }

    let corrections = [
        ("Ironoreodds", "Iron Ore"),
        ("Ironore", "Iron Ore"),
        ("Clearironore", "Pure Iron Ore"),
        ("Goldore", "Gold Ore"),
        ("Biggoldore", "Gold Nugget"),
        ("Silverore", "Silver Ore"),
        ("Nativecopper", "Copper Ore"),
        ("Cobaltore", "Cobalt Ore"),
        ("Manastoune", "Mana Stone"),
        ("Mysticstoune", "Mystic Stone"),
        ("Stoune", "Stone"),
        ("Rune Stoune", "Rune Stone"),
        ("Runecrystal", "Rune Crystal"),
        ("Runestone", "Rune Stone"),
        ("Greenscroll", "Green Scroll"),
        ("Purplescroll", "Purple Scroll"),
        ("Bluescroll", "Blue Scroll"),
        ("Orangescroll", "Orange Scroll"),
        ("Redscroll", "Red Scroll"),
        ("Deathscroll", "Death Scroll"),
        ("Runescroll", "Rune Scroll"),
        ("Manascroll", "Mana Scroll"),
        ("Healingscroll", "Healing Scroll"),
        ("Runeponeti", "Rune Essence"),
        ("Manamushroom", "Mana Mushroom"),
        ("Demonmushroom", "Demon Mushroom"),
        ("Raindropmushroom", "Raindrop Mushroom"),
        ("Stinkymushroom", "Stinky Mushroom"),
        ("Shadowmushroom", "Shadow Mushroom"),
        ("Deadlymushroom", "Deadly Mushroom"),
        ("Manaflower", "Mana Flower"),
        ("Fireflower", "Fire Flower"),
        ("Oilplant", "Oil Plant"),
        ("Shadowberry", "Shadow Berry"),
        ("Raptorherb", "Raptor Herb"),
        ("Dragonherb", "Dragon Herb"),
        ("Shadowflower", "Shadow Flower"),
        ("Redflower", "Red Flower"),
        ("Wildrose", "Wild Rose"),
        ("Purpleflower", "Purple Flower"),
        ("Yellowflower", "Yellow Flower"),
        ("Energyflower", "Energy Flower"),
        ("Stinkyflower", "Stinky Flower"),
        ("Sickflower", "Sick Flower"),
        ("Whiteberry", "White Berry"),
        ("Edgyroot", "Edgy Root"),
        ("Goblineye", "Goblin Eye"),
        ("Fellherb", "Fell Herb"),
        ("Yellowrose", "Yellow Rose"),
        ("Bloodberry", "Blood Berry"),
        ("Cryingflower", "Crying Flower"),
        ("Wormherb", "Worm Herb"),
        ("Mistyflower", "Misty Flower"),
        ("Kingflower", "King Flower"),
        ("Healleaves", "Healing Leaves"),
        ("Manaleaves", "Mana Leaves"),
        ("Woodmoss", "Wood Moss"),
        ("Seakale", "Sea Kale"),
        ("Goldenflower", "Golden Flower"),
        ("Ancientflower", "Ancient Flower"),
        ("Sunflower", "Sunflower"),
        ("Dragonflower", "Dragon Flower"),
        ("Whiteflower", "White Flower"),
        ("Greencrystal", "Green Crystal"),
        ("Redcrystal", "Red Crystal"),
        ("Bluecrystal", "Blue Crystal"),
        ("Orangecrystal", "Orange Crystal"),
        ("Yellowcrystal", "Yellow Crystal"),
        ("Blackcrystal", "Black Crystal"),
        ("Violetcrystal", "Violet Crystal"),
        ("Goldcrystal", "Gold Crystal"),
        ("Shadowcrystal", "Shadow Crystal"),
        ("Fellcrystal", "Fell Crystal"),
        ("Greatorangecrystal", "Great Orange Crystal"),
        ("Greatredcrystal", "Great Red Crystal"),
        ("Greatbluecrystal", "Great Blue Crystal"),
        ("Purplecrystals", "Purple Crystals"),
        ("Spiderteeth", "Spider Teeth"),
        ("Redgrapes", "Red Grapes"),
        ("Greengrapes", "Green Grapes"),
        ("Bluegrapes", "Blue Grapes"),
        ("Tomatos", "Tomatoes"),
        ("Cookingknife", "Cooking Knife"),
        ("Fishingrod", "Fishing Rod"),
        ("Oceanfish", "Ocean Fish"),
        ("Chees", "Cheese"),
        ("Chicken Ready", "Roast Chicken"),
        ("Meat Ready", "Roast Meat"),
        ("Fish Ready", "Grilled Fish"),
        ("Spareribs", "Spare Ribs"),
        ("Iron Patch", "Iron Plate"),
        ("Flake Patch", "Scale Plate"),
        ("Rune Patch", "Runed Plate"),
        ("Chiken Leg", "Chicken Leg"),
        ("Dragon Tale", "Dragon Tail"),
        ("Tyger Skin", "Tiger Skin"),
        ("Greattyger Skin", "Great Tiger Skin"),
        ("Fur Wolf", "Wolf Fur"),
        ("Cooperbar", "Copper Bar"),
        ("Goldenbar", "Gold Bar"),
        ("Fragments Of Stones", "Stone Fragments"),
        ("Hardstone", "Hard Stone"),
        ("Ghostore", "Ghost Ore"),
        ("Peacockore", "Peacock Ore"),
        ("Magicore", "Magic Ore"),
        ("Ancientore", "Ancient Ore"),
        ("Runepart", "Rune Fragment"),
        ("Jaspillite", "Jaspilite"),
        ("Rainbow Pyrite", "Rainbow Pyrite"),
        ("Magicdust", "Magic Dust"),
        ("Reactive Mixture", "Alacrity Mixture"),
        ("Reactive Potion", "Alacrity Potion"),
        ("Heal Potion", "Healing Potion"),
        ("Magicpotion", "Arcane Elixir"),
        ("Manapotion", "Mana Potion"),
        ("Healthpotion", "Healing Potion"),
        ("Medicines", "Restorative Medicine"),
        ("Littlemana Flask", "Minor Mana Flask"),
        ("Littleheal Flask", "Minor Healing Flask"),
        ("Bigmana Flask", "Greater Mana Flask"),
        ("Bigheal Flask", "Greater Healing Flask"),
        ("Hugeheal Flask", "Grand Healing Flask"),
        ("Hugemana Flask", "Grand Mana Flask"),
        ("Bigenergy Flask", "Greater Energy Flask"),
        ("Middle Flask", "Standard Flask"),
        ("Middleheal Flask", "Standard Healing Flask"),
        ("Middlemagical Flask", "Standard Arcane Flask"),
        ("Middleshadow Flask", "Standard Shadow Flask"),
        ("Middleenergy Flask", "Standard Energy Flask"),
        ("Middlemana Flask", "Standard Mana Flask"),
        ("Hugegreen Flask", "Grand Healing Tonic"),
        ("Hugeshadow Flask", "Grand Shadow Tonic"),
        ("Hugepoison Flask", "Grand Venom Coating"),
        ("Hugedark Flask", "Grand Dark Tonic"),
        ("Hugemagic Flask", "Grand Arcane Tonic"),
        ("Minipotions", "Miniature Potion Kit"),
        ("Clothroll", "Cloth Roll"),
        ("Magic Cloth", "Enchanted Cloth"),
        ("Spirit Cloth", "Spiritweave Cloth"),
        ("Demon Cloth", "Demonweave Cloth"),
        ("Frost Cloth", "Frostweave Cloth"),
        ("White Clothroll", "White Cloth Roll"),
        ("Gold Clothroll", "Gold Cloth Roll"),
        ("Blue Clothroll", "Blue Cloth Roll"),
        ("Green Clothroll", "Green Cloth Roll"),
        ("Red Clothroll", "Red Cloth Roll"),
        ("Magicthreads", "Enchanted Thread"),
        ("Magic Yarn", "Enchanted Yarn"),
        ("Magic Scissors", "Enchanted Scissors"),
        ("Silkcloth", "Silk Cloth"),
        ("Fiercloth", "Fireweave Cloth"),
        ("Little Bag", "Small Bag"),
        ("Magic Bag", "Enchanted Bag"),
        ("Enchantment Bag", "Enchanter's Bag"),
        ("Bigred Bag", "Large Red Bag"),
        ("Bigblack Bag", "Large Black Bag"),
        ("Miner Bag", "Miner's Bag"),
        ("Bigmagic Bag", "Large Enchanted Bag"),
        ("Easybandages", "Simple Bandages"),
        ("Silkbandages", "Silk Bandages"),
        ("Magicbandages", "Enchanted Bandages"),
        ("Tightbandages", "Compression Bandages"),
        ("Runebandages", "Rune Bandages"),
        ("Venombandages", "Antivenom Bandages"),
        ("Frostbandages", "Frostweave Bandages"),
        ("Sheepskin", "Sheepskin"),
        ("Whitebear Fur", "White Bear Fur"),
        ("Bearpaw", "Bear Paw"),
        ("Sharptooth", "Sharp Tooth"),
        ("Dragonhead", "Dragon Head"),
        ("Snakehead", "Snake Head"),
        ("Birdhead", "Bird Head"),
        ("Daggeroflove", "Dagger Of Love"),
        ("Training Sword Wood", "Wooden Training Sword"),
        ("Axe Old", "Old Axe"),
        ("Loot Axeold", "Old Raider's Axe"),
        ("Elite Dagger Gold", "Gilded Elite Dagger"),
        ("Elite Dagger Blue", "Blue-steel Elite Dagger"),
        ("Axe Hard", "Heavy Axe"),
        ("Axe Viking", "Viking Axe"),
        ("Wood Shield", "Wooden Shield"),
        ("Wood Shield Black Yellow", "Black-and-gold Wooden Shield"),
        ("Wood Shield Blue White", "Blue-and-white Wooden Shield"),
        ("Wood Shield Blue Yellow", "Blue-and-gold Wooden Shield"),
        ("Wood Shield Green", "Green Wooden Shield"),
        ("Wood Shield Red", "Red Wooden Shield"),
        ("Greece Shield", "Greek Shield"),
        ("Knight Shield Green", "Green Knight Shield"),
        ("Rome Dagger", "Roman Dagger"),
        ("Rome Shield", "Roman Shield"),
        ("Shield Buckler", "Buckler"),
        ("Spear Scythe", "War Scythe"),
        ("Spear Tournament Blue", "Blue Tournament Spear"),
        ("Spear Tournament Red", "Red Tournament Spear"),
        ("Sword Decorated", "Decorated Sword"),
        ("Sword Epee", "Epee"),
        ("Sword Rapier", "Rapier"),
        ("Sword Saber", "Saber"),
        ("Sword Twohanded", "Greatsword"),
        ("Spearhalberd", "Halberd"),
        ("Axe Pick", "War Pick"),
        ("Fat Dagger", "Broad Dagger"),
        ("Knife Dagger", "Fighting Knife"),
        ("The Whip", "Whip"),
        ("Stick", "Wooden Staff"),
        ("Assassins Dagger", "Assassin's Dagger"),
        ("Helm Broken", "Broken Helm"),
        ("Metal Helmet Gold", "Gilded Steel Helmet"),
        ("Ring S", "Silver Ring"),
        ("Barbarian Chest", "Barbarian Cuirass"),
        ("Chest Green E", "Green Tunic"),
        ("Cuirass Red", "Red Cuirass"),
        ("Cuirass Yellow", "Gilded Cuirass"),
        ("Fur Chest", "Fur Jerkin"),
        ("Gambesons", "Gambeson"),
        ("Kings Armor", "King's Armor"),
        ("Knight Chest", "Knight's Cuirass"),
        ("Leather Chest", "Leather Jerkin"),
        ("Mail Chest", "Chainmail Hauberk"),
        ("Mail Chest Red", "Red Chainmail Hauberk"),
        ("Plate Mail Chest", "Plate Cuirass"),
        ("Plate Mail Chest Blue", "Blue Plate Cuirass"),
        ("Plate Mail Chest Purple", "Purple Plate Cuirass"),
        ("Plate Mail Chest Red", "Red Plate Cuirass"),
        ("Plate Mail Chest Yellow", "Gilded Plate Cuirass"),
        ("Padded Armor Chest", "Padded Jack"),
        ("Rome Armor", "Roman Armor"),
        ("Thick Broun Gambleson", "Thick Brown Gambeson"),
        ("Boots S", "Simple Boots"),
        ("Trash Boots", "Tattered Boots"),
        ("Bandage", "Head Bandage"),
        ("Frog Helmet Stechzeug", "Frog-mouth Jousting Helm"),
        ("Gladiators Helm", "Gladiator's Helm"),
        ("Head Mail", "Chainmail Coif"),
        ("Helm S", "Simple Helm"),
        ("Quest Mask", "Mystery Mask"),
        ("Bracelet B", "Bronze Bracelet"),
        ("Necklace Cross", "Cross Pendant"),
        ("Neck B", "Bronze Necklace"),
        ("Ring B", "Bronze Ring"),
        ("Chest S", "Simple Tunic"),
        ("Chest Farmer", "Farmer's Tunic"),
        ("Tabard Clean", "Clean Tabard"),
        ("Boots Common", "Common Boots"),
        ("Hands S", "Simple Gloves"),
        ("Cloth Head", "Cloth Hood"),
        ("Leather Head", "Leather Coif"),
        ("Mail Head", "Chainmail Coif"),
        ("Formicidae", "Formicid"),
        ("Kuo Toa", "Kuo-toa"),
        ("Yuan Ti", "Yuan-ti"),
        ("Mindflayer", "Mind Flayer"),
        ("Stoneman", "Stone Golem"),
        ("Three Headed Dog", "Cerberus"),
        ("Clerics Ward", "Cleric's Ward"),
        ("Natures Touch", "Nature's Touch"),
        ("Assassins Mark", "Assassin's Mark"),
        ("Defenders Oath", "Defender's Oath"),
    ];
    if let Some((_, replacement)) =
        corrections.iter().find(|(source, _)| name.eq_ignore_ascii_case(source))
    {
        name = (*replacement).to_string();
    }

    name
}

/// Performs the contains any operation.
fn contains_any(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

/// Infers an elemental kind from the dominant saturated hue in an icon.
#[cfg(not(target_arch = "wasm32"))]
fn visual_kind(image_path: &Path) -> Option<&'static str> {
    let image = image::open(image_path).ok()?.to_rgba8();
    let mut scores = [0.0f64; 5];
    let mut colored_weight = 0.0f64;

    for pixel in image.pixels() {
        let [r, g, b, a] = pixel.0;
        if a < 32 {
            continue;
        }
        let r = r as f64 / 255.0;
        let g = g as f64 / 255.0;
        let b = b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let chroma = max - min;
        if max < 0.12 || chroma < 0.10 {
            continue;
        }

        let saturation = chroma / max;
        let alpha = a as f64 / 255.0;
        let weight = saturation * max * alpha;
        let hue = if max == r {
            60.0 * ((g - b) / chroma).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / chroma + 2.0)
        } else {
            60.0 * ((r - g) / chroma + 4.0)
        };

        let bucket = match hue {
            h if !(35.0..335.0).contains(&h) => 0, // Fire: red and magenta-red.
            h if h < 75.0 => 3,                    // Holy: gold and yellow.
            h if h < 165.0 => 2,                   // Nature: green.
            h if h < 250.0 => 1,                   // Ice: cyan and blue.
            _ => 4,                                // Shadow: violet and purple.
        };
        scores[bucket] += weight;
        colored_weight += weight;
    }

    let (winner, score) = scores.iter().copied().enumerate().max_by(|a, b| a.1.total_cmp(&b.1))?;
    if colored_weight < 8.0 || score / colored_weight < 0.38 {
        return None;
    }
    Some(["Fire", "Ice", "Nature", "Holy", "Shadow"][winner])
}

/// Disables image-based classification in WebAssembly build-tool stubs.
#[cfg(target_arch = "wasm32")]
fn visual_kind(_image_path: &Path) -> Option<&'static str> {
    None
}

/// Classifies text semantically, optionally falling back to the icon's dominant hue.
fn classify_kind(text: &str, image_path: &Path, infer_from_image: bool) -> &'static str {
    let lower = text.to_lowercase();
    if contains_any(
        &lower,
        &[
            "shadow",
            "dark",
            "curse",
            "vampir",
            "agony",
            "soul",
            "death",
            "devil",
            "demon",
            "unholy",
            "evil",
            "hex",
            "plague",
            "fear",
            "terror",
            "ghoul",
            "doom",
            "necrom",
            "void",
            "abyss",
            "nightmare",
        ],
    ) {
        "Shadow"
    } else if contains_any(
        &lower,
        &[
            "holy", "smite", "divine", "radiance", "judgment", "sacred", "bastion", "light",
            "heal", "bless", "angel", "glory", "prayer", "priest", "cleric", "paladin", "seraph",
        ],
    ) {
        "Holy"
    } else if contains_any(
        &lower,
        &[
            "fire", "pyro", "flame", "infernal", "burn", "cinder", "combust", "lava", "sunfire",
            "phoenix", "heat", "ash", "meteor", "scorch", "magma",
        ],
    ) {
        "Fire"
    } else if contains_any(
        &lower,
        &[
            "frost", "ice", "chill", "cold", "blizzard", "glacial", "freeze", "snow", "hail",
            "winter", "shiver", "rime",
        ],
    ) {
        "Ice"
    } else if contains_any(
        &lower,
        &[
            "nature", "bramble", "wild", "thorn", "bloom", "oak", "earth", "growth", "root",
            "leaf", "spore", "ivy", "forest", "grove", "poison", "venom", "toxic", "beast",
            "serpent",
        ],
    ) {
        "Nature"
    } else if infer_from_image {
        visual_kind(image_path).unwrap_or("Physical")
    } else {
        "Physical"
    }
}

/// Returns thematic suffixes for a combat kind.
fn kind_suffixes(kind: &str) -> &'static [&'static str] {
    match kind {
        "Fire" => FIRE_SUFFIXES,
        "Ice" => ICE_SUFFIXES,
        "Nature" => NATURE_SUFFIXES,
        "Holy" => HOLY_SUFFIXES,
        "Shadow" => SHADOW_SUFFIXES,
        _ => PHYSICAL_SUFFIXES,
    }
}

/// Returns construction details that produce plausible variants of the same equipment family.
fn equipment_traits(base: &str) -> &'static [&'static str] {
    let lower = base.to_lowercase();
    if contains_any(&lower, &["sword", "dagger", "rapier", "saber", "epee", "knife"]) {
        &[
            "Honed",
            "Fullered",
            "Broad",
            "Long",
            "Swept-hilt",
            "Basket-hilt",
            "Fluted",
            "Etched",
            "Riveted",
            "Ceremonial",
            "Dueling",
            "War",
        ]
    } else if contains_any(&lower, &["axe", "hammer", "mace", "club", "pick"]) {
        &[
            "Heavy",
            "Bearded",
            "Flanged",
            "Riveted",
            "Spiked",
            "Reinforced",
            "Etched",
            "War",
            "Forged",
            "Iron-bound",
            "Ceremonial",
            "Runed",
        ]
    } else if contains_any(&lower, &["bow", "crossbow"]) {
        &[
            "Recurved",
            "Composite",
            "Horn-backed",
            "Laminated",
            "Long",
            "Short",
            "Reinforced",
            "Hunter's",
            "War",
            "Etched",
            "Ceremonial",
            "Runed",
        ]
    } else if contains_any(&lower, &["wand", "staff", "book", "tome"]) {
        &[
            "Carved",
            "Gnarled",
            "Crystal-tipped",
            "Runed",
            "Etched",
            "Inlaid",
            "Resonant",
            "Ceremonial",
            "Scholar's",
            "Arcane",
            "Spiral",
            "Jeweled",
        ]
    } else if contains_any(
        &lower,
        &["shield", "armor", "chest", "cuirass", "hauberk", "helm", "boots", "gloves"],
    ) {
        &[
            "Riveted",
            "Reinforced",
            "Layered",
            "Fluted",
            "Embossed",
            "Etched",
            "Polished",
            "Iron-bound",
            "Ceremonial",
            "Tournament",
            "Runed",
            "Jeweled",
        ]
    } else {
        &[
            "Etched",
            "Reinforced",
            "Polished",
            "Riveted",
            "Engraved",
            "Inlaid",
            "Ceremonial",
            "Artisan",
            "Runed",
            "Jeweled",
            "Ancient",
            "Pristine",
        ]
    }
}

/// Produces a globally unique, level- and kind-aware equipment name.
fn unique_name(
    base: &str,
    level: u32,
    variant: usize,
    kind: &str,
    seen: &mut HashSet<String>,
) -> String {
    let candidate = capitalize_words(base);
    if seen.insert(candidate.clone()) {
        return candidate;
    }

    let tier = ((level.saturating_sub(1) / 4) as usize).min(QUALITY_PREFIXES.len() - 1);
    for prefix_offset in 0..QUALITY_PREFIXES[tier].len() {
        let prefix =
            QUALITY_PREFIXES[tier][(variant + prefix_offset) % QUALITY_PREFIXES[tier].len()];
        let candidate = capitalize_words(&format!("{prefix} {base}"));
        if seen.insert(candidate.clone()) {
            return candidate;
        }
        for suffix_offset in 0..kind_suffixes(kind).len() {
            let suffixes = kind_suffixes(kind);
            let suffix = suffixes[(variant + suffix_offset) % suffixes.len()];
            let candidate = capitalize_words(&format!("{prefix} {base} {suffix}"));
            if seen.insert(candidate.clone()) {
                return candidate;
            }
        }
    }

    for trait_name in equipment_traits(base) {
        let candidate = capitalize_words(&format!("{trait_name} {base}"));
        if seen.insert(candidate.clone()) {
            return candidate;
        }
        for suffix in kind_suffixes(kind) {
            let candidate = capitalize_words(&format!("{trait_name} {base} {suffix}"));
            if seen.insert(candidate.clone()) {
                return candidate;
            }
        }
    }

    let mut edition = 2;
    loop {
        let candidate = capitalize_words(&format!("{base} Mark {edition}"));
        if seen.insert(candidate.clone()) {
            return candidate;
        }
        edition += 1;
    }
}

/// Produces a unique artifact name using material-appropriate quality language.
fn unique_artifact_name(
    base: &str,
    level: u32,
    variant: usize,
    kind: &str,
    group: &str,
    seen: &mut HashSet<String>,
) -> String {
    let candidate = capitalize_words(base);
    if seen.insert(candidate.clone()) {
        return candidate;
    }

    let lower = base.to_lowercase();
    let qualifiers: &[&str] = if contains_any(&lower, &["meat", "sausage", "ribs"]) {
        &["Fresh", "Cured", "Smoked", "Salted", "Choice", "Dried", "Roasted", "Marbled"]
    } else if lower.contains("pepper") {
        &["Red", "Green", "Yellow", "Hot", "Sweet", "Dried", "Smoked", "Spiced"]
    } else if group == "cooking" {
        &["Fresh", "Ripe", "Dried", "Smoked", "Preserved", "Choice", "Spiced", "Artisan"]
    } else if contains_any(
        &lower,
        &["rune", "crystal", "scroll", "magic", "mystic", "enchant", "relic"],
    ) {
        &[
            "Faint", "Etched", "Polished", "Charged", "Resonant", "Greater", "Pristine", "Ancient",
            "Royal", "Mythic",
        ]
    } else if contains_any(&lower, &["iron", "steel", "ore", "ingot", "bar", "plate"]) {
        &["Wrought", "Forged", "Tempered", "Hardened", "Refined", "Dense", "Pristine", "Masterwork"]
    } else if contains_any(&lower, &["cloth", "leather", "skin", "patch", "fur"]) {
        &["Raw", "Cured", "Reinforced", "Tempered", "Refined", "Dense", "Pristine", "Masterwork"]
    } else {
        &["Small", "Clean", "Preserved", "Polished", "Fine", "Rare", "Pristine", "Ancient"]
    };
    let qualifier_start = if group == "cooking" {
        variant
    } else {
        level.saturating_sub(1) as usize / 2
    };

    for offset in 0..qualifiers.len() {
        let qualifier = qualifiers[(qualifier_start + offset) % qualifiers.len()];
        let candidate = capitalize_words(&format!("{qualifier} {base}"));
        if seen.insert(candidate.clone()) {
            return candidate;
        }
    }
    for qualifier_offset in 0..qualifiers.len() {
        for suffix_offset in 0..kind_suffixes(kind).len() {
            let qualifier = qualifiers[(qualifier_start + qualifier_offset) % qualifiers.len()];
            let suffixes = kind_suffixes(kind);
            let suffix = suffixes[(variant + suffix_offset) % suffixes.len()];
            let candidate = capitalize_words(&format!("{qualifier} {base} {suffix}"));
            if seen.insert(candidate.clone()) {
                return candidate;
            }
        }
    }

    unique_name(base, level, variant, kind, seen)
}

/// Returns a unique consumable name using potency language appropriate to its level.
fn unique_consumable_name(
    base: &str,
    level: u32,
    variant: usize,
    seen: &mut HashSet<String>,
) -> String {
    let candidate = capitalize_words(base);
    if seen.insert(candidate.clone()) {
        return candidate;
    }

    let tier = ((level.saturating_sub(1) / 4) as usize).min(CONSUMABLE_PREFIXES.len() - 1);
    for offset in 0..CONSUMABLE_PREFIXES[tier].len() {
        let prefix =
            CONSUMABLE_PREFIXES[tier][(variant + offset) % CONSUMABLE_PREFIXES[tier].len()];
        let candidate = capitalize_words(&format!("{prefix} {base}"));
        if seen.insert(candidate.clone()) {
            return candidate;
        }
    }

    let mut batch = 2;
    loop {
        let candidate = capitalize_words(&format!("{base} Formula {batch}"));
        if seen.insert(candidate.clone()) {
            return candidate;
        }
        batch += 1;
    }
}

/// Lists the PNG filenames directly inside a directory.
fn list_png_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "png" {
                        if let Some(name) = path.file_name() {
                            files.push(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    files
}

/// Performs the list image files operation.
fn list_image_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if ext_lower == "png" || ext_lower == "jpg" || ext_lower == "jpeg" {
                        if let Some(name) = path.file_name() {
                            files.push(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    files
}

/// Performs the img name operation.
fn img_name(filename: &str, img_ext: &str) -> String {
    let path = Path::new(filename);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "jpg" || ext == "jpeg" {
        filename.to_string()
    } else {
        format!("{}.{}", path.file_stem().unwrap().to_str().unwrap(), img_ext)
    }
}

/// Chooses an on-hit effect that matches a monster's identity and level.
fn monster_effects(name: &str, level: u32) -> Vec<String> {
    let mut effs = Vec::new();
    let name_lower = name.to_lowercase();

    if level == 1 && name_lower == "snake" {
        effs.push("Poison(damage: 1, duration: 0.5)".to_string());
    } else if level == 1 && ["weasel", "rat", "fox"].contains(&name_lower.as_str()) {
        effs.push("Cleave(damage_pct: 60.0, duration: 0.0)".to_string());
    } else if ["hell hound", "cerberus", "red", "fire troll", "ember drake"]
        .iter()
        .any(|x| name_lower.contains(x))
    {
        effs.push(format!("Burn(damage: {}, duration: 3.0)", 1 + level.div_ceil(2)));
    } else if ["snake", "spider", "basilisk", "yuan-ti", "formicid", "bog imp", "wyrm", "green"]
        .iter()
        .any(|x| name_lower.contains(x))
    {
        let damage = 1 + level.div_ceil(3);
        effs.push(format!("Poison(damage: {damage}, duration: 4.0)"));
    } else if [
        "medusa",
        "lich",
        "skeleton",
        "drow",
        "aboleth",
        "mind flayer",
        "grave warden",
        "mire hag",
        "black",
    ]
    .iter()
    .any(|x| name_lower.contains(x))
    {
        effs.push(format!("Curse(damage: {}, timer: 3)", 3 + level));
    } else if ["ice troll", "blue", "silver", "winter", "frost"]
        .iter()
        .any(|x| name_lower.contains(x))
    {
        effs.push(format!(
            "Freeze(attack_speed_pct: -{:.1}, duration: 2.5)",
            8.0 + level as f32 * 0.8
        ));
    } else if [
        "bear",
        "crocodile",
        "ogre",
        "mountain troll",
        "stone golem",
        "tarrasque",
        "owlbear",
        "worg",
        "balor",
        "bone colossus",
        "crimson minotaur",
        "werewolf",
        "abyssal behemoth",
        "badger",
        "boar",
    ]
    .iter()
    .any(|x| name_lower.contains(x))
    {
        effs.push(format!("Bleed(damage_pct: {:.1})", 8.0 + level as f32 * 2.0));
    } else if ["bat", "owl", "vulture", "storm harpy", "raven"]
        .iter()
        .any(|x| name_lower.contains(x))
    {
        effs.push(format!("Blind(miss_pct: {:.1}, duration: 2.0)", 8.0 + level as f32));
    } else if ["weasel", "rat", "hyena", "puma", "fox"].iter().any(|x| name_lower.contains(x)) {
        effs.push(format!("Cleave(damage_pct: {:.1}, duration: 0.0)", 8.0 + level as f32));
    } else if ["unicorn", "pegasus", "empyrean", "gold", "angel"]
        .iter()
        .any(|x| name_lower.contains(x))
    {
        effs.push(format!("Regen(heal: {}, duration: 3.0)", 1 + level.div_ceil(4)));
    } else if ["griffin", "manticore", "tiger", "void reaver", "lynx", "shadow panther"]
        .iter()
        .any(|x| name_lower.contains(x))
    {
        effs.push(format!(
            "Vulnerability(damage_pct: {:.1}, duration: 2.5)",
            4.0 + level as f32 * 0.6
        ));
    } else {
        effs.push(format!("Pierce(damage: {})", 2 + level));
    }

    effs
}

/// Assigns a deterministic tactical archetype from a monster's identity and family.
fn monster_archetype(name: &str, kind: &str) -> &'static str {
    let lower = name.to_lowercase();
    if kind == "Pet"
        || contains_any(
            &lower,
            &[
                "beast",
                "bear",
                "boar",
                "crocodile",
                "griffin",
                "hound",
                "hydra",
                "manticore",
                "owlbear",
                "tarrasque",
                "tiger",
                "wolf",
                "worg",
                "wyrm",
            ],
        )
    {
        "Beast"
    } else if contains_any(
        &lower,
        &["lich", "skeleton", "grave", "wraith", "hag", "zombie", "bone colossus"],
    ) {
        "Necromancer"
    } else if contains_any(
        &lower,
        &["golem", "knight", "warden", "minotaur", "lizardfolk", "empyrean"],
    ) {
        "Knight"
    } else if contains_any(
        &lower,
        &["drow", "reaver", "harpy", "wererat", "medusa", "rakshasa", "yuan-ti"],
    ) {
        "Assassin"
    } else if contains_any(
        &lower,
        &["vampire", "aboleth", "mind flayer", "leech", "mire", "kuo-toa"],
    ) {
        "Leech"
    } else if kind == "Dragon"
        || contains_any(&lower, &["imp", "basilisk", "formicid", "gnoll", "goblin", "mage"])
    {
        "Mage"
    } else {
        "Berserker"
    }
}

/// Performs the classify artifact kind operation.
fn classify_artifact_kind(name: &str) -> &'static str {
    let name_lower = name.to_lowercase();
    if name_lower.contains("shadow")
        || name_lower.contains("bone")
        || name_lower.contains("scull")
        || name_lower.contains("skull")
        || name_lower.contains("remains")
        || name_lower.contains("ghost")
        || name_lower.contains("ectoplasm")
        || name_lower.contains("death")
        || name_lower.contains("demon")
        || name_lower.contains("skeleton")
        || name_lower.contains("zombie")
        || name_lower.contains("goblin")
        || name_lower.contains("black crystal")
    {
        "Shadow"
    } else if name_lower.contains("holy")
        || name_lower.contains("cross")
        || name_lower.contains("healing")
        || name_lower.contains("order")
        || name_lower.contains("light")
        || name_lower.contains("angel")
        || name_lower.contains("temple")
        || name_lower.contains("sacred")
        || name_lower.contains("bible")
        || name_lower.contains("shrine")
        || name_lower.contains("rosary")
    {
        "Holy"
    } else if name_lower.contains("frost")
        || name_lower.contains("ice")
        || name_lower.contains("raindrop")
        || name_lower.contains("snow")
        || name_lower.contains("whitebear")
        || name_lower.contains("cold")
        || name_lower.contains("water")
        || name_lower.contains("blue crystal")
    {
        "Ice"
    } else if name_lower.contains("fire")
        || name_lower.contains("fiercloth")
        || name_lower.contains("torch")
        || name_lower.contains("dragon")
        || name_lower.contains("sunflower")
        || name_lower.contains("coal")
        || name_lower.contains("barbecue")
        || name_lower.contains("red crystal")
        || name_lower.contains("orange crystal")
    {
        "Fire"
    } else if name_lower.contains("herb")
        || name_lower.contains("flower")
        || name_lower.contains("leaf")
        || name_lower.contains("leaves")
        || name_lower.contains("mushroom")
        || name_lower.contains("plant")
        || name_lower.contains("moss")
        || name_lower.contains("seaweed")
        || name_lower.contains("root")
        || name_lower.contains("seed")
        || name_lower.contains("grass")
        || name_lower.contains("wood")
        || name_lower.contains("branch")
        || name_lower.contains("bark")
        || name_lower.contains("twig")
        || name_lower.contains("sprout")
        || name_lower.contains("dill")
        || name_lower.contains("rucola")
        || name_lower.contains("basilicum")
        || name_lower.contains("parsley")
        || name_lower.contains("rose")
        || name_lower.contains("tulip")
        || name_lower.contains("asparag")
        || name_lower.contains("cactus")
        || name_lower.contains("spider")
        || name_lower.contains("poison")
        || name_lower.contains("venom")
        || name_lower.contains("green crystal")
    {
        "Nature"
    } else {
        "Physical"
    }
}

/// Extracts an artifact's display name and profession group from its filename.
fn artifact_name_parts(filename: &str, variant: usize) -> (String, String) {
    let stem = Path::new(filename).file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
    let (prefix, remainder) = stem.split_once('_').unwrap_or(("", stem));
    let recognized_prefixes = [
        "blacksmith",
        "cooking",
        "enchantment",
        "herbalism",
        "jewelry",
        "loot",
        "mining",
        "quest",
        "res",
        "skinning",
        "tailoring",
    ];
    let (group, source_name) = if recognized_prefixes.contains(&prefix.to_lowercase().as_str()) {
        (
            prefix.to_lowercase(),
            remainder
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .trim_start_matches(['_', '-', ' ']),
        )
    } else {
        ("misc".to_string(), stem)
    };

    let mut name = clean_name(source_name);
    if name.is_empty() {
        let fallbacks: &[&str] = match group.as_str() {
            "quest" => &[
                "Sealed Quest Relic",
                "Pilgrim's Token",
                "Forgotten Quest Seal",
                "Ancient Map Fragment",
            ],
            "loot" => &["Sealed Cache", "Weathered Trophy", "Forgotten Keepsake", "Monster Relic"],
            "res" => {
                &["Unknown Reagent", "Preserved Reagent", "Rare Reagent", "Alchemical Reagent"]
            },
            _ => &["Unidentified Relic", "Crafting Relic", "Old Curio", "Strange Component"],
        };
        name = fallbacks[variant % fallbacks.len()].to_string();
    }

    name = match name.as_str() {
        "Eye" => "Monster Eye".to_string(),
        "Goo" => "Ectoplasm".to_string(),
        "Shell" => "Ocean Shell".to_string(),
        "Sting" => "Venomous Stinger".to_string(),
        "Bark" => "Tree Bark".to_string(),
        "Bone" => "Ancient Bone".to_string(),
        "Horn" => "Animal Horn".to_string(),
        "Fur" => "Animal Fur".to_string(),
        "Piece Of Coal" => "Coal".to_string(),
        "Hook" if group == "cooking" => "Fishhook".to_string(),
        "Mountaineering Hook" => "Climbing Hook".to_string(),
        "Scroll" if group == "enchantment" => "Enchantment Scroll".to_string(),
        _ => name,
    };

    (name, group)
}

/// Assigns an artifact level from material rarity and its sequence within a profession.
fn artifact_level(filename: &str, name: &str, group: &str) -> u32 {
    let lower = format!("{} {}", filename, name).to_lowercase();
    let sequence = get_last_number(filename).unwrap_or(1.0).max(1.0) as u32;
    let mut level = match group {
        "cooking" => 1 + sequence.saturating_sub(1) / 10,
        "herbalism" | "mining" | "skinning" | "tailoring" => 1 + sequence.saturating_sub(1) / 3,
        "jewelry" => sequence,
        "blacksmith" => 1 + sequence.saturating_sub(10) / 2,
        "enchantment" => 2 + sequence.saturating_sub(1) / 3,
        "quest" | "loot" | "res" => 2 + sequence % 9,
        _ => 4,
    };

    if contains_any(
        &lower,
        &[
            "apple",
            "pear",
            "banana",
            "onion",
            "turnip",
            "cabbage",
            "carrot",
            "pepper",
            "tomato",
            "strawberry",
            "cherry",
            "water",
            "flour",
            "corn",
            "egg",
            "milk",
            "bread",
            "peanut",
        ],
    ) {
        level = level.min(2);
    }
    if contains_any(&lower, &["iron", "copper", "leather", "cloth", "wood", "coal", "clay"]) {
        level = level.max(2);
    }
    if contains_any(&lower, &["silver", "pearl", "cobalt", "thick", "crystal"]) {
        level = level.max(6);
    }
    if contains_any(&lower, &["gold", "mana", "rune", "mystic", "magic", "dragon"]) {
        level = level.max(10);
    }
    if contains_any(&lower, &["demon", "shadow", "ancient", "space", "death"]) {
        level = level.max(16);
    }
    level.clamp(1, 20)
}

/// Prices artifacts by rarity while keeping ordinary food and supplies inexpensive.
fn artifact_price(name: &str, group: &str, level: u32) -> u32 {
    let lower = name.to_lowercase();
    if group == "cooking"
        && contains_any(
            &lower,
            &[
                "apple", "pear", "banana", "onion", "turnip", "cabbage", "carrot", "pepper",
                "tomato", "water", "flour", "corn", "egg", "milk", "bread",
            ],
        )
    {
        return 1 + level * 2;
    }

    let base = 2 + level * 2 + level * level / 2;
    let multiplier = if contains_any(&lower, &["diamond", "ancient", "demon", "space"]) {
        2.0
    } else if contains_any(&lower, &["gold", "magic", "mystic", "rune", "dragon"]) {
        1.5
    } else if group == "cooking" {
        0.65
    } else {
        1.0
    };
    (base as f32 * multiplier).round().max(1.0) as u32
}

/// Builds level-scaled, semantically matched ability effects from its presentation name and icon.
///
/// The source filename carries useful visual context that can be absent from a cleaned display name,
/// such as `trapfrost`, `mindbreak`, or `petattack`.  Every branch keeps its effects on one target
/// side because an [`Ability`](crate::core::catalog::abilities::Ability) has one target declaration.
fn ability_effects(kind: &str, name: &str, filename: &str, level: u32) -> (Vec<String>, bool, f32) {
    let lower = format!("{name} {filename}").to_lowercase();
    let power = level as f32;
    let short_duration = 2.5 + power * 0.05;
    let duration = 3.5 + power * 0.08;
    let direct_damage = 3 + level * 3;
    let dot_damage = 1 + level.div_ceil(3);
    let is_direct = contains_any(
        &lower,
        &[
            "attack",
            "bolt",
            "blast",
            "beam",
            "arrow",
            "shot",
            "spear",
            "throw",
            "strike",
            "hit",
            "claw",
            "cut",
            "slash",
            "blow",
            "stab",
            "bomb",
            "explosion",
            "rain",
        ],
    );

    // Beneficial and utility effects take precedence over an elemental school: an Ice Block is a
    // defensive spell, for example, not an enemy slow merely because its icon is blue.
    if contains_any(&lower, &["purge", "dispel", "cleanse", "break chain", "unshackle"]) {
        return (vec!["Purge".to_string()], true, 0.0);
    }
    if contains_any(
        &lower,
        &[
            "heal",
            "healing",
            "renew",
            "restor",
            "tranquility",
            "recovery",
            "overheal",
            "natures touch",
            "nature's touch",
        ],
    ) {
        if contains_any(&lower, &["touch", "renew", "wave", "tranquility", "growth", "restor"]) {
            return (
                vec![format!("Regen(heal: {}, duration: {duration:.1})", 1 + level.div_ceil(2))],
                true,
                duration,
            );
        }
        return (vec![format!("Heal(heal_pct: {})", 12 + level * 2)], true, 0.0);
    }
    let fiery_wall = kind == "Fire" && contains_any(&lower, &["fire wall", "firewall"]);
    if contains_any(
        &lower,
        &[
            "shield",
            "armor",
            "ward",
            "guard",
            "aegis",
            "barrier",
            "wall",
            "stance",
            "defense",
            "defence",
            "skin",
            "bastion",
            "block",
            "fortress",
            "metalarmor",
            "resistance",
        ],
    ) && !fiery_wall
    {
        let mut effects =
            vec![format!("Fortify(defense_pct: {:.1}, duration: {duration:.1})", 8.0 + power)];
        if contains_any(&lower, &["thorn", "spine", "brist"]) {
            effects.push(format!(
                "Thorns(damage_reflected_pct: {:.1}, duration: {duration:.1})",
                6.0 + power * 0.8
            ));
        }
        return (effects, true, duration);
    }
    if contains_any(&lower, &["mana", "meditation", "rune", "arcane", "spirit", "mind", "science"])
        && !is_direct
        && !contains_any(&lower, &["drain", "burn", "steal"])
        && !contains_any(&lower, &["mindbreak", "mind control", "hypnosis", "obsession"])
    {
        if contains_any(&lower, &["meditation", "flow", "rune", "spirit"]) {
            return (
                vec![format!(
                    "ManaFlow(amount: {}, duration: {duration:.1})",
                    1 + level.div_ceil(4)
                )],
                true,
                duration,
            );
        }
        if contains_any(&lower, &["arcane", "magic", "clearcasting"]) {
            return (
                vec![format!(
                    "Clearcasting(reduction_pct: {:.1}, duration: {duration:.1})",
                    12.0 + power * 0.7
                )],
                true,
                duration,
            );
        }
        return (vec![format!("InstantMana(amount: {})", 4 + level * 2)], true, 0.0);
    }
    if contains_any(&lower, &["thorn", "spine", "spikes", "brist"]) {
        return (
            vec![format!(
                "Thorns(damage_reflected_pct: {:.1}, duration: {duration:.1})",
                6.0 + power * 0.8
            )],
            true,
            duration,
        );
    }
    if contains_any(&lower, &["taunt", "aggro", "provoke"]) {
        return (vec![format!("Taunt(duration: {duration:.1})")], true, duration);
    }
    if contains_any(
        &lower,
        &[
            "speed",
            "fast",
            "dash",
            "escape",
            "teleport",
            "portal",
            "jump",
            "move",
            "run",
            "fly",
            "evade",
            "dodge",
            "sneak",
            "stealth",
            "shadowing",
        ],
    ) {
        return (
            vec![format!("Haste(initiative_pct: {:.1}, duration: {duration:.1})", 7.0 + power)],
            true,
            duration,
        );
    }
    if contains_any(
        &lower,
        &["focus", "aim", "target", "precision", "third eye", "headshoot", "eye"],
    ) {
        return (
            vec![format!(
                "Focus(crit_chance_pct: {:.1}, duration: {duration:.1})",
                5.0 + power * 0.75
            )],
            true,
            duration,
        );
    }
    if contains_any(&lower, &["rage", "bloodlust", "anger", "ruthless", "fury", "berserk"]) {
        return (
            vec![format!("Berserk(attack_pct: {:.1}, duration: {duration:.1})", 8.0 + power)],
            true,
            duration,
        );
    }
    if contains_any(
        &lower,
        &[
            "pet", "beast", "animal", "wolf", "bear", "raven", "snake", "spider", "insect", "dog",
            "golem", "summon", "army", "clones",
        ],
    ) {
        return (
            vec![format!(
                "BeastFrenzy(attack_pct: {:.1}, attack_speed_pct: {:.1}, duration: {duration:.1})",
                6.0 + power * 0.8,
                5.0 + power * 0.6
            )],
            true,
            duration,
        );
    }
    if contains_any(&lower, &["vampir", "lifesteal", "devour", "voracity", "soul devouring"]) {
        return (
            vec![format!(
                "Lifesteal(percentage: {:.1}, duration: {duration:.1})",
                4.0 + power * 0.5
            )],
            true,
            duration,
        );
    }
    if contains_any(&lower, &["sharpen", "mastery", "weapon switch", "sword power", "domination"]) {
        return (
            vec![
                format!("Empower(damage_pct: {:.1}, duration: {duration:.1})", 6.0 + power),
                format!("Bleed(damage_pct: {:.1})", 8.0 + power * 1.5),
            ],
            true,
            duration.max(12.0),
        );
    }
    if contains_any(
        &lower,
        &["muscle", "strength", "power", "hard", "endurance", "health", "body", "discipline"],
    ) && !is_direct
    {
        let attribute = if contains_any(&lower, &["endurance", "health", "body", "hard"]) {
            "Constitution"
        } else {
            "Strength"
        };
        return (
            vec![format!(
                "StatBoost(attribute: {attribute}, amount: {}, duration: {duration:.1})",
                1 + level.div_ceil(5)
            )],
            true,
            duration,
        );
    }

    // Offensive control effects are keyed from their gameplay meaning before elemental fallbacks.
    if contains_any(
        &lower,
        &[
            "stun",
            "paralysis",
            "petrif",
            "timestop",
            "skullbreaker",
            "concuss",
            "hammerfall",
            "shock",
            "lightning",
            "mind control",
            "hypnosis",
        ],
    ) {
        return (vec![format!("Stun(duration: {short_duration:.1})")], false, short_duration);
    }
    if contains_any(&lower, &["silence", "mindbreak", "disarm", "mute"]) {
        return (vec![format!("Silence(duration: {duration:.1})")], false, duration);
    }
    if contains_any(&lower, &["blind", "smoke", "blackwater"]) {
        return (
            vec![format!("Blind(miss_pct: {:.1}, duration: {duration:.1})", 8.0 + power)],
            false,
            duration,
        );
    }
    if contains_any(
        &lower,
        &["root", "snare", "net", "trap", "chain", "rope", "lasso", "hook", "capture"],
    ) {
        let mut effects = vec![format!("Immobilize(duration: {duration:.1})")];
        if contains_any(&lower, &["poison", "venom", "toxic", "acid"]) {
            effects.push(format!("Poison(damage: {dot_damage}, duration: {duration:.1})"));
        } else if kind == "Ice" || contains_any(&lower, &["frost", "ice"]) {
            effects.push(format!(
                "Freeze(attack_speed_pct: -{:.1}, duration: {duration:.1})",
                7.0 + power
            ));
        }
        return (effects, false, duration);
    }
    if contains_any(&lower, &["fear", "terror", "nightmare", "howl", "cry", "demonic cry"]) {
        return (
            vec![format!("Paranoia(initiative_pct: -{:.1}, duration: {duration:.1})", 7.0 + power)],
            false,
            duration,
        );
    }
    if contains_any(
        &lower,
        &["curse", "hex", "doom", "plague", "demonic fate", "demon mark", "corrupt"],
    ) {
        return (
            vec![
                format!("Curse(damage: {}, timer: {})", 3 + level * 2, duration.ceil() as u32),
                format!(
                    "Vulnerability(damage_pct: {:.1}, duration: {duration:.1})",
                    5.0 + power * 0.7
                ),
            ],
            false,
            duration,
        );
    }
    if contains_any(&lower, &["mana burn", "manablast", "mana blast", "destroy mana"]) {
        return (vec![format!("ManaBurn(amount: {})", 4 + level * 2)], false, 0.0);
    }
    if contains_any(&lower, &["drain", "siphon", "mana steal", "manasteal"]) {
        return (vec![format!("Manasteal(percentage: {:.1})", 7.0 + power)], false, 0.0);
    }
    if contains_any(
        &lower,
        &["poison", "venom", "toxic", "acid", "infection", "bite", "sting", "scorpion"],
    ) {
        return (
            vec![format!("Poison(damage: {dot_damage}, duration: {duration:.1})")],
            false,
            duration,
        );
    }

    match kind {
        "Fire" if is_direct => (
            vec![
                format!("Pierce(damage: {direct_damage})"),
                format!("Burn(damage: {}, duration: {duration:.1})", 1 + level.div_ceil(2)),
            ],
            false,
            duration,
        ),
        "Fire" => (
            vec![format!("Burn(damage: {}, duration: {duration:.1})", 1 + level.div_ceil(2))],
            false,
            duration,
        ),
        "Ice" if is_direct => (
            vec![
                format!("Pierce(damage: {})", 2 + level * 2),
                format!("Freeze(attack_speed_pct: -{:.1}, duration: {duration:.1})", 7.0 + power),
            ],
            false,
            duration,
        ),
        "Ice" => (
            vec![format!("Freeze(attack_speed_pct: -{:.1}, duration: {duration:.1})", 7.0 + power)],
            false,
            duration,
        ),
        "Nature" if is_direct => (
            vec![
                format!("Pierce(damage: {})", 2 + level * 2),
                format!("Poison(damage: {dot_damage}, duration: {duration:.1})"),
            ],
            false,
            duration,
        ),
        "Nature" => (
            vec![format!("Poison(damage: {dot_damage}, duration: {duration:.1})")],
            false,
            duration,
        ),
        "Holy"
            if contains_any(
                &lower,
                &["smite", "judgment", "hammer", "wrath", "strike", "light", "sun", "holy"],
            ) =>
        {
            (vec![format!("Pierce(damage: {direct_damage})")], false, 0.0)
        },
        "Holy" => (
            vec![format!("Regen(heal: {}, duration: {duration:.1})", 1 + level.div_ceil(2))],
            true,
            duration,
        ),
        "Shadow" if is_direct => (
            vec![
                format!("Pierce(damage: {direct_damage})"),
                format!("Vulnerability(damage_pct: {:.1}, duration: {duration:.1})", 5.0 + power),
            ],
            false,
            duration,
        ),
        "Shadow" => (
            vec![format!("Vulnerability(damage_pct: {:.1}, duration: {duration:.1})", 5.0 + power)],
            false,
            duration,
        ),
        _ if contains_any(
            &lower,
            &["cleave", "wave", "whirl", "vortex", "spin", "flock", "thousand"],
        ) =>
        {
            (
                vec![format!("Cleave(damage_pct: {:.1}, duration: 0.0)", 16.0 + power * 1.8)],
                false,
                0.0,
            )
        },
        _ => (vec![format!("Pierce(damage: {direct_damage})")], false, 0.0),
    }
}

/// Builds semantic passive modifiers for a perk from its presentation name and icon.
fn perk_modifiers(kind: &str, name: &str, filename: &str, level: u32) -> Vec<String> {
    let lower = format!("{name} {filename}").to_lowercase();
    let stat = level.div_ceil(4).max(1) as i32;
    let percent = 3.0 + level as f32;
    let resource = 2 + level.div_ceil(4) as i32;
    let defensive = contains_any(
        &lower,
        &[
            "shield",
            "armor",
            "guard",
            "defense",
            "defence",
            "skin",
            "wall",
            "stone body",
            "body",
            "barrier",
            "fort",
            "plate",
            "protection",
            "ward",
        ],
    );
    let pet = contains_any(
        &lower,
        &[
            "pet",
            "beast",
            "animal",
            "wolf",
            "bear",
            "raven",
            "snake",
            "spider",
            "insect",
            "dog",
            "golem",
            "summon",
            "companion",
        ],
    );
    let resource_magic = contains_any(
        &lower,
        &["mana", "magic", "arcane", "rune", "mind", "science", "wizard", "mage"],
    );
    let healing =
        contains_any(&lower, &["heal", "restor", "renew", "life", "druid", "paladin", "priest"]);
    let fast = contains_any(
        &lower,
        &["speed", "fast", "run", "jump", "move", "agile", "flex", "evade", "dodge"],
    );
    let precise =
        contains_any(&lower, &["aim", "target", "focus", "eye", "precision", "headshoot", "sharp"]);
    let mut modifiers = Vec::new();

    if defensive {
        modifiers.push(format!("DefenseModifier({stat})"));
        if level >= 5 {
            modifiers.push(format!("MaxHealthModifier({})", level as i32 * 3));
        }
        if kind != "Physical" {
            modifiers.push(format!("KindResistanceMultiplier({kind}, {:.1})", percent * 0.75));
        }
    } else if pet {
        modifiers.push(format!("PetAttackModifier({stat})"));
        modifiers.push(format!("PetDefenseModifier({})", stat.max(1)));
        if level >= 7 {
            modifiers.push(format!("PetInitiativeModifier({})", stat.max(1)));
        }
    } else if healing {
        modifiers.push(format!("HealingMultiplier({percent:.1})"));
        modifiers.push(format!("HealthRegen({resource})"));
        if level >= 9 {
            modifiers.push(format!("MaxHealthModifier({})", level as i32 * 3));
        }
    } else if resource_magic {
        modifiers.push(format!("MaxManaModifier({})", level as i32 * 3));
        modifiers.push(format!("ManaRegen({resource})"));
        if kind != "Physical" {
            modifiers.push(format!("KindPowerMultiplier({kind}, {percent:.1})"));
        }
    } else if precise {
        modifiers.push(format!("CritChanceModifier({percent:.1})"));
        if level >= 5 {
            modifiers.push(format!("AttributeModifier(Dexterity, {stat})"));
        }
    } else if fast {
        modifiers.push(format!("InitiativeModifier({stat})"));
        modifiers.push(format!("AttackSpeedModifier({percent:.1})"));
    } else if contains_any(&lower, &["bow", "arrow", "shot", "archer", "crossbow", "gun", "hunter"])
    {
        modifiers.push(format!("CategoryPowerMultiplier(Range, {percent:.1})"));
        if level >= 5 {
            modifiers.push(format!("AttributeModifier(Dexterity, {stat})"));
        }
    } else if contains_any(&lower, &["dagger", "assassin", "rogue", "knife", "poison", "venom"]) {
        modifiers.push(format!("CategoryPowerMultiplier(Finesse, {percent:.1})"));
        modifiers.push(format!("CritChanceModifier({:.1})", percent * 0.5));
    } else if contains_any(
        &lower,
        &["sword", "axe", "hammer", "club", "spear", "warrior", "knight"],
    ) {
        modifiers.push(format!("CategoryPowerMultiplier(Melee, {percent:.1})"));
        if level >= 5 {
            modifiers.push(format!("AttributeModifier(Strength, {stat})"));
        }
    } else {
        match kind {
            "Fire" | "Nature" | "Shadow" => {
                modifiers.push(format!("KindPowerMultiplier({kind}, {percent:.1})"));
            },
            "Ice" => {
                modifiers.push(format!("KindResistanceMultiplier(Ice, {percent:.1})"));
                if level >= 5 {
                    modifiers.push(format!("KindPowerMultiplier(Ice, {:.1})", percent * 0.6));
                }
            },
            "Holy" => {
                modifiers.push(format!("HealingMultiplier({percent:.1})"));
                if level >= 5 {
                    modifiers.push(format!("KindPowerMultiplier(Holy, {:.1})", percent * 0.6));
                }
            },
            _ => {
                modifiers.push(format!("AttackModifier({stat})"));
                if level >= 5 {
                    modifiers.push(format!("AttributeModifier(Strength, {stat})"));
                }
            },
        }
    }

    if contains_any(&lower, &["vampir", "leech", "drain", "devour"]) {
        modifiers.push(format!("LifeSteal({:.1})", 1.5 + power_fraction(level)));
    }
    modifiers.sort();
    modifiers.dedup();
    modifiers
}

/// Returns the modest level-scaled percentage used by life-steal perk modifiers.
fn power_fraction(level: u32) -> f32 {
    level as f32 * 0.35
}

/// Adds a title- and category-driven specialty so visually distinct weapons play differently.
fn weapon_identity_modifier(
    filename: &str,
    name: &str,
    kind: &str,
    category: &str,
    level: u32,
    variant: usize,
) -> String {
    let identity = format!("{filename} {name}").to_lowercase();
    let stat = 1 + level / 5;
    let resource = 1 + level / 7;
    let percent = 1.5 + level as f32 * 0.35;

    if contains_any(&identity, &["warlord", "great", "heavy", "brutal"]) {
        format!("AttackModifier({stat})")
    } else if category == "Shield" || contains_any(&identity, &["sentinel", "vanguard", "guard"]) {
        format!("DefenseModifier({stat})")
    } else if category == "Finesse" || contains_any(&identity, &["duelist", "honed", "assassin"]) {
        format!("CritChanceModifier({percent:.1})")
    } else if category == "Range" || contains_any(&identity, &["hunter", "recurved"]) {
        format!("InitiativeModifier({stat})")
    } else if matches!(category, "Book" | "Magical")
        || contains_any(&identity, &["runed", "rune", "arcane"])
    {
        format!("ManaRegen({resource})")
    } else {
        match variant % 6 {
            0 => format!("AttackModifier({stat})"),
            1 => format!("CritChanceModifier({percent:.1})"),
            2 => format!("InitiativeModifier({stat})"),
            3 => format!("MaxHealthModifier({})", level * 3),
            4 if kind == "Physical" => format!("AttackSpeedModifier({percent:.1})"),
            4 => format!("KindPowerMultiplier({kind}, {percent:.1})"),
            _ => format!("MaxManaModifier({})", level * 2),
        }
    }
}

/// Builds a varied on-hit or defensive effect for higher-level weapons.
fn weapon_effects(kind: &str, category: &str, level: u32, variant: usize) -> Vec<String> {
    if level < 8 {
        return Vec::new();
    }
    let power = level as f32;
    let effect = match kind {
        "Fire" if variant.is_multiple_of(3) => {
            format!("Berserk(attack_pct: {:.1}, duration: 3.0)", 5.0 + power * 0.6)
        },
        "Fire" => format!("Burn(damage: {}, duration: 3.0)", 1 + level.div_ceil(3)),
        "Ice" if variant.is_multiple_of(3) => {
            format!("Fortify(defense_pct: {:.1}, duration: 3.0)", 7.0 + power * 0.7)
        },
        "Ice" => format!("Freeze(attack_speed_pct: -{:.1}, duration: 3.0)", 6.0 + power * 0.7),
        "Nature" if variant.is_multiple_of(3) => {
            format!("Regen(heal: {}, duration: 3.0)", 1 + level.div_ceil(5))
        },
        "Nature" => format!("Poison(damage: {}, duration: 4.0)", 1 + level.div_ceil(4)),
        "Shadow" if variant.is_multiple_of(3) => {
            format!("Lifesteal(percentage: {:.1}, duration: 3.0)", 3.0 + power * 0.35)
        },
        "Shadow" => {
            format!("Vulnerability(damage_pct: {:.1}, duration: 3.0)", 5.0 + power * 0.6)
        },
        "Holy" if variant.is_multiple_of(3) => {
            format!("Fortify(defense_pct: {:.1}, duration: 3.0)", 8.0 + power * 0.7)
        },
        "Holy" => format!("Regen(heal: {}, duration: 3.0)", 1 + level.div_ceil(5)),
        _ => match category {
            "Shield" if variant.is_multiple_of(2) => {
                format!("Fortify(defense_pct: {:.1}, duration: 3.0)", 7.0 + power * 0.7)
            },
            "Shield" => {
                format!("Thorns(damage_reflected_pct: {:.1}, duration: 3.0)", 5.0 + power)
            },
            "Book" | "Magical" if variant.is_multiple_of(2) => {
                format!("Empower(damage_pct: {:.1}, duration: 3.0)", 5.0 + power * 0.5)
            },
            "Book" | "Magical" => {
                format!("Clearcasting(reduction_pct: {:.1}, duration: 3.0)", 12.0 + power * 0.5)
            },
            "Finesse" if variant.is_multiple_of(2) => {
                format!("Focus(crit_chance_pct: {:.1}, duration: 3.0)", 4.0 + power * 0.4)
            },
            "Finesse" => {
                format!("Lifesteal(percentage: {:.1}, duration: 3.0)", 3.0 + power * 0.35)
            },
            "Range" if variant.is_multiple_of(2) => {
                format!("Pierce(damage: {})", 1 + level.div_ceil(3))
            },
            "Range" => format!("Blind(miss_pct: {:.1}, duration: 2.5)", 8.0 + power * 0.6),
            _ if variant.is_multiple_of(2) => {
                format!("Cleave(damage_pct: {:.1}, duration: 3.0)", 8.0 + power * 0.6)
            },
            _ => format!("Bleed(damage_pct: {:.1})", 10.0 + power),
        },
    };
    vec![effect]
}

/// Adds a small deterministic specialty when two weapons would otherwise share a build.
fn make_weapon_profile_unique(
    fixed_profile: &str,
    modifiers: &mut Vec<String>,
    effects: &[String],
    profiles: &mut HashSet<String>,
) {
    let mut collision = 0u32;
    loop {
        let profile = format!("{fixed_profile}|{}|{}", modifiers.join(","), effects.join(","));
        if profiles.insert(profile) {
            break;
        }
        collision += 1;
        modifiers.push(match collision % 4 {
            1 => format!("AttackModifier({collision})"),
            2 => format!("InitiativeModifier({collision})"),
            3 => format!("MaxHealthModifier({collision})"),
            _ => format!("MaxManaModifier({collision})"),
        });
    }
}

/// Adds a material- or title-driven specialty so visually distinct wearables play differently.
fn wearable_identity_modifier(
    filename: &str,
    name: &str,
    kind: &str,
    level: u32,
    variant: usize,
) -> String {
    let identity = format!("{filename} {name}").to_lowercase();
    let stat = 1 + level / 5;
    let resource = 1 + level / 7;
    let percent = 1.5 + level as f32 * 0.35;

    if contains_any(&identity, &["warlord", "veteran", "berserk", "war "]) {
        format!("AttackModifier({stat})")
    } else if contains_any(&identity, &["sentinel", "vanguard", "guard", "tower"]) {
        format!("DefenseModifier({stat})")
    } else if contains_any(&identity, &["duelist", "assassin", "hunter", "ranger"]) {
        format!("CritChanceModifier({percent:.1})")
    } else if contains_any(&identity, &["cloth", "runed", "rune", "arcane", "scholar"]) {
        format!("ManaRegen({resource})")
    } else if contains_any(&identity, &["mail", "chain", "plate", "iron", "steel"]) {
        format!("MaxHealthModifier({})", level * 3)
    } else if contains_any(&identity, &["leather", "hide", "swift", "speed"]) {
        format!("InitiativeModifier({stat})")
    } else {
        match variant % 8 {
            0 => format!("AttackModifier({stat})"),
            1 => format!("DefenseModifier({stat})"),
            2 => format!("InitiativeModifier({stat})"),
            3 => format!("CritChanceModifier({percent:.1})"),
            4 => format!("MaxHealthModifier({})", level * 3),
            5 => format!("MaxManaModifier({})", level * 2),
            6 => format!("HealthRegen({resource})"),
            _ if kind == "Physical" => format!("AttackSpeedModifier({percent:.1})"),
            _ => format!("KindPowerMultiplier({kind}, {percent:.1})"),
        }
    }
}

/// Builds a varied on-being-hit effect appropriate to a wearable's identity and element.
fn wearable_effects(
    kind: &str,
    filename: &str,
    name: &str,
    slot: &str,
    level: u32,
    variant: usize,
) -> Vec<String> {
    if level < 8 {
        return Vec::new();
    }
    let identity = format!("{filename} {name}").to_lowercase();
    let power = level as f32;
    let effect = match kind {
        "Fire" if variant.is_multiple_of(3) => {
            format!("Burn(damage: {}, duration: 3.0)", 1 + level.div_ceil(4))
        },
        "Fire" => format!("Berserk(attack_pct: {:.1}, duration: 3.0)", 6.0 + power * 0.7),
        "Ice" if variant.is_multiple_of(3) => {
            format!("Fortify(defense_pct: {:.1}, duration: 3.0)", 7.0 + power * 0.7)
        },
        "Ice" => format!("Freeze(attack_speed_pct: -{:.1}, duration: 2.5)", 6.0 + power * 0.6),
        "Nature" if variant.is_multiple_of(3) => {
            format!("Poison(damage: {}, duration: 4.0)", 1 + level.div_ceil(5))
        },
        "Nature" => format!("Regen(heal: {}, duration: 3.0)", 1 + level.div_ceil(5)),
        "Shadow" if variant.is_multiple_of(3) => {
            format!("Lifesteal(percentage: {:.1}, duration: 3.0)", 3.0 + power * 0.35)
        },
        "Shadow" => {
            format!("Paranoia(initiative_pct: -{:.1}, duration: 3.0)", 5.0 + power * 0.6)
        },
        "Holy" if variant.is_multiple_of(3) => {
            format!("Regen(heal: {}, duration: 3.0)", 1 + level.div_ceil(5))
        },
        "Holy" => format!("Fortify(defense_pct: {:.1}, duration: 3.0)", 8.0 + power * 0.8),
        _ if contains_any(&identity, &["warlord", "veteran", "berserk"]) => {
            format!("Berserk(attack_pct: {:.1}, duration: 3.0)", 5.0 + power * 0.65)
        },
        _ if contains_any(&identity, &["cloth", "runed", "rune", "arcane"]) => {
            format!("Clearcasting(reduction_pct: {:.1}, duration: 3.0)", 8.0 + power * 0.5)
        },
        _ if contains_any(&identity, &["mail", "chain", "plate", "sentinel", "vanguard"]) => {
            format!("Fortify(defense_pct: {:.1}, duration: 3.0)", 7.0 + power * 0.7)
        },
        _ if slot == "Boots" || contains_any(&identity, &["leather", "swift", "speed"]) => {
            format!("Haste(initiative_pct: {:.1}, duration: 3.0)", 6.0 + power * 0.7)
        },
        _ => format!("Thorns(damage_reflected_pct: {:.1}, duration: 3.0)", 4.0 + power * 0.8),
    };
    vec![effect]
}

/// Adds a small deterministic specialty when two wearables would otherwise share a build.
fn make_wearable_profile_unique(
    kind: &str,
    slot: &str,
    modifiers: &mut Vec<String>,
    effects: &[String],
    profiles: &mut HashSet<String>,
) {
    let mut collision = 0u32;
    loop {
        let profile = format!("{kind}|{slot}|{}|{}", modifiers.join(","), effects.join(","));
        if profiles.insert(profile) {
            break;
        }
        collision += 1;
        modifiers.push(match collision % 4 {
            1 => format!("MaxHealthModifier({collision})"),
            2 => format!("MaxManaModifier({collision})"),
            3 => format!("InitiativeModifier({collision})"),
            _ => format!("AttackModifier({collision})"),
        });
    }
}

/// Derives consumable potency from size labels, sequence, and special rarity terms.
fn consumable_level(filename: &str) -> u32 {
    let lower = filename.to_lowercase();
    let sequence = get_last_number(filename).unwrap_or(1.0).max(1.0) as u32;
    let mut level = 1 + sequence.saturating_sub(1) / 3;
    if contains_any(&lower, &["tea", "water", "little", "minor"]) {
        level = level.min(3);
    } else if contains_any(&lower, &["middle", "medium"]) {
        level = level.clamp(7, 11);
    } else if contains_any(&lower, &["big", "greater"]) {
        level = level.clamp(12, 16);
    } else if contains_any(&lower, &["huge", "grand"]) {
        level = level.max(17);
    }
    if contains_any(&lower, &["deadly", "immortal", "invisibility", "plague", "spiritual"]) {
        level = level.max(14);
    }
    if lower.contains("potion_king") {
        level = level.max(12);
    } else if contains_any(&lower, &["potion_shadow", "potion_spider"]) {
        level = level.max(8);
    } else if lower.contains("potion_green") {
        level = level.max(4);
    }
    level.clamp(1, 20)
}

/// Derives a readable consumable name from its source filename and pictured contents.
fn consumable_base_name(filename: &str, variant: usize) -> String {
    let stem = Path::new(filename).file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
    let stem_lower = stem.to_lowercase();
    let exact_name = match stem_lower.as_str() {
        "alchemy_53_huge_flask2" => Some("Grand Silver Elixir"),
        "alchemy_53_huge_flask3" => Some("Grand Gilded Elixir"),
        "alchemy_53_huge_flask4" => Some("Grand Emerald Elixir"),
        "potion_green" => Some("Verdant Tonic"),
        "potion_king" => Some("King's Elixir"),
        "potion_shadow" => Some("Shadow Tonic"),
        "potion_spider" => Some("Spider Venom Coating"),
        "potion_energy" => Some("Energy Tonic"),
        "questbottle" => Some("Questmaster's Remedy"),
        "quest_139_potions" => Some("Venom Vial Set"),
        "quest_140_potions" => Some("Prismatic Elixir Set"),
        "quest_76" => Some("Amber Serum"),
        "quest_77" => Some("Aqua Serum"),
        "quest_78" => Some("Umbral Serum"),
        "res_39_colbgreen" => Some("Verdant Restorative"),
        "res_40_colbred" => Some("Crimson Restorative"),
        "res_41_colbshadow" => Some("Umbral Restorative"),
        "res_42_ink" => Some("Umbral Draught"),
        "res_43_manapotion" => Some("Mana Potion"),
        "res_44_healthpotion" => Some("Healing Potion"),
        "res_46_medicines" => Some("Restorative Medicine"),
        "res_47_medicines" => Some("Restorative Medicine"),
        "res_48_medicines" => Some("Restorative Medicine"),
        "res_49_health" => Some("Crimson Restorative"),
        "res_51_stun" => Some("Clarity Tonic"),
        "res_97" => Some("Sealed Remedy"),
        "res_98" => Some("Bound Crimson Elixir"),
        "res_99" => Some("Bound Violet Elixir"),
        "res_100" => Some("Bound Teal Elixir"),
        "res_101" => Some("Ember Elixir"),
        "res_102" => Some("Rose Elixir"),
        "res_103_magicpotion" => Some("Arcane Elixir"),
        "res_117" => Some("Crimson Ring Flask"),
        "res_118" => Some("Verdant Ring Flask"),
        "res_119" => Some("Violet Ring Flask"),
        _ => None,
    };
    if let Some(name) = exact_name {
        return name.to_string();
    }

    let source = stem
        .split_once('_')
        .map(|(prefix, rest)| {
            if ["alchemy", "enchantment", "potion", "quest", "res", "tailoring"]
                .contains(&prefix.to_lowercase().as_str())
            {
                rest.trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches(['_', '-', ' '])
            } else {
                stem
            }
        })
        .unwrap_or(stem);
    let mut name = clean_name(source);
    if name.is_empty() {
        let fallback = ["Restorative Elixir", "Traveler's Tonic", "Amber Draught", "Silver Flask"];
        name = fallback[variant % fallback.len()].to_string();
    }
    let lower = filename.to_lowercase();
    if lower.contains("poison") {
        name = match name.as_str() {
            "Deadly Poison" => "Deadly Venom Coating".to_string(),
            "Black Poison" => "Black Venom Coating".to_string(),
            "Big Poison" => "Greater Venom Coating".to_string(),
            "Fastpoison" => "Swift Venom Coating".to_string(),
            "Poisonousherbs" => "Venomous Herb Coating".to_string(),
            _ if name == "Poison" => "Venom Coating".to_string(),
            _ => name,
        };
    } else if lower.contains("magicdust") {
        name = "Arcane Dust Tonic".to_string();
    } else if name == "Water" {
        name = "Springwater Tonic".to_string();
    } else if name == "Blood" {
        name = "Blood Tonic".to_string();
    } else if name == "Shadow" {
        name = "Shadow Tonic".to_string();
    } else if name == "Mercury" {
        name = "Quicksilver Tonic".to_string();
    } else if name == "Colb" {
        name = "Alchemist's Flask".to_string();
    }
    name
}

/// Assigns consumable effects that match the pictured mixture and its potency.
fn consumable_effects(filename: &str, level: u32) -> Vec<String> {
    let lower = filename.to_lowercase();
    if contains_any(&lower, &["potion_spider", "quest_139"]) {
        vec![format!("Empower(damage_pct: {:.1}, duration: 8.0)", 6.0 + level as f32)]
    } else if lower.contains("reactive") {
        vec![format!("Haste(initiative_pct: {:.1}, duration: 8.0)", 10.0 + level as f32)]
    } else if lower.contains("potion_king") {
        vec![
            format!("Heal(heal_pct: {})", 12 + level * 2),
            format!("Fortify(defense_pct: {:.1}, duration: 8.0)", 10.0 + level as f32),
        ]
    } else if lower.contains("res_51_stun") {
        vec![
            format!("Focus(crit_chance_pct: {:.1}, duration: 8.0)", 5.0 + level as f32 * 0.5),
            format!("Haste(initiative_pct: {:.1}, duration: 8.0)", 8.0 + level as f32),
        ]
    } else if lower.contains("quest_140") {
        vec![
            format!("Heal(heal_pct: {})", 12 + level * 2),
            format!("InstantMana(amount: {})", 15 + level * 8),
        ]
    } else if contains_any(&lower, &["quest_77", "res_99", "res_100", "res_119"]) {
        vec![
            format!("InstantMana(amount: {})", 15 + level * 8),
            format!("ManaFlow(amount: {}, duration: 5.0)", 1 + level.div_ceil(4)),
        ]
    } else if lower.contains("quest_78") {
        vec![format!(
            "Clearcasting(reduction_pct: {:.1}, duration: 8.0)",
            10.0 + level as f32 * 0.5
        )]
    } else if contains_any(&lower, &["res_98", "res_101"]) {
        vec![format!("Berserk(attack_pct: {:.1}, duration: 8.0)", 8.0 + level as f32)]
    } else if contains_any(&lower, &["quest_76", "res_97", "res_102", "res_117", "res_118"]) {
        vec![
            format!("Heal(heal_pct: {})", 15 + level * 2),
            format!("Regen(heal: {}, duration: 5.0)", 1 + level.div_ceil(4)),
        ]
    } else if lower.contains("holywater") {
        vec!["Purge".to_string(), format!("Heal(heal_pct: {})", 15 + level)]
    } else if lower.contains("plague") {
        vec![format!("Poison(damage: {}, duration: 5.0)", 1 + level.div_ceil(3))]
    } else if lower.contains("poison") {
        vec![format!("Empower(damage_pct: {:.1}, duration: 8.0)", 6.0 + level as f32)]
    } else if lower.contains("blood") {
        vec![
            format!("Berserk(attack_pct: {:.1}, duration: 8.0)", 8.0 + level as f32),
            format!("Lifesteal(percentage: {:.1}, duration: 8.0)", 3.0 + level as f32 * 0.4),
        ]
    } else if lower.contains("invisibility") {
        vec![
            format!("Haste(initiative_pct: {:.1}, duration: 8.0)", 12.0 + level as f32),
            format!("Focus(crit_chance_pct: {:.1}, duration: 8.0)", 5.0 + level as f32 * 0.5),
        ]
    } else if lower.contains("mercury") {
        vec![format!("Haste(initiative_pct: {:.1}, duration: 8.0)", 10.0 + level as f32)]
    } else if contains_any(&lower, &["stamina", "immortal"]) {
        vec![format!("Fortify(defense_pct: {:.1}, duration: 10.0)", 10.0 + level as f32)]
    } else if contains_any(
        &lower,
        &["mana", "magic", "energy", "blue", "spiritual", "ink", "magicdust"],
    ) {
        let mut effects = vec![format!("InstantMana(amount: {})", 15 + level * 8)];
        if level >= 7 {
            effects.push(format!("ManaFlow(amount: {}, duration: 5.0)", 1 + level.div_ceil(4)));
        }
        effects
    } else if contains_any(&lower, &["health", "heal", "green", "medicine", "tea"]) {
        let mut effects = vec![format!("Heal(heal_pct: {})", 15 + level * 2)];
        if level >= 7 {
            effects.push(format!("Regen(heal: {}, duration: 5.0)", 1 + level.div_ceil(4)));
        }
        effects
    } else if contains_any(&lower, &["shadow", "dark"]) {
        vec![format!(
            "Clearcasting(reduction_pct: {:.1}, duration: 8.0)",
            10.0 + level as f32 * 0.5
        )]
    } else {
        vec![format!("Heal(heal_pct: {})", 12 + level * 2)]
    }
}

/// Calculates archetype-adjusted monster stats from level and monster family.
fn monster_stats(name: &str, level: u32, kind: &str) -> (u32, u32, u32, u32, f32, i32) {
    let lower = name.to_lowercase();
    let (mut health, mut attack, mut defense, mut initiative, mut speed, regen): (
        f32,
        f32,
        f32,
        f32,
        f32,
        i32,
    ) = match kind {
        "Pet" => (
            34.0 + level as f32 * 8.0,
            4.0 + level as f32 * 1.7,
            4.0 + level as f32 * 1.3,
            5.0 + level as f32 * 1.5,
            1.0,
            1 + (level / 5) as i32,
        ),
        "Dragon" => (
            72.0 + level as f32 * 14.0,
            8.0 + level as f32 * 2.5,
            7.0 + level as f32 * 1.8,
            7.0 + level as f32 * 1.7,
            0.9,
            (level / 10) as i32,
        ),
        _ => (
            52.0 + level as f32 * 11.0,
            6.0 + level as f32 * 2.1,
            5.0 + level as f32 * 1.5,
            6.0 + level as f32 * 1.5,
            1.0,
            (level / 12) as i32,
        ),
    };

    if contains_any(
        &lower,
        &[
            "troll",
            "ogre",
            "stone golem",
            "tarrasque",
            "bear",
            "crocodile",
            "hydra",
            "grave warden",
            "bone colossus",
            "crimson minotaur",
            "abyssal behemoth",
            "badger",
            "boar",
        ],
    ) {
        health *= 1.18;
        defense *= 1.12;
        initiative *= 0.82;
        speed *= 0.9;
    } else if lower.contains("void reaver") {
        health *= 0.92;
        attack *= 1.18;
        defense *= 0.94;
        initiative *= 1.18;
        speed *= 1.10;
    } else if contains_any(
        &lower,
        &[
            "bat",
            "weasel",
            "puma",
            "tiger",
            "spider",
            "snake",
            "drow",
            "vulture",
            "bog imp",
            "storm harpy",
            "werewolf",
            "wererat",
            "fox",
            "raven",
            "lynx",
            "shadow panther",
        ],
    ) {
        health *= 0.90;
        defense *= 0.90;
        initiative *= 1.22;
        speed *= 1.12;
    } else if contains_any(
        &lower,
        &[
            "lich",
            "vampire",
            "mind flayer",
            "rakshasa",
            "aboleth",
            "medusa",
            "mire hag",
            "frostbound wraith",
        ],
    ) {
        health *= 0.94;
        attack *= 1.12;
        initiative *= 1.08;
    }

    (
        health.round().max(1.0) as u32,
        attack.round().max(1.0) as u32,
        defense.round().max(0.0) as u32,
        initiative.round().max(0.0) as u32,
        speed.max(0.5),
        regen,
    )
}

/// Returns the curated encounter level for a creature.
fn monster_creature_level(name: &str) -> u32 {
    match name.to_lowercase().as_str() {
        "goblin" | "skeleton" => 1,
        "formicid" | "kuo-toa" | "lizardfolk" | "gnoll" => 2,
        "drow" => 3,
        "bog imp" => 3,
        "ogre" => 4,
        "mire hag" => 5,
        "basilisk" | "fire troll" | "ice troll" | "mountain troll" => 6,
        "medusa" | "owlbear" | "griffin" | "manticore" => 7,
        "bone colossus" => 8,
        "grave warden" => 9,
        "storm harpy" => 9,
        "worg" => 4,
        "hydra" | "yuan-ti" => 10,
        "crimson minotaur" => 11,
        "mind flayer" | "rakshasa" => 12,
        "aboleth" => 13,
        "frostbound wraith" => 14,
        "empyrean" => 15,
        "void reaver" => 16,
        "wererat" => 8,
        "werebear" => 14,
        "werewolf" => 17,
        "lich" => 18,
        "vampire" => 19,
        "balor" => 19,
        "tarrasque" | "abyssal behemoth" => 20,
        _ => 5,
    }
}

/// Returns the curated progression level for a tameable pet.
fn monster_pet_level(name: &str) -> u32 {
    match name.to_lowercase().as_str() {
        "rat" | "bat" | "snake" | "spider" | "weasel" | "owl" | "vulture" | "lizard" | "fox"
        | "raven" => 1,
        "hyena" | "puma" | "eagle" | "crocodile" | "badger" => 2,
        "wolf" | "worg" | "bear" | "tiger" | "boar" | "lynx" => 3,
        "hell hound" | "griffin" | "owlbear" | "cerberus" | "shadow panther" => 5,
        "frost stag" => 6,
        "pegasus" | "unicorn" | "manticore" | "ember drake" => 8,
        _ => 4,
    }
}

/// Generate all inventory RON catalogs.
///
/// - `src_images`:    path to `assets-src/images`
/// - `out_inventory`: path to output directory (e.g. `assets/catalog`)
/// - `img_ext`:       image extension used in RON references (`"webp"` or `"png"`)
pub fn run(src_images: &str, out_inventory: &str, img_ext: &str) {
    fs::create_dir_all(out_inventory).unwrap();

    // ── 1. ABILITIES ─────────────────────────────────────────────────────────
    let abilities_dir = format!("{}/catalog/abilities", src_images);
    let mut abilities_files: Vec<(String, f64)> = list_png_files(&abilities_dir)
        .into_iter()
        .map(|f| {
            let score = get_image_score(&f);
            (f, score)
        })
        .collect();
    abilities_files.sort_by(|a, b| {
        a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal).then_with(|| a.0.cmp(&b.0))
    });

    let total_abs = abilities_files.len();
    let chunk_size_abs = total_abs as f64 / 20.0;
    let mut abilities_ron = String::from("[\n");
    let mut seen_abilities = HashSet::new();
    let mut represented_starting_kinds = HashSet::new();

    for (idx, (filename, _)) in abilities_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_size_abs) as u32 + 1;
        if level > 20 {
            level = 20;
        }

        let mut cleaned = clean_name(filename);
        let needs_generated_name = cleaned.is_empty();
        let textual_kind = classify_kind(
            &format!("{filename} {cleaned}"),
            &Path::new(&abilities_dir).join(filename),
            false,
        );
        let kind = if textual_kind != "Physical" || !needs_generated_name {
            textual_kind
        } else {
            visual_kind(&Path::new(&abilities_dir).join(filename)).unwrap_or("Physical")
        };
        if STARTING_ABILITY_KINDS.contains(&kind) && represented_starting_kinds.insert(kind) {
            level = 1;
        }
        let pool = match kind {
            "Fire" => FIRE_POOL,
            "Ice" => FROST_POOL,
            "Nature" => NATURE_POOL,
            "Holy" => HOLY_POOL,
            "Shadow" => SHADOW_POOL,
            _ => PHYSICAL_POOL,
        };
        if cleaned.is_empty() {
            cleaned = pool[idx % pool.len()].to_string();
        }
        if cleaned.len() > 25 {
            cleaned = pool[idx % pool.len()].to_string();
        }

        let mut name = cleaned.to_lowercase();
        let mut ctr = 1;
        while seen_abilities.contains(&capitalize_words(&name)) {
            let mod_idx = (idx + ctr as usize) % UNIQUE_MODIFIERS.len();
            name = format!("{} {}", cleaned.to_lowercase(), UNIQUE_MODIFIERS[mod_idx]);
            ctr += 1;
        }
        let name = capitalize_words(&name);
        seen_abilities.insert(name.clone());

        let lower = format!("{filename} {name}").to_lowercase();
        let is_aoe = [
            "wave",
            "rain",
            "blizzard",
            "storm",
            "aoe",
            "clones",
            "army",
            "pack",
            "summon",
            "pet",
            "companion",
            "healing wave",
            "aura",
        ]
        .iter()
        .any(|cue| lower.contains(cue));
        let (effects, on_self, max_duration) = ability_effects(kind, &name, filename, level);
        let mana_cost = 4 + level * 2 + u32::from(is_aoe) * 3 + (idx % 3) as u32;
        let cooldown = (7.5
            + level as f32 * 0.25
            + if is_aoe {
                2.0
            } else {
                0.0
            }
            + (idx % 3) as f32 * 0.5)
            .max(max_duration + 2.0);

        abilities_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/catalog/abilities/{img}\",\n        kind: {kind},\n        level: {level},\n        mana_cost: {mana_cost},\n        cooldown: {cooldown:.1},\n        on_self: {on_self},\n        is_aoe: {is_aoe},\n        effects: [{effects}],\n    ),\n",
            name = name,
            img = img_name(filename, img_ext),
            kind = kind,
            level = level,
            mana_cost = mana_cost,
            cooldown = cooldown,
            on_self = on_self,
            is_aoe = is_aoe,
            effects = effects.join(", "),
        ));
    }
    abilities_ron.push_str("]\n");
    File::create(format!("{out_inventory}/abilities.ron"))
        .unwrap()
        .write_all(abilities_ron.as_bytes())
        .unwrap();
    println!("Generated {} abilities in abilities.ron", total_abs);

    // ── 2. PERKS ─────────────────────────────────────────────────────────────
    let perks_dir = format!("{}/catalog/perks", src_images);
    let mut perks_files: Vec<(String, f64)> = list_png_files(&perks_dir)
        .into_iter()
        .map(|f| {
            let s = get_image_score(&f);
            (f, s)
        })
        .collect();
    perks_files.sort_by(|a, b| {
        a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal).then_with(|| a.0.cmp(&b.0))
    });

    let total_pks = perks_files.len();
    let chunk_size_pks = total_pks as f64 / 20.0;
    let mut perks_ron = String::from("[\n");
    let mut seen_perks = HashSet::new();

    for (idx, (filename, _)) in perks_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_size_pks) as u32 + 1;
        if level > 20 {
            level = 20;
        }

        let mut cleaned = clean_name(filename);
        let kind = classify_kind(
            &format!("{filename} {cleaned}"),
            &Path::new(&perks_dir).join(filename),
            true,
        );
        let pool = match kind {
            "Fire" => FIRE_POOL,
            "Ice" => FROST_POOL,
            "Nature" => NATURE_POOL,
            "Holy" => HOLY_POOL,
            "Shadow" => SHADOW_POOL,
            _ => PHYSICAL_POOL,
        };
        if cleaned.is_empty() {
            cleaned = format!("{} Passive", pool[idx % pool.len()]);
        }
        if cleaned.len() > 25 {
            cleaned = format!("{} Passive", pool[idx % pool.len()]);
        }

        let mut name = cleaned.to_lowercase();
        let mut ctr = 1;
        while seen_perks.contains(&capitalize_words(&name)) {
            name = format!(
                "{} {}",
                cleaned.to_lowercase(),
                UNIQUE_MODIFIERS[(idx + ctr) % UNIQUE_MODIFIERS.len()]
            );
            ctr += 1;
        }
        let name = capitalize_words(&name);
        seen_perks.insert(name.clone());
        let modifiers = perk_modifiers(kind, &name, filename, level);

        perks_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/catalog/perks/{img}\",\n        level: {level},\n        modifiers: [{mods}],\n    ),\n",
            name = name, img = img_name(filename, img_ext), level = level, mods = modifiers.join(", "),
        ));
    }
    perks_ron.push_str("]\n");
    File::create(format!("{out_inventory}/perks.ron"))
        .unwrap()
        .write_all(perks_ron.as_bytes())
        .unwrap();
    println!("Generated {} perks in perks.ron", total_pks);

    // ── 3. ARTIFACTS ─────────────────────────────────────────────────────────
    let artifacts_dir = format!("{}/catalog/artifacts", src_images);
    let mut artifacts_files = list_png_files(&artifacts_dir);
    artifacts_files.sort();

    let dyes_filename = "Tailoring_13_dyes.png";
    let dyes_available =
        Path::new(src_images).join("catalog/consumable").join(dyes_filename).is_file();
    let total_arts = artifacts_files.len() + usize::from(dyes_available);
    let mut artifacts_ron = String::from("[\n");
    let mut equipment_names = HashSet::new();

    for (idx, filename) in artifacts_files.iter().enumerate() {
        let (base_name, group) = artifact_name_parts(filename, idx);
        let level = artifact_level(filename, &base_name, &group);
        let kind = classify_artifact_kind(&base_name);
        let name = unique_artifact_name(&base_name, level, idx, kind, &group, &mut equipment_names);
        let img = img_name(filename, img_ext);
        let price = artifact_price(&base_name, &group, level);

        artifacts_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/catalog/artifacts/{img}\",\n        kind: {kind},\n        level: {level},\n        price: {price},\n    ),\n",
            name = name, img = img, kind = kind, level = level, price = price
        ));
    }
    if dyes_available {
        let name = "Dye Pigments".to_string();
        equipment_names.insert(name.clone());
        artifacts_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/catalog/consumable/{img}\",\n        kind: Physical,\n        level: 4,\n        price: 30,\n    ),\n",
            img = img_name(dyes_filename, img_ext),
        ));
    }
    artifacts_ron.push_str("]\n");
    File::create(format!("{out_inventory}/artifacts.ron"))
        .unwrap()
        .write_all(artifacts_ron.as_bytes())
        .unwrap();
    println!("Generated {} artifacts in artifacts.ron", total_arts);

    // ── 4. WEAPONS ───────────────────────────────────────────────────────────
    let weapons_dir = format!("{}/catalog/equipment/weapon", src_images);
    let mut weapons_files: Vec<(String, f64)> = list_png_files(&weapons_dir)
        .into_iter()
        .filter(|f| {
            let l = f.to_lowercase();
            !l.contains("arrow")
                && !l.contains("quiver")
                && !l.contains("bullet")
                && !l.contains("bolt")
        })
        .map(|f| {
            let s = get_image_score(&f);
            (f, s)
        })
        .collect();
    weapons_files.sort_by(|a, b| {
        a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal).then_with(|| a.0.cmp(&b.0))
    });

    let total_wps = weapons_files.len();
    let chunk_wps = total_wps as f64 / 20.0;
    let mut weapons_ron = String::from("[\n");
    let mut weapon_profiles = HashSet::new();
    let mut weapon_prices = HashSet::new();

    for (idx, (filename, _)) in weapons_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_wps) as u32 + 1;
        if level > 20 {
            level = 20;
        }
        let lower = filename.to_lowercase();

        let hand =
            if ["bow", "staff", "two", "2h", "great", "spear", "halberd", "scythe", "claymore"]
                .iter()
                .any(|x| lower.contains(x))
            {
                "TwoHand"
            } else {
                "OneHand"
            };
        let category = if lower.contains("shield") {
            "Shield"
        } else if lower.contains("book") || lower.contains("scroll") || lower.contains("tome") {
            "Book"
        } else if lower.contains("wand") || lower.contains("staff") || lower.contains("scepter") {
            "Magical"
        } else if lower.contains("bow") || lower.contains("crossbow") || lower.contains("sling") {
            "Range"
        } else if contains_any(
            &lower,
            &["dagger", "rapier", "katar", "brassknuckle", "fist", "whip"],
        ) {
            "Finesse"
        } else {
            "Melee"
        };

        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = match category {
                "Shield" => "Shield",
                "Book" => "Spellbook",
                "Magical" => "Arcane Focus",
                "Range" => "Hunting Bow",
                "Finesse" => "Dueling Blade",
                _ => "Steel Weapon",
            }
            .to_string();
        }
        let kind = classify_kind(
            &format!("{filename} {cleaned}"),
            &Path::new(&weapons_dir).join(filename),
            false,
        );
        let name = unique_name(&cleaned, level, idx, kind, &mut equipment_names);

        let hm = if hand == "TwoHand" {
            1.75f32
        } else {
            1.0
        };
        let mut attack = 0u32;
        let mut speed = 0.0f32;
        let mut crit = 0.0f32;
        let mut modifiers = Vec::new();
        let stat_bonus = (level.div_ceil(4) as f32 * hm).round().max(1.0) as i32;

        match category {
            "Shield" => {
                modifiers.push(format!("DefenseModifier({})", 2 + level.div_ceil(2)));
                modifiers.push(format!("AttributeModifier(Constitution, {stat_bonus})"));
            },
            "Book" => {
                modifiers.push(format!("AttributeModifier(Wisdom, {stat_bonus})"));
                modifiers.push(format!("ManaRegen({})", 1 + level / 8));
            },
            "Magical" => {
                attack = ((2.0 + level as f32) * hm).round() as u32;
                speed = 0.95;
                modifiers.push(format!("AttributeModifier(Intelligence, {stat_bonus})"));
            },
            "Finesse" => {
                attack = ((2.0 + level as f32 * 1.15) * hm).round() as u32;
                speed = 1.35;
                crit = 0.08 + level as f32 * 0.005;
                modifiers.push(format!("AttributeModifier(Dexterity, {stat_bonus})"));
            },
            "Range" => {
                attack = ((3.0 + level as f32 * 1.4) * hm).round() as u32;
                speed = 0.80;
                crit = 0.04 + level as f32 * 0.003;
                modifiers.push(format!("AttributeModifier(Dexterity, {stat_bonus})"));
            },
            _ => {
                attack = ((3.0 + level as f32 * 1.3) * hm).round() as u32;
                speed = 1.0;
                crit = 0.03 + level as f32 * 0.003;
                modifiers.push(format!("AttributeModifier(Strength, {stat_bonus})"));
            },
        }
        if hand == "TwoHand" && speed > 0.0 {
            speed *= 0.85;
        }
        if kind != "Physical" {
            modifiers.push(format!("KindPowerMultiplier({kind}, {:.1})", 3.0 + level as f32 * 0.6));
        }
        modifiers.push(weapon_identity_modifier(filename, &name, kind, category, level, idx));
        let effects = weapon_effects(kind, category, level, idx);
        let fixed_profile = format!("{kind}|{category}|{hand}|{attack}|{speed:.2}|{crit:.2}");
        make_weapon_profile_unique(&fixed_profile, &mut modifiers, &effects, &mut weapon_profiles);
        let mut price = 20.0 + (level * level * 18) as f32;
        if hand == "TwoHand" {
            price *= 1.55;
        }
        price *= match category {
            "Shield" => 0.90,
            "Magical" => 1.10,
            "Book" => 1.05,
            _ => 1.0,
        };
        if !effects.is_empty() {
            price *= 1.10;
        }
        let mut price = price.round().max(1.0) as u32;
        while !weapon_prices.insert(format!("{kind}|{category}|{hand}|{level}|{price}")) {
            price += 1;
        }

        weapons_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/catalog/equipment/weapon/{img}\",\n        kind: {kind},\n        category: {category},\n        hand: {hand},\n        level: {level},\n        price: {price},\n        attack: {attack},\n        attack_speed: {speed:.2},\n        crit_chance: {crit:.2},\n        modifiers: [{mods}],\n        effects: [{effects}],\n    ),\n",
            name = name, img = img_name(filename, img_ext), kind = kind, category = category, hand = hand,
            level = level, price = price, attack = attack, speed = speed, crit = crit,
            mods = modifiers.join(", "), effects = effects.join(", "),
        ));
    }
    weapons_ron.push_str("]\n");
    File::create(format!("{out_inventory}/weapons.ron"))
        .unwrap()
        .write_all(weapons_ron.as_bytes())
        .unwrap();
    println!("Generated {} weapons in weapons.ron", total_wps);

    // ── 4. WEARABLES ─────────────────────────────────────────────────────────
    let armor_folders = [
        ("accessory", "Accessory"),
        ("armor", "Chestplate"),
        ("boots", "Boots"),
        ("gloves", "Gloves"),
        ("helmet", "Helmet"),
    ];
    let mut armor_files: Vec<(String, f64, String, String)> = Vec::new();
    for (folder, slot) in &armor_folders {
        for f in list_png_files(&format!("{}/catalog/equipment/{}", src_images, folder)) {
            let s = get_image_score(&f);
            armor_files.push((f, s, folder.to_string(), slot.to_string()));
        }
    }
    armor_files.sort_by(|a, b| {
        a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal).then_with(|| a.0.cmp(&b.0))
    });

    let total_arm = armor_files.len();
    let chunk_arm = total_arm as f64 / 20.0;
    let mut armor_ron = String::from("[\n");
    let mut wearable_profiles = HashSet::new();
    let mut wearable_prices = HashSet::new();

    for (idx, (filename, _, folder, slot)) in armor_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_arm) as u32 + 1;
        if level > 20 {
            level = 20;
        }
        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = match slot.as_str() {
                "Helmet" => "Helmet",
                "Chestplate" => "Chestplate",
                "Gloves" => "Gauntlets",
                "Boots" => "Boots",
                _ => "Talisman",
            }
            .to_string();
        }
        let kind = classify_kind(
            &format!("{filename} {cleaned}"),
            &Path::new(src_images).join("catalog").join("equipment").join(folder).join(filename),
            false,
        );
        let name = unique_name(&cleaned, level, idx, kind, &mut equipment_names);

        let mut modifiers = Vec::new();
        let stat_bonus = level.div_ceil(4) as i32;

        match slot.as_str() {
            "Chestplate" => {
                modifiers.push(format!("DefenseModifier({})", 2 + level));
                let attr = match kind {
                    "Nature" => "Dexterity",
                    "Holy" | "Shadow" | "Fire" | "Ice" => "Intelligence",
                    _ => "Constitution",
                };
                modifiers.push(format!("AttributeModifier({attr}, {stat_bonus})"));
            },
            "Helmet" => {
                modifiers.push(format!("DefenseModifier({})", 1 + level * 3 / 4));
                let attr = if kind == "Physical" {
                    "Constitution"
                } else {
                    "Wisdom"
                };
                modifiers.push(format!("AttributeModifier({attr}, {stat_bonus})"));
            },
            "Gloves" => {
                modifiers.push(format!("DefenseModifier({})", 1 + level / 2));
                let attr = if kind == "Physical" {
                    "Strength"
                } else {
                    "Intelligence"
                };
                modifiers.push(format!("AttributeModifier({attr}, {stat_bonus})"));
            },
            "Boots" => {
                modifiers.push(format!("DefenseModifier({})", 1 + level / 2));
                modifiers.push(format!("InitiativeModifier({})", 1 + level / 4));
                modifiers.push(format!("AttributeModifier(Dexterity, {stat_bonus})"));
            },
            "Accessory" => {
                if idx % 2 == 0 {
                    modifiers.push(format!("MaxHealthModifier({})", (level * 5) as i32));
                } else {
                    modifiers.push(format!("MaxManaModifier({})", (level * 3) as i32));
                }
            },
            _ => {
                modifiers.push(format!("HealthRegen({})", 1 + level / 6));
            },
        }
        if kind != "Physical" && level >= 5 {
            modifiers
                .push(format!("KindResistanceMultiplier({kind}, {:.1})", 3.0 + level as f32 * 0.5));
        }
        modifiers.push(wearable_identity_modifier(filename, &name, kind, level, idx));
        let effects = wearable_effects(kind, filename, &name, slot, level, idx);
        make_wearable_profile_unique(kind, slot, &mut modifiers, &effects, &mut wearable_profiles);
        let slot_multiplier = match slot.as_str() {
            "Chestplate" => 1.30,
            "Helmet" => 1.10,
            "Accessory" => 1.05,
            _ => 0.90,
        };
        let magic_multiplier = if kind == "Physical" {
            1.0
        } else {
            1.08
        };
        let effect_multiplier = if effects.is_empty() {
            1.0
        } else {
            1.10
        };
        let mut price = ((15 + level * level * 16) as f32
            * slot_multiplier
            * magic_multiplier
            * effect_multiplier)
            .round()
            .max(1.0) as u32;
        while !wearable_prices.insert(format!("{kind}|{slot}|{level}|{price}")) {
            price += 1;
        }

        armor_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/catalog/equipment/{folder}/{img}\",\n        kind: {kind},\n        price: {price},\n        slot: {slot},\n        modifiers: [{mods}],\n        effects: [{effects}],\n        level: {level},\n    ),\n",
            name = name, folder = folder, img = img_name(filename, img_ext), kind = kind,
            price = price, slot = slot, mods = modifiers.join(", "), effects = effects.join(", "), level = level,
        ));
    }
    armor_ron.push_str("]\n");
    File::create(format!("{out_inventory}/wearables.ron"))
        .unwrap()
        .write_all(armor_ron.as_bytes())
        .unwrap();
    println!("Generated {} wearables in wearables.ron", total_arm);

    // ── 5. CONSUMABLES ───────────────────────────────────────────────────────
    let consumables_dir = format!("{}/catalog/consumable", src_images);
    let mut consumables_files = list_png_files(&consumables_dir);
    consumables_files.retain(|filename| !filename.to_lowercase().contains("dyes"));
    consumables_files.sort();

    let total_cons = consumables_files.len();
    let mut consumables_ron = String::from("[\n");
    let mut consumable_profiles = HashSet::new();
    let mut consumable_prices = HashSet::new();

    for (idx, filename) in consumables_files.iter().enumerate() {
        let level = consumable_level(filename);
        let base_name = consumable_base_name(filename, idx);
        let name = unique_consumable_name(&base_name, level, idx, &mut equipment_names);
        let mut price = 4 + level * 3 + level * level * 3;
        while !consumable_prices.insert(format!("{level}|{price}")) {
            price += 1;
        }
        let mut effects = consumable_effects(filename, level);
        let mut collision = 0u32;
        while !consumable_profiles.insert(effects.join(",")) {
            collision += 1;
            effects.push(format!(
                "Focus(crit_chance_pct: {:.1}, duration: 5.0)",
                1.0 + collision as f32
            ));
        }

        consumables_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/catalog/consumable/{img}\",\n        level: {level},\n        price: {price},\n        effects: [{effects}],\n        craft: [],\n    ),\n",
            name = name, img = img_name(filename, img_ext), level = level, price = price, effects = effects.join(", "),
        ));
    }
    consumables_ron.push_str("]\n");
    File::create(format!("{out_inventory}/consumables.ron"))
        .unwrap()
        .write_all(consumables_ron.as_bytes())
        .unwrap();
    println!("Generated {} consumables in consumables.ron", total_cons);

    // ── 6. MONSTERS ──────────────────────────────────────────────────────────
    let creatures_dir = format!("{}/monsters/creatures", src_images);
    let pets_dir = format!("{}/monsters/pets", src_images);

    let mut creatures_files = list_image_files(&creatures_dir)
        .into_iter()
        .map(|filename| {
            let name = capitalize_words(&clean_name(&filename));
            let level = monster_creature_level(&name);
            (filename, level)
        })
        .collect::<Vec<_>>();
    creatures_files.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    let mut pets_files = list_image_files(&pets_dir)
        .into_iter()
        .map(|filename| {
            let name = capitalize_words(&clean_name(&filename));
            let level = monster_pet_level(&name);
            (filename, level)
        })
        .collect::<Vec<_>>();
    pets_files.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    let mut monsters_ron = String::from("[\n");

    // Creatures
    for (filename, level) in creatures_files.iter() {
        let name = capitalize_words(&clean_name(filename));
        let img = img_name(filename, img_ext);
        let (max_hp, attack, defense, initiative, attack_speed, regen) =
            monster_stats(&name, *level, "Creature");
        let effects = monster_effects(&name, *level);
        let archetype = monster_archetype(&name, "Creature");

        monsters_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/monsters/creatures/{img}\",\n        level: {level},\n        kind: Creature,\n        archetype: {archetype},\n        health: {max_hp},\n        max_health: {max_hp},\n        attack: {attack},\n        defense: {defense},\n        initiative: {initiative},\n        attack_speed: {attack_speed:.2},\n        health_regen: {regen},\n        modifiers: [],\n        effects: [{effs}],\n    ),\n",
            name = name,
            img = img,
            level = *level,
            archetype = archetype,
            max_hp = max_hp,
            attack = attack,
            defense = defense,
            initiative = initiative,
            attack_speed = attack_speed,
            regen = regen,
            effs = effects.join(", "),
        ));
    }

    // Pets
    for (filename, level) in pets_files.iter() {
        let name = capitalize_words(&clean_name(filename));
        let img = img_name(filename, img_ext);
        let (max_hp, attack, defense, initiative, attack_speed, regen) =
            monster_stats(&name, *level, "Pet");
        let effects = monster_effects(&name, *level);
        let archetype = monster_archetype(&name, "Pet");

        monsters_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/monsters/pets/{img}\",\n        level: {level},\n        kind: Pet,\n        archetype: {archetype},\n        health: {max_hp},\n        max_health: {max_hp},\n        attack: {attack},\n        defense: {defense},\n        initiative: {initiative},\n        attack_speed: {attack_speed:.2},\n        health_regen: {regen},\n        modifiers: [],\n        effects: [{effs}],\n    ),\n",
            name = name,
            img = img,
            level = *level,
            archetype = archetype,
            max_hp = max_hp,
            attack = attack,
            defense = defense,
            initiative = initiative,
            attack_speed = attack_speed,
            regen = regen,
            effs = effects.join(", "),
        ));
    }

    // Dragons
    let dragon_colors = [
        ("Black", [2, 8, 14]),
        ("Green", [3, 9, 15]),
        ("Blue", [4, 10, 16]),
        ("Silver", [5, 11, 17]),
        ("Red", [6, 12, 18]),
        ("Gold", [7, 13, 20]),
    ];
    let dragon_ages =
        [("Dragon Hatchling", "hatchling"), ("Dragon", "adult"), ("Elder Wyrm", "wyrm")];

    for (color, levels) in &dragon_colors {
        for (age_idx, (age_display, file_suffix)) in dragon_ages.iter().enumerate() {
            let level = levels[age_idx];
            let name = format!("{} {}", color, age_display);
            let img = format!("{}_{}.{}", color.to_lowercase(), file_suffix, img_ext);
            let (max_hp, attack, defense, initiative, attack_speed, regen) =
                monster_stats(&name, level, "Dragon");
            let effects = monster_effects(&name, level);
            let archetype = monster_archetype(&name, "Dragon");

            monsters_ron.push_str(&format!(
                "    (\n        name: \"{name}\",\n        image: \"images/monsters/dragons/{img}\",\n        level: {level},\n        kind: Dragon,\n        archetype: {archetype},\n        health: {max_hp},\n        max_health: {max_hp},\n        attack: {attack},\n        defense: {defense},\n        initiative: {initiative},\n        attack_speed: {attack_speed:.2},\n        health_regen: {regen},\n        modifiers: [],\n        effects: [{effs}],\n    ),\n",
                name = name,
                img = img,
                level = level,
                archetype = archetype,
                max_hp = max_hp,
                attack = attack,
                defense = defense,
                initiative = initiative,
                attack_speed = attack_speed,
                regen = regen,
                effs = effects.join(", "),
            ));
        }
    }

    monsters_ron.push_str("]\n");
    File::create(format!("{out_inventory}/monsters.ron"))
        .unwrap()
        .write_all(monsters_ron.as_bytes())
        .unwrap();
    println!("Generated monsters in monsters.ron");
}

#[allow(dead_code)]
/// Runs the generate-catalogs entry point.
fn main() {
    #[cfg(feature = "process-assets")]
    let img_ext = "webp";
    #[cfg(not(feature = "process-assets"))]
    let img_ext = "png";

    run("assets-src/images", "assets/catalog", img_ext);
}

#[cfg(test)]
mod tests {
    use super::monster_archetype;

    #[test]
    /// Verifies tactical archetypes follow stable semantic monster families.
    fn monster_archetypes_are_semantically_stable() {
        assert_eq!(monster_archetype("Lich", "Creature"), "Necromancer");
        assert_eq!(monster_archetype("Stone Golem", "Creature"), "Knight");
        assert_eq!(monster_archetype("Vampire", "Creature"), "Leech");
        assert_eq!(monster_archetype("Red Dragon", "Dragon"), "Mage");
        assert_eq!(monster_archetype("Wolf", "Pet"), "Beast");
    }
}
