/// Catalog generation logic: reads PNG image lists from assets-src/images/catalog/
/// and writes inventory RON files to assets/inventory/.
///
/// This file is used in two ways:
///   1. As the `generate_catalogs` binary  (`cargo run --bin generate_catalogs`)
///   2. Included via `include!()` in both `src/bin/build.rs` and the root `build.rs`
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const LEVEL_ADJECTIVES: &[&str] = &[
    "Rusty",
    "Worn",
    "Apprentice",
    "Novice",
    "Simple",
    "Sturdy",
    "Reliable",
    "Skilled",
    "Refined",
    "Veteran",
    "Gilded",
    "Heavy",
    "Elite",
    "Master",
    "Championship",
    "Vanguard",
    "Mythic",
    "Legendary",
    "Ancient",
    "Divine",
];

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

    words.join(" ")
}

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

fn img_name(filename: &str, img_ext: &str) -> String {
    format!("{}.{}", Path::new(filename).file_stem().unwrap().to_str().unwrap(), img_ext)
}

fn classify_artifact_kind(name: &str) -> &'static str {
    let name_lower = name.to_lowercase();
    if name_lower.contains("frost")
        || name_lower.contains("ice")
        || name_lower.contains("raindrop")
        || name_lower.contains("snow")
        || name_lower.contains("whitebear")
        || name_lower.contains("cold")
        || name_lower.contains("water")
    {
        "Ice"
    } else if name_lower.contains("fire")
        || name_lower.contains("fiercloth")
        || name_lower.contains("torch")
        || name_lower.contains("dragon")
        || name_lower.contains("sunflower")
        || name_lower.contains("coal")
        || name_lower.contains("barbecue")
    {
        "Fire"
    } else if name_lower.contains("shadow")
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
        || name_lower.contains("spider")
    {
        "Shadow"
    } else if name_lower.contains("holy")
        || name_lower.contains("cross")
        || name_lower.contains("scroll")
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
        || name_lower.contains("crystal")
        || name_lower.contains("ore")
        || name_lower.contains("ingot")
        || name_lower.contains("gold")
        || name_lower.contains("silver")
        || name_lower.contains("copper")
        || name_lower.contains("runestone")
        || name_lower.contains("stoune")
    {
        "Nature"
    } else {
        "Physical"
    }
}

fn deterministic_shuffle<T>(items: &mut [T], mut seed: u64) {
    for i in (1..items.len()).rev() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (seed % (i as u64 + 1)) as usize;
        items.swap(i, j);
    }
}

