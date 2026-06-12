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

fn main() {
    fs::create_dir_all("assets/inventory").unwrap();

    // 1. ABILITIES
    let abilities_dir = "assets/images/build/abilities";
    let mut abilities_files = Vec::new();
    if let Ok(entries) = fs::read_dir(abilities_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().to_lowercase() == "png" {
                            let filename = path.file_name().unwrap().to_string_lossy().to_string();
                            let score = get_image_score(&filename);
                            abilities_files.push((filename, score));
                        }
                    }
                }
            }
        }
    }
    abilities_files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

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
            "fire", "pyro", "flame", "infernal", "burn", "cinder", "combustion", "lava", "blast",
            "sun", "phoenix", "red", "heat", "ash", "meteor", "scorch", "singe", "magma",
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
            "heal", "shield", "bless", "angel", "glory", "pray", "aura", "cure", "priest", "cleric",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Holy";
            pool = HOLY_POOL;
        } else if [
            "shadow", "dark", "curse", "vampiric", "agony", "soul", "covenant", "withering", "drain",
            "death", "devil", "demonic", "unholy", "evil", "hex", "blackwater", "plague", "fear",
            "terror", "ghoul", "spirit", "chain", "shackle", "doom", "necrom",
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
        let is_aoe = (level % 4 == 0)
            || ["wave", "rain", "blizzard", "storm", "aoe", "clones", "explode"]
                .iter()
                .any(|x| lower.contains(x));

        let mut effects = Vec::new();
        let mut on_self = false;

        match kind {
            "Fire" => {
                effects.push(format!(
                    "Burn(damage: {}, duration: 4.0)",
                    level * 2 + (idx % 3) as u32
                ));
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
                    effects.push(format!(
                        "Immobilize(duration: {:.1})",
                        2.0 + (idx % 3) as f32 * 0.5
                    ));
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
                        5.0 + level as f32 * 1.0
                    ));
                } else {
                    effects.push(format!(
                        "Paranoia(initiative_pct: {:.1}, duration: 5.0)",
                        -5.0 - level as f32 * 1.0
                    ));
                }
            },
            _ => { // Physical
                if idx % 3 == 0 {
                    effects.push(format!(
                        "Bleed(damage_pct: {:.1})",
                        10.0 + level as f32 * 5.0
                    ));
                } else if idx % 3 == 1 {
                    effects.push(format!(
                        "Cleave(damage_pct: {:.1}, duration: 4.0)",
                        20.0 + level as f32 * 2.0
                    ));
                } else {
                    effects.push(format!("Pierce(damage: {})", level * 5));
                }
            }
        }
        let effects_str = effects.join(", ");

        abilities_ron.push_str(&format!(
            "    (
        name: \"{}\",
        image: \"images/build/abilities/{}\",
        kind: {},
        level: {},
        mana_cost: {},
        cooldown: {:.1},
        on_self: {},
        is_aoe: {},
        effects: [{}],
    ),
",
            name,
            filename,
            kind,
            level,
            mana_cost,
            cooldown,
            on_self,
            is_aoe,
            effects_str
        ));
    }
    abilities_ron.push_str("]\n");
    let mut file = File::create("assets/inventory/abilities.ron").unwrap();
    file.write_all(abilities_ron.as_bytes()).unwrap();
    println!("Generated {} abilities in abilities.ron", total_abs);

    // 2. PERKS
    let perks_dir = "assets/images/build/perks";
    let mut perks_files = Vec::new();
    if let Ok(entries) = fs::read_dir(perks_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().to_lowercase() == "png" {
                            let filename = path.file_name().unwrap().to_string_lossy().to_string();
                            let score = get_image_score(&filename);
                            perks_files.push((filename, score));
                        }
                    }
                }
            }
        }
    }
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
            "fire", "pyro", "flame", "infernal", "burn", "cinder", "combustion", "lava", "blast",
            "sun", "phoenix", "red", "heat", "ash", "meteor", "scorch", "singe", "magma",
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
            "heal", "shield", "bless", "angel", "glory", "pray", "aura", "cure", "priest", "cleric",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            pool = HOLY_POOL;
        } else if [
            "shadow", "dark", "curse", "vampiric", "agony", "soul", "covenant", "withering", "drain",
            "death", "devil", "demonic", "unholy", "evil", "hex", "blackwater", "plague", "fear",
            "terror", "ghoul", "spirit", "chain", "shackle", "doom", "necrom",
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
            let mod_idx = (idx + ctr as usize) % UNIQUE_MODIFIERS.len();
            name = format!("{} {}", cleaned.to_lowercase(), UNIQUE_MODIFIERS[mod_idx]);
            ctr += 1;
        }
        let name = capitalize_words(&name);
        seen_perks.push(name.clone());

        let mut modifiers = Vec::new();
        let attrs = ["Strength", "Dexterity", "Constitution", "Intelligence", "Wisdom", "Charisma"];

        // Determine modifiers based on level
        if level < 5 {
            // Low level perks: just one modifier (usually AttributeModifier, but some give other things)
            // unless they are the ones giving a negative modifier as well.
            if idx % 3 == 0 {
                // Positive attribute modifier (boosted so net sum is always positive)
                let attr_val = level as i32;
                let pos_attr = attrs[idx % attrs.len()];
                let neg_attr = attrs[(idx + 1 + (idx / attrs.len()) % (attrs.len() - 1)) % attrs.len()];
                modifiers.push(format!("AttributeModifier({}, {})", pos_attr, attr_val + 1));

                // And a negative modifier besides the positive!
                let neg_val = -attr_val;
                modifiers.push(format!("AttributeModifier({}, {})", neg_attr, neg_val));
            } else if idx % 3 == 1 {
                // Something else than AttributeModifier (just one!)
                let other_mod = match (idx + level as usize) % 5 {
                    0 => format!("AttackModifier({})", level as i32),
                    1 => format!("DefenseModifier({})", level as i32),
                    2 => format!("InitiativeModifier({})", level as i32),
                    3 => format!("MaxHealthModifier({})", (level * 10) as i32),
                    _ => format!("MaxManaModifier({})", (level * 5) as i32),
                };
                modifiers.push(other_mod);
            } else {
                // Regular attribute modifier, and just one!
                let attr_val = level as i32;
                let attr_name = attrs[idx % attrs.len()];
                modifiers.push(format!("AttributeModifier({}, {})", attr_name, attr_val));
            }
        } else {
            // High level perks (level >= 5)
            // 1. Primary attribute
            let attr_val = level as i32;
            let attr_name = attrs[idx % attrs.len()];
            modifiers.push(format!("AttributeModifier({}, {})", attr_name, attr_val));

            // 2. Extra modifiers for higher levels
            if level >= 5 {
                let extra_mod = match (idx + level as usize) % 6 {
                    0 => format!("MaxHealthModifier({})", (level * 10) as i32),
                    1 => format!("MaxManaModifier({})", (level * 5) as i32),
                    2 => format!("AttackModifier({})", (level as i32 + 1) / 2),
                    3 => format!("DefenseModifier({})", (level as i32 + 1) / 2),
                    4 => format!("InitiativeModifier({})", (level as i32 + 2) / 3),
                    _ => format!("LifeSteal({:.1})", level as f32 * 1.0),
                };
                modifiers.push(extra_mod);
            }
            if level >= 13 {
                let third_mod = match (idx + 1) % 4 {
                    0 => format!("HealthRegen({})", (level / 4) as i32 + 1),
                    1 => format!("ManaRegen({})", (level / 5) as i32 + 1),
                    2 => format!("HealingMultiplier({:.1})", level as f32 * 1.5),
                    _ => format!("AttributeModifier(Constitution, {})", (level / 3) as i32),
                };
                modifiers.push(third_mod);
            }
        }

        let mods_str = modifiers.join(", ");

        perks_ron.push_str(&format!(
            "    (
        name: \"{}\",
        image: \"images/build/perks/{}\",
        level: {},
        modifiers: [{}],
    ),
",
            name, filename, level, mods_str
        ));
    }
    perks_ron.push_str("]\n");
    let mut file = File::create("assets/inventory/perks.ron").unwrap();
    file.write_all(perks_ron.as_bytes()).unwrap();
    println!("Generated {} perks in perks.ron", total_pks);

    // 3. WEAPONS
    let weapons_dir = "assets/images/build/equipment/weapon";
    let mut weapons_files = Vec::new();
    if let Ok(entries) = fs::read_dir(weapons_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().to_lowercase() == "png" {
                            let filename = path.file_name().unwrap().to_string_lossy().to_string();
                            let lower = filename.to_lowercase();
                            if lower.contains("arrow")
                                || lower.contains("quiver")
                                || lower.contains("bullet")
                                || lower.contains("bolt")
                            {
                                continue;
                            }
                            let score = get_image_score(&filename);
                            weapons_files.push((filename, score));
                        }
                    }
                }
            }
        }
    }
    weapons_files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut weapon_cleaned_counts = std::collections::HashMap::new();
    for (filename, _) in &weapons_files {
        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Steel Weapon".to_string();
        }
        *weapon_cleaned_counts.entry(cleaned).or_insert(0) += 1;
    }

    let total_wps = weapons_files.len();
    let chunk_size_wps = total_wps as f64 / 20.0;
    let mut weapons_ron = String::from("[\n");
    let mut seen_weapons = Vec::new();

    for (idx, (filename, _)) in weapons_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_size_wps) as u32 + 1;
        if level > 20 {
            level = 20;
        }

        let lower = filename.to_lowercase();
        let mut hand = "OneHand";
        if ["bow", "staff", "two", "2h", "great", "spear", "halberd", "scythe", "claymore"]
            .iter()
            .any(|x| lower.contains(x))
        {
            hand = "TwoHand";
        }

        let mut kind = "Physical";
        let mut category = "Melee";

        if lower.contains("shield") {
            category = "Shield";
        } else if lower.contains("book") || lower.contains("scroll") || lower.contains("tome") {
            category = "Book";
        } else if lower.contains("wand") || lower.contains("staff") || lower.contains("scepter") {
            category = "Magical";
        } else if lower.contains("bow") || lower.contains("crossbow") || lower.contains("sling") {
            category = "Range";
        } else if lower.contains("dagger") || lower.contains("rapier") || lower.contains("katar") {
            category = "Finesse";
        }

        if [
            "fire", "pyro", "flame", "infernal", "burn", "cinder", "combustion", "lava", "blast",
            "sun", "phoenix", "red", "heat", "ash", "meteor", "scorch", "singe", "magma",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Fire";
        } else if [
            "frost", "ice", "chill", "cold", "blizzard", "glacial", "tomb", "shackle", "freeze",
            "crystal", "snow", "hail", "winter", "blue", "shiver",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Ice";
        } else if [
            "holy", "smite", "divine", "radiance", "lay", "judgment", "sacred", "bastion", "light",
            "heal", "bless", "angel", "glory", "pray", "aura", "cure", "priest", "cleric",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Holy";
        } else if [
            "shadow", "dark", "curse", "vampiric", "agony", "soul", "covenant", "withering", "drain",
            "death", "devil", "demonic", "unholy", "evil", "hex", "blackwater", "plague", "fear",
            "terror", "ghoul", "spirit", "chain", "shackle", "doom", "necrom", "void", "abyss",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Shadow";
        } else if [
            "nature", "bramble", "wild", "thorn", "bloom", "oak", "earth", "growth", "root",
            "leaf", "spore", "ivy", "forest", "grove", "poison", "venom", "toxic",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Nature";
        }

        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Steel Weapon".to_string();
        }

        let is_unique = *weapon_cleaned_counts.get(&cleaned).unwrap_or(&0) == 1;
        let mut name = if is_unique {
            cleaned.to_lowercase()
        } else {
            let adj = LEVEL_ADJECTIVES[level as usize - 1];
            format!("{} {}", adj, cleaned).to_lowercase()
        };

        let mut ctr = 1;
        while seen_weapons.contains(&capitalize_words(&name)) {
            let mod_idx = (idx + ctr as usize) % UNIQUE_MODIFIERS.len();
            if is_unique {
                name = format!("{} {}", cleaned.to_lowercase(), UNIQUE_MODIFIERS[mod_idx])
                    .to_lowercase();
            } else {
                let adj = LEVEL_ADJECTIVES[level as usize - 1];
                name = format!("{} {} {}", adj, cleaned, UNIQUE_MODIFIERS[mod_idx]).to_lowercase();
            }
            ctr += 1;
        }
        let name = capitalize_words(&name);
        seen_weapons.push(name.clone());

        let price = 10 + level * level * 25 + (idx % 10) as u32 * 15 + (idx % 3) as u32 * 3;

        let mut attack = 0;
        let mut attack_speed = 0.0;
        let mut crit_chance = 0.0;
        let mut modifiers = Vec::new();
        let mut effects = Vec::new();

        let hand_mult = if hand == "TwoHand" { 2.0 } else { 1.0 };

        match category {
            "Shield" => {
                let def_val = ((level as f32 + 1.0) * hand_mult) as i32;
                modifiers.push(format!("DefenseModifier({})", def_val));
                if level >= 8 {
                    effects.push(format!("Thorns(damage_reflected_pct: {:.1}, duration: 4.0)", 10.0 + level as f32 * 2.0));
                }
            },
            "Book" => {
                let int_val = level as i32;
                modifiers.push(format!("AttributeModifier(Intelligence, {})", int_val));
                if level >= 5 {
                    effects.push(format!("ManaFlow(amount: {}, duration: 5.0)", level + 2));
                }
            },
            "Magical" => {
                attack = (level as f32 * hand_mult) as u32;
                attack_speed = 1.0;
                let int_val = level as i32;
                modifiers.push(format!("AttributeModifier(Intelligence, {})", int_val));
                if level >= 8 {
                    effects.push(format!("Clearcasting(reduction_pct: 20.0, duration: {:.1})", 3.0 + level as f32 * 0.2));
                }
            },
            "Finesse" => {
                attack = (level as f32 * hand_mult) as u32;
                attack_speed = 1.4;
                crit_chance = 0.10 + level as f32 * 0.01;
                modifiers.push(format!("AttributeModifier(Dexterity, {})", level as i32));
                if level >= 8 {
                    effects.push(format!("Lifesteal(percentage: {:.1}, duration: 4.0)", 5.0 + level as f32 * 0.5));
                }
            },
            "Range" => {
                attack = ((level as f32 + 1.0) * hand_mult) as u32;
                attack_speed = 0.9;
                crit_chance = 0.03 + level as f32 * 0.005;
                modifiers.push(format!("AttributeModifier(Dexterity, {})", level as i32));
                if level >= 8 {
                    effects.push(format!("Blind(miss_pct: 25.0, duration: {:.1})", 3.0 + level as f32 * 0.1));
                }
            },
            _ => { // Melee
                attack = ((level as f32 + 1.0) * hand_mult) as u32;
                attack_speed = 1.1;
                crit_chance = 0.05 + level as f32 * 0.008;
                modifiers.push(format!("AttributeModifier(Strength, {})", level as i32));
                if level >= 8 {
                    effects.push(format!("Bleed(damage_pct: {:.1})", 15.0 + level as f32 * 1.5));
                }
            }
        }

        let mods_str = modifiers.join(", ");
        let effects_str = effects.join(", ");

        weapons_ron.push_str(&format!(
            "    (
        name: \"{}\",
        image: \"images/build/equipment/weapon/{}\",
        kind: {},
        category: {},
        hand: {},
        level: {},
        price: {},
        attack: {},
        attack_speed: {:.2},
        crit_chance: {:.2},
        modifiers: [{}],
        effects: [{}],
    ),
",
            name, filename, kind, category, hand, level, price, attack, attack_speed, crit_chance, mods_str, effects_str
        ));
    }
    weapons_ron.push_str("]\n");
    let mut file = File::create("assets/inventory/weapons.ron").unwrap();
    file.write_all(weapons_ron.as_bytes()).unwrap();
    println!("Generated {} weapons in weapons.ron", total_wps);

    // 4. ARMOR / WEARABLES
    let armor_folders = [
        ("accessory", "Accessory"),
        ("armor", "Chestplate"),
        ("boots", "Boots"),
        ("consumable", "Consumable"),
        ("gloves", "Gloves"),
        ("helmet", "Helmet"),
    ];

    let mut armor_files = Vec::new();
    for (folder, slot) in &armor_folders {
        let dir_path = format!("assets/images/build/equipment/{}", folder);
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext.to_string_lossy().to_lowercase() == "png" {
                                let filename =
                                    path.file_name().unwrap().to_string_lossy().to_string();
                                let score = get_image_score(&filename);
                                armor_files.push((
                                    filename,
                                    score,
                                    folder.to_string(),
                                    slot.to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    armor_files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut armor_cleaned_counts = std::collections::HashMap::new();
    for (filename, _, _, _) in &armor_files {
        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Armor".to_string();
        }
        *armor_cleaned_counts.entry(cleaned).or_insert(0) += 1;
    }

    let total_arm = armor_files.len();
    let chunk_size_arm = total_arm as f64 / 20.0;
    let mut armor_ron = String::from("[\n");
    let mut seen_armor = Vec::new();

    for (idx, (filename, _, folder, slot)) in armor_files.iter().enumerate() {
        let mut level = (idx as f64 / chunk_size_arm) as u32 + 1;
        if level > 20 {
            level = 20;
        }

        let lower = filename.to_lowercase();
        let mut kind = "Physical";
        if ["leather", "scout", "assassin", "ranger", "poison", "venom", "toxic"].iter().any(|x| lower.contains(x)) {
            kind = "Nature";
        } else if ["silk", "robe", "mage", "cloth", "wizard", "priest", "holy", "divine", "cleric", "solomon"].iter().any(|x| lower.contains(x)) {
            kind = "Holy";
        } else if ["dark", "shadow", "void", "curse", "unholy", "necrom", "abyss"].iter().any(|x| lower.contains(x)) {
            kind = "Shadow";
        } else if ["fire", "pyro", "flame", "lava", "sun", "phoenix", "scorch"].iter().any(|x| lower.contains(x)) {
            kind = "Fire";
        } else if ["frost", "ice", "chill", "cold", "winter"].iter().any(|x| lower.contains(x)) {
            kind = "Ice";
        }

        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Armor".to_string();
        }

        let is_unique = *armor_cleaned_counts.get(&cleaned).unwrap_or(&0) == 1;
        let mut name = if is_unique {
            cleaned.to_lowercase()
        } else {
            let adj = LEVEL_ADJECTIVES[level as usize - 1];
            format!("{} {}", adj, cleaned).to_lowercase()
        };

        let mut ctr = 1;
        while seen_armor.contains(&capitalize_words(&name)) {
            let mod_idx = (idx + ctr as usize) % UNIQUE_MODIFIERS.len();
            if is_unique {
                name = format!("{} {}", cleaned.to_lowercase(), UNIQUE_MODIFIERS[mod_idx])
                    .to_lowercase();
            } else {
                let adj = LEVEL_ADJECTIVES[level as usize - 1];
                name = format!("{} {} {}", adj, cleaned, UNIQUE_MODIFIERS[mod_idx]).to_lowercase();
            }
            ctr += 1;
        }
        let name = capitalize_words(&name);
        seen_armor.push(name.clone());

        let price = 10 + level * level * 20 + (idx % 10) as u32 * 12 + (idx % 3) as u32 * 2;

        let mut modifiers = Vec::new();
        let mut effects = Vec::new();

        match slot.as_str() {
            "Chestplate" | "Helmet" | "Boots" | "Gloves" => {
                let def_val = level as i32 * 2 + 1;
                modifiers.push(format!("DefenseModifier({})", def_val));
                
                let attr_name = match kind {
                    "Nature" => "Dexterity",
                    "Holy" | "Shadow" | "Fire" | "Ice" => "Intelligence",
                    _ => "Constitution",
                };
                modifiers.push(format!("AttributeModifier({}, {})", attr_name, (level as i32 + 1) / 2));
            },
            "Accessory" => {
                if idx % 2 == 0 {
                    modifiers.push(format!("MaxHealthModifier({})", (level * 8) as i32));
                } else {
                    modifiers.push(format!("MaxManaModifier({})", (level * 4) as i32));
                }
            },
            _ => { // Consumable
                modifiers.push(format!("HealthRegen({})", (level as i32 + 1) / 2));
            }
        }

        if level >= 8 && slot != "Accessory" && slot != "Consumable" {
            if kind == "Physical" {
                effects.push(format!("Thorns(damage_reflected_pct: {:.1}, duration: 3.0)", 5.0 + level as f32 * 1.0));
            } else if kind == "Nature" {
                effects.push(format!("Freeze(attack_speed_pct: -15.0, duration: {:.1})", 2.0 + level as f32 * 0.1));
            } else {
                effects.push(format!("Regen(heal: {}, duration: 4.0)", level + 1));
            }
        }

        let mods_str = modifiers.join(", ");
        let effects_str = effects.join(", ");

        armor_ron.push_str(&format!(
            "    (
        name: \"{}\",
        image: \"images/build/equipment/{}/{}\",
        kind: {},
        price: {},
        slot: {},
        modifiers: [{}],
        effects: [{}],
        level: {},
    ),
",
            name, folder, filename, kind, price, slot, mods_str, effects_str, level
        ));
    }
    armor_ron.push_str("]\n");
    let mut file = File::create("assets/inventory/wearables.ron").unwrap();
    file.write_all(armor_ron.as_bytes()).unwrap();
    println!("Generated {} wearables in wearables.ron", total_arm);
}