/// Generate all inventory RON catalogs.
/// - `src_images`:    path to `assets-src/images`
/// - `out_inventory`: path to output directory (e.g. `assets/inventory`)
/// - `img_ext`:       image extension used in RON references (`"ktx2"` or `"png"`)
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
    abilities_files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    deterministic_shuffle(&mut abilities_files, 42);

    let total_abs = abilities_files.len();
    let chunk_size_abs = total_abs as f64 / 20.0;
    let mut abilities_ron = String::from("[\n");
    let mut seen_abilities = Vec::new();

    for (idx, (filename, _)) in abilities_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_size_abs) as u32 + 1;
        if level > 20 {
            level = 20;
        }

        let lower = filename.to_lowercase();
        let mut kind = "Physical";
        let mut pool = PHYSICAL_POOL;

        if [
            "fire",
            "pyro",
            "flame",
            "infernal",
            "burn",
            "cinder",
            "combustion",
            "lava",
            "blast",
            "sun",
            "phoenix",
            "red",
            "heat",
            "ash",
            "meteor",
            "scorch",
            "singe",
            "magma",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Fire";
            pool = FIRE_POOL;
        } else if [
            "frost", "ice", "chill", "cold", "blizzard", "glacial", "tomb", "shackle", "freeze",
            "crystal", "snow", "hail", "winter", "blue", "shiver",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Ice";
            pool = FROST_POOL;
        } else if [
            "holy", "smite", "divine", "radiance", "lay", "judgment", "sacred", "bastion", "light",
            "heal", "shield", "bless", "angel", "glory", "pray", "aura", "cure", "priest",
            "cleric",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Holy";
            pool = HOLY_POOL;
        } else if [
            "shadow",
            "dark",
            "curse",
            "vampiric",
            "agony",
            "soul",
            "covenant",
            "withering",
            "drain",
            "death",
            "devil",
            "demonic",
            "unholy",
            "evil",
            "hex",
            "blackwater",
            "plague",
            "fear",
            "terror",
            "ghoul",
            "spirit",
            "chain",
            "shackle",
            "doom",
            "necrom",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Shadow";
            pool = SHADOW_POOL;
        } else if [
            "nature", "bramble", "wild", "thorn", "bloom", "oak", "earth", "growth", "root",
            "leaf", "spore", "ivy", "forest", "grove", "poison", "venom", "toxic",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Nature";
            pool = NATURE_POOL;
        }

        let mut cleaned = clean_name(filename);
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
        seen_abilities.push(name.clone());

        let mana_cost = 5 + level * 2 + (idx % 3) as u32;
        let cooldown = (10.0 - level as f32 * 0.3 + (idx % 5) as f32 * 0.2).max(1.0);
        let is_aoe = level.is_multiple_of(4)
            || ["wave", "rain", "blizzard", "storm", "aoe", "clones", "explode"]
                .iter()
                .any(|x| lower.contains(x));

        let mut effects = Vec::new();
        let mut on_self = false;

        match kind {
            "Fire" => {
                effects
                    .push(format!("Burn(damage: {}, duration: 4.0)", level * 2 + (idx % 3) as u32));
            },
            "Ice" => {
                effects.push(format!(
                    "Freeze(attack_speed_pct: {:.1}, duration: 3.0)",
                    -10.0 - (level as f32 * 1.5)
                ));
            },
            "Nature" => {
                if idx % 2 == 0 {
                    effects.push(format!(
                        "Poison(damage: {}, duration: 5.0)",
                        level + (idx % 3) as u32
                    ));
                } else {
                    effects
                        .push(format!("Immobilize(duration: {:.1})", 2.0 + (idx % 3) as f32 * 0.5));
                }
            },
            "Holy" => {
                on_self = true;
                if idx % 2 == 0 {
                    effects.push(format!(
                        "Regen(heal: {}, duration: 5.0)",
                        level * 2 + (idx % 4) as u32
                    ));
                } else {
                    effects.push("Purge".to_string());
                }
            },
            "Shadow" => {
                if idx % 2 == 0 {
                    effects.push(format!(
                        "Vulnerability(damage_pct: {:.1}, duration: 5.0)",
                        5.0 + level as f32
                    ));
                } else {
                    effects.push(format!(
                        "Paranoia(initiative_pct: {:.1}, duration: 5.0)",
                        -5.0 - level as f32
                    ));
                }
            },
            _ => {
                if idx % 3 == 0 {
                    effects.push(format!("Bleed(damage_pct: {:.1})", 10.0 + level as f32 * 5.0));
                } else if idx % 3 == 1 {
                    effects.push(format!(
                        "Cleave(damage_pct: {:.1}, duration: 4.0)",
                        20.0 + level as f32 * 2.0
                    ));
                } else {
                    effects.push(format!("Pierce(damage: {})", level * 5));
                }
            },
        }

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
    perks_files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let total_pks = perks_files.len();
    let chunk_size_pks = total_pks as f64 / 20.0;
    let mut perks_ron = String::from("[\n");
    let mut seen_perks = Vec::new();

    for (idx, (filename, _)) in perks_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_size_pks) as u32 + 1;
        if level > 20 {
            level = 20;
        }

        let lower = filename.to_lowercase();
        let mut pool = PHYSICAL_POOL;

        if [
            "fire",
            "pyro",
            "flame",
            "infernal",
            "burn",
            "cinder",
            "combustion",
            "lava",
            "blast",
            "sun",
            "phoenix",
            "red",
            "heat",
            "ash",
            "meteor",
            "scorch",
            "singe",
            "magma",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            pool = FIRE_POOL;
        } else if [
            "frost", "ice", "chill", "cold", "blizzard", "glacial", "tomb", "shackle", "freeze",
            "crystal", "snow", "hail", "winter", "blue", "shiver",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            pool = FROST_POOL;
        } else if [
            "holy", "smite", "divine", "radiance", "lay", "judgment", "sacred", "bastion", "light",
            "heal", "shield", "bless", "angel", "glory", "pray", "aura", "cure", "priest",
            "cleric",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            pool = HOLY_POOL;
        } else if [
            "shadow",
            "dark",
            "curse",
            "vampiric",
            "agony",
            "soul",
            "covenant",
            "withering",
            "drain",
            "death",
            "devil",
            "demonic",
            "unholy",
            "evil",
            "hex",
            "blackwater",
            "plague",
            "fear",
            "terror",
            "ghoul",
            "spirit",
            "chain",
            "shackle",
            "doom",
            "necrom",
            "void",
            "abyss",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            pool = SHADOW_POOL;
        } else if [
            "nature", "bramble", "wild", "thorn", "bloom", "oak", "earth", "growth", "root",
            "leaf", "spore", "ivy", "forest", "grove", "poison", "venom", "toxic",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            pool = NATURE_POOL;
        }

        let mut cleaned = clean_name(filename);
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
        seen_perks.push(name.clone());

        let mut modifiers = Vec::new();
        let attrs = ["Strength", "Dexterity", "Constitution", "Intelligence", "Wisdom", "Charisma"];

        if level < 5 {
            if idx % 3 == 0 {
                let pos = attrs[idx % attrs.len()];
                let neg = attrs[(idx + 1 + (idx / attrs.len()) % (attrs.len() - 1)) % attrs.len()];
                modifiers.push(format!("AttributeModifier({}, {})", pos, level as i32 + 1));
                modifiers.push(format!("AttributeModifier({}, {})", neg, -(level as i32)));
            } else if idx % 3 == 1 {
                modifiers.push(match (idx + level as usize) % 5 {
                    0 => format!("AttackModifier({})", level as i32),
                    1 => format!("DefenseModifier({})", level as i32),
                    2 => format!("InitiativeModifier({})", level as i32),
                    3 => format!("MaxHealthModifier({})", (level * 10) as i32),
                    _ => format!("MaxManaModifier({})", (level * 5) as i32),
                });
            } else {
                modifiers.push(format!(
                    "AttributeModifier({}, {})",
                    attrs[idx % attrs.len()],
                    level as i32
                ));
            }
        } else {
            modifiers.push(format!(
                "AttributeModifier({}, {})",
                attrs[idx % attrs.len()],
                level as i32
            ));
            modifiers.push(match (idx + level as usize) % 6 {
                0 => format!("MaxHealthModifier({})", (level * 10) as i32),
                1 => format!("MaxManaModifier({})", (level * 5) as i32),
                2 => format!("AttackModifier({})", (level as i32 + 1) / 2),
                3 => format!("DefenseModifier({})", (level as i32 + 1) / 2),
                4 => format!("InitiativeModifier({})", (level as i32 + 2) / 3),
                _ => format!("LifeSteal({:.1})", level as f32),
            });
            if level >= 13 {
                modifiers.push(match (idx + 1) % 4 {
                    0 => format!("HealthRegen({})", (level / 4) as i32 + 1),
                    1 => format!("ManaRegen({})", (level / 5) as i32 + 1),
                    2 => format!("HealingMultiplier({:.1})", level as f32 * 1.5),
                    _ => format!("AttributeModifier(Constitution, {})", (level / 3) as i32),
                });
            }
        }

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
    deterministic_shuffle(&mut artifacts_files, 4242);

    let total_arts = artifacts_files.len();
    let mut artifacts_ron = String::from("[\n");

    for (idx, filename) in artifacts_files.iter().enumerate() {
        let level = if total_arts > 1 {
            ((idx as f64 / (total_arts - 1) as f64) * 19.0) as u32 + 1
        } else {
            1
        };

        let stem = Path::new(filename).file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
        let mut clean_stem = stem;
        for prefix in
            &["Herbalism_", "Jewelry_", "Mining_", "skinning_", "Res_", "Loot_", "Cooking_"]
        {
            if clean_stem.to_lowercase().starts_with(&prefix.to_lowercase()) {
                clean_stem = &clean_stem[prefix.len()..];
            }
        }
        let mut cleaned = clean_name(clean_stem);
        if cleaned.is_empty() {
            cleaned = "Artifact".to_string();
        }

        if cleaned == "Ironoreodds" {
            cleaned = "Iron Ore".to_string();
        } else if cleaned == "Goldore" {
            cleaned = "Gold Ore".to_string();
        } else if cleaned == "Silverore" {
            cleaned = "Silver Ore".to_string();
        } else if cleaned == "Nativecopper" {
            cleaned = "Copper Ore".to_string();
        } else if cleaned == "Cobaltore" {
            cleaned = "Cobalt Ore".to_string();
        } else if cleaned == "Manamushroom" {
            cleaned = "Mana Mushroom".to_string();
        } else if cleaned == "Demonmushroom" {
            cleaned = "Demon Mushroom".to_string();
        } else if cleaned == "Greencrystal" {
            cleaned = "Green Crystal".to_string();
        } else if cleaned == "Redcrystal" {
            cleaned = "Red Crystal".to_string();
        } else if cleaned == "Bluecrystal" {
            cleaned = "Blue Crystal".to_string();
        } else if cleaned == "Eye" {
            cleaned = "Monster Eye".to_string();
        } else if cleaned == "Goo" {
            cleaned = "Ectoplasm Goo".to_string();
        } else if cleaned == "Shell" {
            cleaned = "Ocean Shell".to_string();
        } else if cleaned == "Sting" {
            cleaned = "Poisonous Sting".to_string();
        } else if cleaned == "Bark" {
            cleaned = "Tree Bark".to_string();
        } else if cleaned == "Bone" {
            cleaned = "Ancient Bone".to_string();
        } else if cleaned == "Spiderteeth" {
            cleaned = "Spider Teeth".to_string();
        } else if cleaned == "Horn" {
            cleaned = "Animal Horn".to_string();
        } else if cleaned == "Fur" {
            cleaned = "Animal Fur".to_string();
        } else if cleaned == "Piece Of Coal" {
            cleaned = "Coal".to_string();
        }

        let name = capitalize_words(&cleaned);
        let img = img_name(filename, img_ext);
        let price = 1 + level * level * 2 + (idx % 2) as u32;
        let kind = classify_artifact_kind(&name);

        artifacts_ron.push_str(&format!(
            "    (\n        name: \"{name}\",\n        image: \"images/catalog/artifacts/{img}\",\n        kind: {kind},\n        level: {level},\n        price: {price},\n    ),\n",
            name = name, img = img, kind = kind, level = level, price = price
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
    weapons_files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut weapon_counts: HashMap<String, u32> = HashMap::new();
    for (f, _) in &weapons_files {
        let mut c = clean_name(f);
        if c.is_empty() {
            c = "Steel Weapon".to_string();
        }
        *weapon_counts.entry(c).or_insert(0) += 1;
    }

    let total_wps = weapons_files.len();
    let chunk_wps = total_wps as f64 / 20.0;
    let mut weapons_ron = String::from("[\n");
    let mut seen_weapons = Vec::new();

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
        } else if lower.contains("dagger") || lower.contains("rapier") || lower.contains("katar") {
            "Finesse"
        } else {
            "Melee"
        };

        let kind = if [
            "fire",
            "pyro",
            "flame",
            "infernal",
            "burn",
            "cinder",
            "combustion",
            "lava",
            "blast",
            "sun",
            "phoenix",
            "red",
            "heat",
            "ash",
            "meteor",
            "scorch",
            "singe",
            "magma",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            "Fire"
        } else if [
            "frost", "ice", "chill", "cold", "blizzard", "glacial", "tomb", "shackle", "freeze",
            "crystal", "snow", "hail", "winter", "blue", "shiver",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            "Ice"
        } else if [
            "holy", "smite", "divine", "radiance", "lay", "judgment", "sacred", "bastion", "light",
            "heal", "bless", "angel", "glory", "pray", "aura", "cure", "priest", "cleric",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            "Holy"
        } else if [
            "shadow",
            "dark",
            "curse",
            "vampiric",
            "agony",
            "soul",
            "covenant",
            "withering",
            "drain",
            "death",
            "devil",
            "demonic",
            "unholy",
            "evil",
            "hex",
            "blackwater",
            "plague",
            "fear",
            "terror",
            "ghoul",
            "spirit",
            "chain",
            "shackle",
            "doom",
            "necrom",
            "void",
            "abyss",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            "Shadow"
        } else if [
            "nature", "bramble", "wild", "thorn", "bloom", "oak", "earth", "growth", "root",
            "leaf", "spore", "ivy", "forest", "grove", "poison", "venom", "toxic",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            "Nature"
        } else {
            "Physical"
        };

        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Steel Weapon".to_string();
        }
        let is_unique = *weapon_counts.get(&cleaned).unwrap_or(&0) == 1;
        let adj = LEVEL_ADJECTIVES[level as usize - 1];
        let mut name = if is_unique {
            cleaned.to_lowercase()
        } else {
            format!("{adj} {cleaned}").to_lowercase()
        };
        let mut ctr = 1;
        while seen_weapons.contains(&capitalize_words(&name)) {
            let mi = (idx + ctr) % UNIQUE_MODIFIERS.len();
            name = if is_unique {
                format!("{} {}", cleaned.to_lowercase(), UNIQUE_MODIFIERS[mi])
            } else {
                format!("{adj} {cleaned} {}", UNIQUE_MODIFIERS[mi]).to_lowercase()
            };
            ctr += 1;
        }
        let name = capitalize_words(&name);
        seen_weapons.push(name.clone());

        let price = 10 + level * level * 25 + (idx % 10) as u32 * 15 + (idx % 3) as u32 * 3;
        let hm = if hand == "TwoHand" {
            2.0f32
        } else {
            1.0
        };
        let mut attack = 0u32;
        let mut speed = 0.0f32;
        let mut crit = 0.0f32;
        let mut modifiers = Vec::new();
        let mut effects = Vec::new();

        match category {
            "Shield" => {
                modifiers.push(format!("DefenseModifier({})", ((level as f32 + 1.0) * hm) as i32));
                if level >= 8 {
                    effects.push(format!(
                        "Thorns(damage_reflected_pct: {:.1}, duration: 4.0)",
                        10.0 + level as f32 * 2.0
                    ));
                }
            },
            "Book" => {
                modifiers.push(format!("AttributeModifier(Intelligence, {})", level as i32));
                if level >= 5 {
                    effects.push(format!("ManaFlow(amount: {}, duration: 5.0)", level + 2));
                }
            },
            "Magical" => {
                attack = (level as f32 * hm) as u32;
                speed = 1.0;
                modifiers.push(format!("AttributeModifier(Intelligence, {})", level as i32));
                if level >= 8 {
                    effects.push(format!(
                        "Clearcasting(reduction_pct: 20.0, duration: {:.1})",
                        3.0 + level as f32 * 0.2
                    ));
                }
            },
            "Finesse" => {
                attack = (level as f32 * hm) as u32;
                speed = 1.4;
                crit = 0.10 + level as f32 * 0.01;
                modifiers.push(format!("AttributeModifier(Dexterity, {})", level as i32));
                if level >= 8 {
                    effects.push(format!(
                        "Lifesteal(percentage: {:.1}, duration: 4.0)",
                        5.0 + level as f32 * 0.5
                    ));
                }
            },
            "Range" => {
                attack = ((level as f32 + 1.0) * hm) as u32;
                speed = 0.9;
                crit = 0.03 + level as f32 * 0.005;
                modifiers.push(format!("AttributeModifier(Dexterity, {})", level as i32));
                if level >= 8 {
                    effects.push(format!(
                        "Blind(miss_pct: 25.0, duration: {:.1})",
                        3.0 + level as f32 * 0.1
                    ));
                }
            },
            _ => {
                attack = ((level as f32 + 1.0) * hm) as u32;
                speed = 1.1;
                crit = 0.05 + level as f32 * 0.008;
                modifiers.push(format!("AttributeModifier(Strength, {})", level as i32));
                if level >= 8 {
                    effects.push(format!("Bleed(damage_pct: {:.1})", 15.0 + level as f32 * 1.5));
                }
            },
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
    armor_files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut armor_counts: HashMap<String, u32> = HashMap::new();
    for (f, _, _, _) in &armor_files {
        let mut c = clean_name(f);
        if c.is_empty() {
            c = "Armor".to_string();
        }
        *armor_counts.entry(c).or_insert(0) += 1;
    }

    let total_arm = armor_files.len();
    let chunk_arm = total_arm as f64 / 20.0;
    let mut armor_ron = String::from("[\n");
    let mut seen_armor = Vec::new();

    for (idx, (filename, _, folder, slot)) in armor_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_arm) as u32 + 1;
        if level > 20 {
            level = 20;
        }
        let lower = filename.to_lowercase();

        let kind = if ["leather", "scout", "assassin", "ranger", "poison", "venom", "toxic"]
            .iter()
            .any(|x| lower.contains(x))
        {
            "Nature"
        } else if [
            "silk", "robe", "mage", "cloth", "wizard", "priest", "holy", "divine", "cleric",
            "solomon",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            "Holy"
        } else if ["dark", "shadow", "void", "curse", "unholy", "necrom", "abyss"]
            .iter()
            .any(|x| lower.contains(x))
        {
            "Shadow"
        } else if ["fire", "pyro", "flame", "lava", "sun", "phoenix", "scorch"]
            .iter()
            .any(|x| lower.contains(x))
        {
            "Fire"
        } else if ["frost", "ice", "chill", "cold", "winter"].iter().any(|x| lower.contains(x)) {
            "Ice"
        } else {
            "Physical"
        };

        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Armor".to_string();
        }
        let is_unique = *armor_counts.get(&cleaned).unwrap_or(&0) == 1;
        let adj = LEVEL_ADJECTIVES[level as usize - 1];
        let mut name = if is_unique {
            cleaned.to_lowercase()
        } else {
            format!("{adj} {cleaned}").to_lowercase()
        };
        let mut ctr = 1;
        while seen_armor.contains(&capitalize_words(&name)) {
            let mi = (idx + ctr) % UNIQUE_MODIFIERS.len();
            name = if is_unique {
                format!("{} {}", cleaned.to_lowercase(), UNIQUE_MODIFIERS[mi])
            } else {
                format!("{adj} {cleaned} {}", UNIQUE_MODIFIERS[mi]).to_lowercase()
            };
            ctr += 1;
        }
        let name = capitalize_words(&name);
        seen_armor.push(name.clone());

        let price = 10 + level * level * 20 + (idx % 10) as u32 * 12 + (idx % 3) as u32 * 2;
        let mut modifiers = Vec::new();
        let mut effects = Vec::new();

        match slot.as_str() {
            "Chestplate" | "Helmet" | "Boots" | "Gloves" => {
                modifiers.push(format!("DefenseModifier({})", level as i32 * 2 + 1));
                let attr = match kind {
                    "Nature" => "Dexterity",
                    "Holy" | "Shadow" | "Fire" | "Ice" => "Intelligence",
                    _ => "Constitution",
                };
                modifiers.push(format!("AttributeModifier({}, {})", attr, (level as i32 + 1) / 2));
            },
            "Accessory" => {
                if idx % 2 == 0 {
                    modifiers.push(format!("MaxHealthModifier({})", (level * 8) as i32));
                } else {
                    modifiers.push(format!("MaxManaModifier({})", (level * 4) as i32));
                }
            },
            _ => {
                modifiers.push(format!("HealthRegen({})", (level as i32 + 1) / 2));
            },
        }
        if level >= 8 && slot != "Accessory" && slot != "Consumable" {
            if kind == "Physical" {
                effects.push(format!(
                    "Thorns(damage_reflected_pct: {:.1}, duration: 3.0)",
                    5.0 + level as f32
                ));
            } else if kind == "Nature" {
                effects.push(format!(
                    "Freeze(attack_speed_pct: -15.0, duration: {:.1})",
                    2.0 + level as f32 * 0.1
                ));
            } else {
                effects.push(format!("Regen(heal: {}, duration: 4.0)", level + 1));
            }
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
    consumables_files.sort();

    let total_cons = consumables_files.len();
    let chunk_cons = total_cons as f64 / 20.0;
    let mut seen_consumables = Vec::new();
    let mut consumables_ron = String::from("[\n");

    for (idx, filename) in consumables_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_cons) as u32 + 1;
        if level > 20 {
            level = 20;
        }
        let lower = filename.to_lowercase();
        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Potion".to_string();
        }

        let mut name = cleaned.to_lowercase();
        let mut ctr = 1;
        while seen_consumables.contains(&capitalize_words(&name)) {
            name = format!(
                "{} {}",
                cleaned.to_lowercase(),
                UNIQUE_MODIFIERS[(idx + ctr) % UNIQUE_MODIFIERS.len()]
            );
            ctr += 1;
        }
        let name = capitalize_words(&name);
        seen_consumables.push(name.clone());

        let price = 5 + level * level * 5 + (idx % 5) as u32 * 4;
        let mut effects = Vec::new();

        if lower.contains("mana") || lower.contains("energy") {
            if level < 5 {
                effects.push(format!("InstantMana(amount: {})", level * 15 + 10));
            } else {
                effects.push(format!("InstantMana(amount: {})", level * 20 + 20));
                effects.push(format!("ManaFlow(amount: {}, duration: 5.0)", level + 2));
            }
        } else if lower.contains("health") || lower.contains("green") {
            if level < 5 {
                effects.push(format!("Heal(heal_pct: {})", 20 + level * 5));
            } else {
                effects.push(format!("Heal(heal_pct: {})", 30 + level * 3));
                effects.push(format!("Regen(heal: {}, duration: 5.0)", level + 2));
            }
        } else if lower.contains("king") || lower.contains("spider") || lower.contains("shadow") {
            let stat = ["Strength", "Dexterity", "Constitution", "Intelligence"][idx % 4];
            effects.push(format!(
                "StatBoost(attribute: {}, amount: {}, duration: 10.0)",
                stat,
                (level + 2) / 2
            ));
            effects.push(format!("Heal(heal_pct: {})", 15 + level * 2));
            if idx % 2 == 0 {
                effects.push(format!("InstantMana(amount: {})", level * 10 + 10));
            }
        } else if idx % 2 == 0 {
            effects.push(format!("Heal(heal_pct: {})", 15 + level * 4));
        } else {
            effects.push(format!("InstantMana(amount: {})", level * 15 + 10));
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
}

fn main() {
    #[cfg(feature = "process-assets")]
    let img_ext = "ktx2";
    #[cfg(not(feature = "process-assets"))]
    let img_ext = "png";

    run("assets-src/images", "assets/inventory", img_ext);
}
