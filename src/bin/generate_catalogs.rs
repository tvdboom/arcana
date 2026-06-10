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

const COSMIC_POOL: &[&str] = &[
    "Chronos Rift",
    "Supernova Burst",
    "Astral Alignment",
    "Gravity Well",
    "Nebula Shroud",
    "Cosmic Flux",
    "Celestial Gateway",
    "Starfall Cascade",
    "Galaxy Swirl",
    "Singularity Event",
    "Nebula Mist",
    "Pulsar Beam",
    "Starlight Blessing",
    "Void Warp",
    "Quantum Jump",
    "Aetherial Shield",
    "Temporal Loop",
    "Solar Flare",
    "Moonbeam Aegis",
    "Eclipse Strike",
    "Nebulous Barrier",
    "Star Shard",
    "Supernova Blast",
    "Black Hole Vortex",
    "Astral Grace",
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

const LIGHTNING_POOL: &[&str] = &[
    "Chain Capacitor",
    "Overload Surge",
    "Thunderclad Dash",
    "Static Discharge",
    "Volt Strike",
    "Thunder Clap",
    "Tesla Overcharge",
    "Lightning Rod",
    "Galvanic Arc",
    "Storm Barrier",
    "Shocking Grasp",
    "Fulgurite Spike",
    "Plasma Ray",
    "Ion Storm",
    "Blitz Speed",
    "Magnetic Pull",
    "Current Ripple",
    "Spark shower",
    "Sonic Boom",
    "Nimbus Ride",
    "Stormlord Call",
    "Electro Spark",
    "Volt Barrier",
    "Sky Strike",
    "Flash Bolt",
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

const MARTIAL_POOL: &[&str] = &[
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
];

const BULWARK_POOL: &[&str] = &[
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
];

const ASSASSINATION_POOL: &[&str] = &[
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
];

const SKIRMISH_POOL: &[&str] = &[
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

const TACTIC_POOL: &[&str] = &[
    "Precision Aim",
    "Find Weakness",
    "Expose Armor",
    "Tactical Focus",
    "Target Mark",
    "Analyze Flow",
    "Calculated Strike",
    "Clever Trap",
    "Premeditated Shot",
    "Smart Combat",
    "Cunning Plan",
    "Disarming Strike",
    "Exploit Gap",
    "Battle Map",
    "Strategic Position",
];

const COMMAND_POOL: &[&str] = &[
    "Rallying Cry",
    "Inspirational Horn",
    "Pack Leader",
    "Summon Ally",
    "Beast Frenzy",
    "Commanding Voice",
    "Roar of Triumph",
    "Call of the Wild",
    "Minion Fury",
    "Tame Companion",
    "Battle Orders",
    "Coordinated Attack",
    "Loyal Pack",
    "Alpha Instinct",
    "War Horn",
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
    let abilities_dir = "assets/images/inventory/abilities";
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
        let mut kind = "Nature";
        let mut pool = NATURE_POOL;

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
            kind = "Frost";
            pool = FROST_POOL;
        } else if [
            "lightn",
            "bolt",
            "static",
            "overload",
            "volt",
            "thunder",
            "discharge",
            "capacitor",
            "surge",
            "spark",
            "storm",
            "shock",
            "electric",
            "current",
            "charge",
            "plasma",
            "flash",
            "arc",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Lightning";
            pool = LIGHTNING_POOL;
        } else if [
            "holy", "smite", "divine", "radiance", "lay", "judgment", "sacred", "bastion", "light",
            "heal", "shield", "bless", "angel", "glory", "pray", "aura", "cure",
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
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Shadow";
            pool = SHADOW_POOL;
        } else if [
            "cosmic",
            "chronos",
            "rift",
            "supernova",
            "burst",
            "astral",
            "alignment",
            "gravity",
            "nebula",
            "shroud",
            "moon",
            "star",
            "time",
            "void",
            "portal",
            "eclipse",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Cosmic";
            pool = COSMIC_POOL;
        } else if [
            "slash", "cleave", "cut", "rend", "bash", "pierce", "blade", "sword", "weapon",
            "punch", "kick", "heavy", "martial", "strike",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Martial";
            pool = &[
                "Heavy Strike",
                "Overhead Slash",
                "Cleaving Swing",
                "Shield Bash",
                "Piercing Thrust",
                "Blade Dance",
                "Flurry of Blows",
                "Decisive Cut",
                "Sweeping Kick",
                "Concussive Blow",
            ];
        } else if [
            "assassin",
            "stab",
            "poison",
            "stealth",
            "bleed",
            "dagger",
            "backstab",
            "crit",
            "critical",
            "lethal",
            "venom",
            "shadowstep",
            "cloak",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Assassination";
            pool = &[
                "Backstab",
                "Poison Blade",
                "Stealth Strike",
                "Shadowstep",
                "Lethal Venom",
                "Bleeding Cut",
                "Dagger Fan",
                "Assassinate",
                "Critical Pierce",
                "Cloak of Shadows",
            ];
        } else if [
            "guard",
            "defend",
            "block",
            "wall",
            "fortress",
            "armor",
            "iron",
            "barricade",
            "parry",
            "taunt",
            "bulwark",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Bulwark";
            pool = &[
                "Iron Guard",
                "Shield Block",
                "Unbreakable Wall",
                "Defensive Parry",
                "Challenging Taunt",
                "Fortress Stance",
                "Barricade",
                "Bulwark Shield",
                "Stalwart Defense",
                "Spiked Armor",
            ];
        } else if [
            "bow", "arrow", "shot", "shoot", "ranged", "skirmish", "trap", "hunt", "quiver", "aim",
            "snipe",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Skirmish";
            pool = &[
                "Aimed Shot",
                "Arrow Rain",
                "Piercing Bolt",
                "Skirmish Step",
                "Hunter Trap",
                "Sniper Focus",
                "Quiver Burst",
                "Double Shot",
                "Concussive Arrow",
                "Crippling Shot",
            ];
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
        while seen_abilities.contains(&name) {
            let mod_idx = (idx + ctr as usize) % UNIQUE_MODIFIERS.len();
            name = format!("{} {}", cleaned.to_lowercase(), UNIQUE_MODIFIERS[mod_idx]);
            ctr += 1;
        }
        seen_abilities.push(name.clone());

        let mana_cost = 5 + level * 2 + (idx % 3) as u32;
        let base_stat = 10 + level * 8 + (idx % 5) as u32 * 2;
        let scaling_factor = 0.5 + level as f32 * 0.1 + (idx % 4) as f32 * 0.05;
        let cooldown = 10.0 - level as f32 * 0.3 + (idx % 5) as f32 * 0.2;
        let is_aoe = (level % 4 == 0)
            || ["wave", "rain", "blizzard", "storm", "aoe", "clones", "explode"]
                .iter()
                .any(|x| lower.contains(x));

        let mut effects = Vec::new();
        if level >= 5 {
            match kind {
                "Fire" => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "Burn(damage_per_sec: {}, duration: 4.0)",
                            level * 2 + (idx % 3) as u32
                        ));
                    } else {
                        effects.push(format!(
                            "Empower(damage_bonus_pct: {:.2}, duration: 5.0)",
                            0.05 + (idx % 3) as f32 * 0.02
                        ));
                    }
                },
                "Frost" => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "Freeze(attack_speed_reduction: {:.2}, duration: 3.0)",
                            0.1 + level as f32 * 0.01 + (idx % 3) as f32 * 0.02
                        ));
                    } else {
                        effects.push(format!(
                            "ChillBlast(radius: 3.0, frost_damage: {})",
                            level * 3 + (idx % 5) as u32
                        ));
                    }
                },
                "Lightning" => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "ChainArc(jumps: {}, damage_decay_pct: 0.2)",
                            1 + level / 5 + (idx % 2) as u32
                        ));
                    } else {
                        effects.push(format!(
                            "Haste(initiative_bonus: {:.2}, duration: 4.0)",
                            0.1 + (idx % 3) as f32 * 0.05
                        ));
                    }
                },
                "Nature" => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "Poison(damage_per_sec: {}, duration: 5.0)",
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
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "Regen(heal_per_sec: {}, duration: 5.0)",
                            level * 2 + (idx % 4) as u32
                        ));
                    } else {
                        effects.push("Purge".to_string());
                    }
                },
                "Shadow" => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "Vulnerability(damage_taken_multiplier: {:.2}, duration: 5.0)",
                            1.05 + level as f32 * 0.01 + (idx % 3) as f32 * 0.02
                        ));
                    } else {
                        effects.push(format!(
                            "DoomCurse(stacks_required: 3, explosion_damage: {})",
                            level * 10 + (idx % 5) as u32
                        ));
                    }
                },
                "Martial" => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "Bleed(damage_per_sec: {}, duration: 4.0)",
                            level * 2 + (idx % 3) as u32
                        ));
                    } else {
                        effects.push(format!(
                            "Weaken(attack_power_reduction: {}, duration: 4.0)",
                            level + (idx % 3) as u32
                        ));
                    }
                },
                "Assassination" => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "Lifesteal(percentage: {:.2})",
                            0.05 + level as f32 * 0.01
                        ));
                    } else {
                        effects.push("Blind(miss_chance: 0.25, duration: 3.0)".to_string());
                    }
                },
                "Bulwark" => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "Fortify(armor_bonus_pct: {:.2}, duration: 5.0)",
                            0.10 + level as f32 * 0.01
                        ));
                    } else {
                        effects.push(format!(
                            "Thorns(damage_reflected: {}, duration: 5.0)",
                            level * 2 + (idx % 3) as u32
                        ));
                    }
                },
                "Skirmish" => {
                    if idx % 2 == 0 {
                        effects
                            .push(format!("Stun(duration: {:.1})", 1.0 + (idx % 3) as f32 * 0.5));
                    } else {
                        effects.push(format!(
                            "Focus(crit_chance_bonus: {:.2}, duration: 4.0)",
                            0.05 + level as f32 * 0.01
                        ));
                    }
                },
                _ => {
                    if idx % 2 == 0 {
                        effects.push(format!(
                            "TimeWarp(initiative_reduction: {:.2}, duration: 4.0)",
                            0.05 + level as f32 * 0.01 + (idx % 3) as f32 * 0.01
                        ));
                    } else {
                        effects.push(format!("Silence(duration: {:.1})", 2.0 + (idx % 3) as f32));
                    }
                },
            }
        }
        let effects_str = effects.join(", ");

        abilities_ron.push_str(&format!(
            "    (
        name: \"{}\",
        image: \"images/inventory/abilities/{}\",
        kind: {},
        level: {},
        mana_cost: {},
        base: {},
        scaling_factor: {:.1},
        cooldown: {:.1},
        is_aoe: {},
        modifiers: [],
        effects: [{}],
    ),
",
            name,
            filename,
            kind,
            level,
            mana_cost,
            base_stat,
            scaling_factor,
            cooldown,
            is_aoe,
            effects_str
        ));
    }
    abilities_ron.push_str("]\n");
    let mut file = File::create("assets/inventory/abilities.ron").unwrap();
    file.write_all(abilities_ron.as_bytes()).unwrap();
    println!("Generated {} abilities in abilities.ron", total_abs);

    // 2. PERKS
    let perks_dir = "assets/images/inventory/perks";
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
        let kind;
        let pool;

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
            kind = "Frost";
            pool = FROST_POOL;
        } else if [
            "lightn",
            "bolt",
            "static",
            "overload",
            "volt",
            "thunder",
            "discharge",
            "capacitor",
            "surge",
            "spark",
            "storm",
            "shock",
            "electric",
            "current",
            "charge",
            "plasma",
            "flash",
            "arc",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Lightning";
            pool = LIGHTNING_POOL;
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
            "cosmic",
            "chronos",
            "rift",
            "supernova",
            "burst",
            "astral",
            "alignment",
            "gravity",
            "nebula",
            "shroud",
            "moon",
            "star",
            "time",
            "void",
            "portal",
            "eclipse",
            "space",
            "galaxy",
            "pulsar",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Cosmic";
            pool = COSMIC_POOL;
        } else if [
            "assassin",
            "stab",
            "stealth",
            "hidden",
            "sneak",
            "backstab",
            "dagger",
            "poison",
            "toxin",
            "venom",
            "bleed",
            "lethal",
            "fatal",
            "execut",
            "slice",
            "cut",
            "silent",
            "shadowstrike",
            "crit",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Assassination";
            pool = ASSASSINATION_POOL;
        } else if [
            "shield",
            "block",
            "parry",
            "guard",
            "wall",
            "defend",
            "bastion",
            "fortress",
            "armor",
            "absorb",
            "barrier",
            "bulk",
            "sturdy",
            "unyielding",
            "iron",
            "stone",
            "hard",
            "tough",
            "heavy",
            "fortify",
            "resist",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Bulwark";
            pool = BULWARK_POOL;
        } else if [
            "skirmish", "dodge", "evade", "swift", "quick", "dash", "roll", "flee", "mobility",
            "jump", "leap", "throw", "toss", "ranger", "scout", "agile", "haste", "speed", "wind",
            "arrow", "bow", "shoot", "distance", "run",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Skirmish";
            pool = SKIRMISH_POOL;
        } else if [
            "tactic",
            "plan",
            "analyze",
            "find",
            "scout",
            "expose",
            "weakness",
            "focus",
            "precision",
            "aim",
            "target",
            "mark",
            "study",
            "prep",
            "exploit",
            "cunning",
            "smart",
            "clever",
            "trap",
            "disarm",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Tactic";
            pool = TACTIC_POOL;
        } else if [
            "command",
            "leader",
            "rally",
            "shout",
            "cry",
            "horn",
            "order",
            "inspire",
            "pet",
            "summon",
            "beast",
            "minion",
            "companion",
            "tame",
            "call",
            "roar",
            "frenzy",
            "pack",
            "ally",
            "allies",
            "wolf",
            "bear",
            "eagle",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Command";
            pool = COMMAND_POOL;
        } else if [
            "martial",
            "strike",
            "attack",
            "slash",
            "cut",
            "pierce",
            "smash",
            "punch",
            "swing",
            "kick",
            "jab",
            "claw",
            "rend",
            "combat",
            "melee",
            "blade",
            "fist",
            "bash",
            "hammer",
            "axe",
            "sword",
            "mace",
            "weapon",
            "fury",
            "rage",
            "warrior",
            "damage",
            "power",
            "hit",
            "cyclone",
            "whirlwind",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Martial";
            pool = MARTIAL_POOL;
        } else if [
            "nature", "bramble", "wild", "thorn", "bloom", "oak", "earth", "growth", "root",
            "leaf", "spore", "ivy", "forest", "vine", "flora", "serpent", "wood", "green", "plant",
            "herb", "season", "spring", "summer", "autumn", "fall",
        ]
        .iter()
        .any(|x| lower.contains(x))
        {
            kind = "Nature";
            pool = NATURE_POOL;
        } else {
            let choices = [
                ("Martial", MARTIAL_POOL),
                ("Bulwark", BULWARK_POOL),
                ("Assassination", ASSASSINATION_POOL),
                ("Skirmish", SKIRMISH_POOL),
                ("Tactic", TACTIC_POOL),
                ("Command", COMMAND_POOL),
                ("Nature", NATURE_POOL),
            ];
            let choice = choices[idx % choices.len()];
            kind = choice.0;
            pool = choice.1;
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
        while seen_perks.contains(&name) {
            let mod_idx = (idx + ctr as usize) % UNIQUE_MODIFIERS.len();
            name = format!("{} {}", cleaned.to_lowercase(), UNIQUE_MODIFIERS[mod_idx]);
            ctr += 1;
        }
        seen_perks.push(name.clone());

        let mut modifiers = Vec::new();
        // 1. Primary attribute (determined based on kind)
        let attr_val = level as i32;
        let attr_name = match kind {
            "Martial" => "BonusStrength",
            "Bulwark" => "BonusConstitution",
            "Assassination" => "BonusDexterity",
            "Skirmish" => "BonusDexterity",
            "Tactic" => "BonusWisdom",
            "Command" => "BonusCharisma",
            "Fire" => "BonusIntelligence",
            "Frost" => "BonusWisdom",
            "Lightning" => "BonusDexterity",
            "Nature" => "BonusConstitution",
            "Holy" => "BonusCharisma",
            "Shadow" => "BonusStrength",
            "Cosmic" => "BonusIntelligence",
            _ => "BonusStrength",
        };
        modifiers.push(format!("{}({})", attr_name, attr_val));

        // 2. Secondary Modifier (ensures uniqueness even at level 1)
        let sec_mod_idx = (idx + level as usize) % 15;
        match sec_mod_idx {
            0 => {
                let val = (level as i32 * 2) + (idx % 3) as i32;
                modifiers.push(format!("BonusAttack({})", val));
            },
            1 => {
                let val = 1.02 + (level as f32 * 0.01) + (idx % 5) as f32 * 0.005;
                modifiers.push(format!("AttackMultiplier({:.3})", val));
            },
            2 => {
                let val = (level as i32 * 2) + (idx % 3) as i32;
                modifiers.push(format!("BonusDefense({})", val));
            },
            3 => {
                let val = 1.02 + (level as f32 * 0.01) + (idx % 5) as f32 * 0.005;
                modifiers.push(format!("DefenseMultiplier({:.3})", val));
            },
            4 => {
                let val = (level as i32 / 3) + 1 + (idx % 2) as i32;
                modifiers.push(format!("BonusInitiative({})", val));
            },
            5 => {
                let val = 1.01 + (level as f32 * 0.005) + (idx % 4) as f32 * 0.002;
                modifiers.push(format!("InitiativeMultiplier({:.3})", val));
            },
            6 => {
                let val = 0.01 + (level as f32 * 0.005) + (idx % 5) as f32 * 0.002;
                modifiers.push(format!("BonusCritChance({:.3})", val));
            },
            7 => {
                let val = 1.01 + (level as f32 * 0.005) + (idx % 4) as f32 * 0.002;
                modifiers.push(format!("CritChanceMultiplier({:.3})", val));
            },
            8 => {
                let val = 0.01 + (level as f32 * 0.005) + (idx % 4) as f32 * 0.002;
                modifiers.push(format!("BonusAttackSpeed({:.3})", val));
            },
            9 => {
                let val = 1.01 + (level as f32 * 0.005) + (idx % 4) as f32 * 0.002;
                modifiers.push(format!("AttackSpeedMultiplier({:.3})", val));
            },
            10 => {
                let val = (level as i32 * 2) + (idx % 3) as i32;
                modifiers.push(format!("BonusPetDamage({})", val));
            },
            11 => {
                let val = 1.02 + (level as f32 * 0.01) + (idx % 5) as f32 * 0.005;
                modifiers.push(format!("PetDamageMultiplier({:.3})", val));
            },
            12 => {
                let val = (level as i32 * 2) + (idx % 3) as i32;
                modifiers.push(format!("BonusDefenseDamage({})", val));
            },
            13 => {
                let val = (level as i32 / 3) + 1 + (idx % 2) as i32;
                modifiers.push(format!("BonusPetInitiative({})", val));
            },
            _ => {
                let val = 1.02 + (level as f32 * 0.01) + (idx % 5) as f32 * 0.005;
                modifiers.push(format!("HealingReceivedMultiplier({:.3})", val));
            },
        }

        let mods_str = modifiers.join(", ");

        let mut effects = Vec::new();
        let mut templates = Vec::new();
        match kind {
            "Fire" => {
                templates.push(format!(
                    "Burn(damage_per_sec: {}, duration: {:.1})",
                    level * 2 + (idx % 3) as u32,
                    5.0 + (idx % 4) as f32
                ));
                templates.push(format!(
                    "FireResistance(percentage: {:.3})",
                    0.10 + (level as f32 * 0.02) + (idx % 5) as f32 * 0.01
                ));
                templates.push(format!(
                    "Empower(damage_bonus_pct: {:.3}, duration: 999999.0)",
                    0.05 + (level as f32 * 0.01)
                ));
            },
            "Frost" => {
                templates.push(format!(
                    "ChillBlast(radius: {:.1}, frost_damage: {})",
                    3.0 + (idx % 3) as f32,
                    level * 3 + (idx % 5) as u32
                ));
                templates.push(format!(
                    "FrostResistance(percentage: {:.3})",
                    0.10 + (level as f32 * 0.02) + (idx % 5) as f32 * 0.01
                ));
                templates.push(format!(
                    "Freeze(attack_speed_reduction: {:.3}, duration: {:.1})",
                    0.10 + (level as f32 * 0.01),
                    3.0 + (idx % 3) as f32
                ));
                templates.push(format!(
                    "Fortify(armor_bonus_pct: {:.3}, duration: 999999.0)",
                    0.05 + (level as f32 * 0.01)
                ));
            },
            "Lightning" => {
                templates.push(format!(
                    "ChainArc(jumps: {}, damage_decay_pct: {:.2})",
                    2 + (idx % 3) as u32,
                    0.10 + (idx % 4) as f32 * 0.05
                ));
                templates.push(format!(
                    "LightningResistance(percentage: {:.3})",
                    0.10 + (level as f32 * 0.02) + (idx % 5) as f32 * 0.01
                ));
                templates.push(format!(
                    "Haste(initiative_bonus: {:.3}, duration: 999999.0)",
                    0.05 + (level as f32 * 0.01)
                ));
            },
            "Nature" => {
                templates.push(format!(
                    "Regen(heal_per_sec: {}, duration: 999999.0)",
                    level * 2 + (idx % 4) as u32
                ));
                templates.push(format!(
                    "Poison(damage_per_sec: {}, duration: {:.1})",
                    level * 2 + (idx % 3) as u32,
                    6.0 + (idx % 3) as f32
                ));
                templates.push(format!(
                    "PoisonResistance(percentage: {:.3})",
                    0.10 + (level as f32 * 0.02) + (idx % 5) as f32 * 0.01
                ));
                templates.push(format!(
                    "Thorns(damage_reflected: {}, duration: 999999.0)",
                    level * 3 + (idx % 5) as u32
                ));
            },
            "Holy" => {
                templates.push(format!("Clearcasting(duration: 999999.0)"));
                templates.push(format!(
                    "HolyResistance(percentage: {:.3})",
                    0.10 + (level as f32 * 0.02) + (idx % 5) as f32 * 0.01
                ));
                templates.push(format!(
                    "Regen(heal_per_sec: {}, duration: 999999.0)",
                    level + (idx % 3) as u32
                ));
            },
            "Shadow" => {
                templates
                    .push(format!("Lifesteal(percentage: {:.3})", 0.05 + (level as f32 * 0.01)));
                templates.push(format!(
                    "ShadowResistance(percentage: {:.3})",
                    0.10 + (level as f32 * 0.02) + (idx % 5) as f32 * 0.01
                ));
                templates.push(format!(
                    "DoomCurse(stacks_required: 3, explosion_damage: {})",
                    level * 10 + (idx % 20) as u32
                ));
            },
            "Cosmic" => {
                templates.push(format!(
                    "TimeWarp(initiative_reduction: {:.3}, duration: {:.1})",
                    0.05 + (level as f32 * 0.005),
                    5.0 + (idx % 3) as f32
                ));
                templates
                    .push(format!("MonarchShield(duration: {:.1})", 2.0 + (level as f32 * 0.2)));
            },
            "Martial" => {
                templates.push(format!(
                    "Bleed(damage_per_sec: {}, duration: {:.1})",
                    level * 2 + (idx % 3) as u32,
                    5.0 + (idx % 4) as f32
                ));
                templates.push(format!(
                    "BleedResistance(percentage: {:.3})",
                    0.10 + (level as f32 * 0.02) + (idx % 5) as f32 * 0.01
                ));
                templates.push(format!(
                    "Weaken(attack_power_reduction: {}, duration: {:.1})",
                    level * 2 + (idx % 5) as u32,
                    5.0 + (idx % 3) as f32
                ));
            },
            "Bulwark" => {
                templates.push(format!(
                    "Fortify(armor_bonus_pct: {:.3}, duration: 999999.0)",
                    0.05 + (level as f32 * 0.01)
                ));
                templates.push(format!(
                    "Thorns(damage_reflected: {}, duration: 999999.0)",
                    level * 2 + (idx % 4) as u32
                ));
                templates.push(format!(
                    "ArmorShred(reduction: {}, duration: {:.1})",
                    level * 2 + (idx % 5) as u32,
                    6.0 + (idx % 3) as f32
                ));
            },
            "Assassination" => {
                templates.push(format!(
                    "Poison(damage_per_sec: {}, duration: {:.1})",
                    level * 2 + (idx % 4) as u32,
                    5.0 + (idx % 3) as f32
                ));
                templates.push(format!(
                    "Bleed(damage_per_sec: {}, duration: {:.1})",
                    level * 2 + (idx % 3) as u32,
                    5.0 + (idx % 4) as f32
                ));
                templates
                    .push(format!("Lifesteal(percentage: {:.3})", 0.02 + (level as f32 * 0.005)));
            },
            "Skirmish" => {
                templates.push(format!(
                    "Haste(initiative_bonus: {:.3}, duration: 999999.0)",
                    0.05 + (level as f32 * 0.01)
                ));
                templates.push(format!(
                    "Blind(miss_chance: {:.2}, duration: {:.1})",
                    0.10 + (level as f32 * 0.01),
                    4.0 + (idx % 3) as f32
                ));
            },
            "Tactic" => {
                templates.push(format!(
                    "Vulnerability(damage_taken_multiplier: {:.3}, duration: {:.1})",
                    1.05 + (level as f32 * 0.01),
                    6.0 + (idx % 3) as f32
                ));
                templates.push(format!(
                    "Focus(crit_chance_bonus: {:.3}, duration: 999999.0)",
                    0.05 + (level as f32 * 0.01)
                ));
            },
            "Command" => {
                templates.push(format!(
                    "BeastFrenzy(attack_speed_bonus: {:.3}, duration: {:.1})",
                    0.10 + (level as f32 * 0.01),
                    8.0 + (idx % 5) as f32
                ));
                templates.push(format!("Taunt(duration: {:.1})", 5.0 + (idx % 4) as f32));
            },
            _ => {
                templates.push(format!("Clearcasting(duration: 999999.0)"));
            },
        }

        // Determine number of effects based on level
        let num_effects = if level <= 4 {
            1
        } else if level <= 12 {
            2
        } else {
            3
        };

        for i in 0..num_effects {
            if templates.is_empty() {
                break;
            }
            let template_idx = (idx + i as usize) % templates.len();
            effects.push(templates[template_idx].clone());
        }

        let effects_str = effects.join(", ");

        perks_ron.push_str(&format!(
            "    (
        name: \"{}\",
        image: \"images/inventory/perks/{}\",
        kind: {},
        level: {},
        modifiers: [{}],
        effects: [{}],
    ),
",
            name, filename, kind, level, mods_str, effects_str
        ));
    }
    perks_ron.push_str("]\n");
    let mut file = File::create("assets/inventory/perks.ron").unwrap();
    file.write_all(perks_ron.as_bytes()).unwrap();
    println!("Generated {} perks in perks.ron", total_pks);

    // 3. WEAPONS
    let weapons_dir = "assets/images/inventory/equipment/weapon";
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

        let mut kind = "Martial";
        let mut category = "Melee";
        if ["dagger", "assassin", "poison"].iter().any(|x| lower.contains(x)) {
            kind = "Assassination";
        } else if ["shield"].iter().any(|x| lower.contains(x)) {
            category = "Shield";
            kind = "Bulwark";
        } else if ["bulwark", "defend"].iter().any(|x| lower.contains(x)) {
            kind = "Bulwark";
        } else if ["staff", "wand", "scroll", "book"].iter().any(|x| lower.contains(x)) {
            kind = "Tactic";
            category = "Magic";
        } else if ["bow", "crossbow", "sling"].iter().any(|x| lower.contains(x)) {
            kind = "Skirmish";
            category = "Range";
        }

        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Steel Weapon".to_string();
        }

        let adj = LEVEL_ADJECTIVES[level as usize - 1];
        let mut name = format!("{} {}", adj, cleaned).to_lowercase();
        let mut ctr = 1;
        while seen_weapons.contains(&name) {
            let mod_idx = (idx + ctr as usize) % UNIQUE_MODIFIERS.len();
            name = format!("{} {} {}", adj, cleaned, UNIQUE_MODIFIERS[mod_idx]).to_lowercase();
            ctr += 1;
        }
        seen_weapons.push(name.clone());

        let price = 10 + level * level * 25 + (idx % 10) as u32 * 15 + (idx % 3) as u32 * 3;
        let base_attack = 5 + level * 4 + (idx % 5) as u32;
        let attack_speed = 1.0 + level as f32 * 0.05 + (idx % 4) as f32 * 0.02;
        let crit_chance = 0.05 + level as f32 * 0.01 + (idx % 5) as f32 * 0.005;

        let mut modifiers = Vec::new();
        match kind {
            "Assassination" => {
                modifiers.push(format!("BonusDexterity({})", level as i32));
                if level >= 5 {
                    modifiers.push(format!("BonusCritChance({:.2})", (idx % 5 + 1) as f32 * 0.01));
                }
                if level >= 10 {
                    modifiers
                        .push(format!("BonusAttack({})", (idx % 3 + 1) as i32 * level as i32 / 2));
                }
            },
            "Bulwark" => {
                modifiers.push(format!("BonusConstitution({})", level as i32));
                if level >= 5 {
                    modifiers.push(format!(
                        "BonusDefense({})",
                        (idx % 3 + 1) as i32 * level as i32 / 4 + 1
                    ));
                }
                if level >= 10 {
                    modifiers.push(format!("BonusStrength({})", (idx % 3 + 1) as i32));
                }
            },
            "Tactic" => {
                modifiers.push(format!("BonusIntelligence({})", level as i32));
                if level >= 5 {
                    modifiers.push(format!("BonusWisdom({})", (idx % 3 + 1) as i32));
                }
                if level >= 10 {
                    modifiers.push(format!("BonusInitiative({})", (idx % 4 + 1) as i32));
                }
            },
            _ => {
                modifiers.push(format!("BonusStrength({})", level as i32));
                if level >= 5 {
                    modifiers.push(format!("BonusAttackSpeed({:.2})", (idx % 5 + 1) as f32 * 0.01));
                }
                if level >= 10 {
                    modifiers.push(format!("BonusDexterity({})", (idx % 3 + 1) as i32));
                }
            },
        }
        let mods_str = modifiers.join(", ");

        let mut effects = Vec::new();
        if level >= 8 {
            match kind {
                "Assassination" => {
                    if idx % 2 == 0 {
                        effects.push("Blind(miss_chance: 0.25, duration: 3.0)".to_string());
                    } else {
                        effects.push("Lifesteal(percentage: 0.08)".to_string());
                    }
                },
                "Bulwark" => {
                    if idx % 2 == 0 {
                        effects.push("Thorns(damage_reflected: 10, duration: 4.0)".to_string());
                    } else {
                        effects.push("Fortify(armor_bonus_pct: 0.15, duration: 5.0)".to_string());
                    }
                },
                "Tactic" => {
                    if idx % 2 == 0 {
                        effects.push("Clearcasting(duration: 4.0)".to_string());
                    } else {
                        effects.push("ManaBurn(amount: 15)".to_string());
                    }
                },
                "Skirmish" => {
                    if idx % 2 == 0 {
                        effects.push("Stun(duration: 1.5)".to_string());
                    } else {
                        effects.push("Focus(crit_chance_bonus: 0.20, duration: 4.0)".to_string());
                    }
                },
                _ => {
                    if idx % 2 == 0 {
                        effects.push("Bleed(damage_per_sec: 8, duration: 4.0)".to_string());
                    } else {
                        effects.push("Cleave(radius: 2.5, damage_pct: 0.40)".to_string());
                    }
                },
            }
        }
        let effects_str = effects.join(", ");

        weapons_ron.push_str(&format!(
            "    (
        name: \"{}\",
        image: \"images/inventory/equipment/weapon/{}\",
        kind: {},
        category: {},
        hand: {},
        level: {},
        price: {},
        base_attack: {},
        attack_speed: {:.2},
        crit_chance: {:.2},
        modifiers: [{}],
        effects: [{}],
    ),
",
            name,
            filename,
            kind,
            category,
            hand,
            level,
            price,
            base_attack,
            attack_speed,
            crit_chance,
            mods_str,
            effects_str
        ));
    }
    weapons_ron.push_str("]\n");
    let mut file = File::create("assets/inventory/weapons.ron").unwrap();
    file.write_all(weapons_ron.as_bytes()).unwrap();
    println!("Generated {} weapons in weapons.ron", total_wps);

    // 4. ARMOR
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
        let dir_path = format!("assets/images/inventory/equipment/{}", folder);
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
        let mut kind = "Bulwark";
        if ["leather", "scout", "assassin", "ranger"].iter().any(|x| lower.contains(x)) {
            kind = "Assassination";
        } else if ["silk", "robe", "mage", "cloth", "wizard", "priest"]
            .iter()
            .any(|x| lower.contains(x))
        {
            kind = "Tactic";
        }

        let mut cleaned = clean_name(filename);
        if cleaned.is_empty() {
            cleaned = "Armor".to_string();
        }

        let adj = LEVEL_ADJECTIVES[level as usize - 1];
        let mut name = format!("{} {}", adj, cleaned).to_lowercase();
        let mut ctr = 1;
        while seen_armor.contains(&name) {
            let mod_idx = (idx + ctr as usize) % UNIQUE_MODIFIERS.len();
            name = format!("{} {} {}", adj, cleaned, UNIQUE_MODIFIERS[mod_idx]).to_lowercase();
            ctr += 1;
        }
        seen_armor.push(name.clone());

        let price = 10 + level * level * 20 + (idx % 10) as u32 * 12 + (idx % 3) as u32 * 2;
        let base_defense = 2 + level * 3 + (idx % 4) as u32;

        let mut modifiers = Vec::new();
        match kind {
            "Assassination" => {
                modifiers.push(format!("BonusDexterity({})", level as i32));
                if level >= 5 {
                    modifiers.push(format!("BonusInitiative({})", (idx % 3 + 1) as i32));
                }
                if level >= 10 {
                    modifiers.push(format!("BonusCritChance({:.2})", (idx % 4 + 1) as f32 * 0.01));
                }
            },
            "Bulwark" => {
                modifiers.push(format!("BonusConstitution({})", level as i32));
                if level >= 5 {
                    modifiers.push(format!(
                        "BonusDefense({})",
                        (idx % 3 + 1) as i32 * level as i32 / 5 + 1
                    ));
                }
                if level >= 10 {
                    modifiers.push(format!("BonusStrength({})", (idx % 3 + 1) as i32));
                }
            },
            "Tactic" => {
                modifiers.push(format!("BonusIntelligence({})", level as i32));
                if level >= 5 {
                    modifiers.push(format!("BonusWisdom({})", (idx % 3 + 1) as i32));
                }
                if level >= 10 {
                    modifiers.push(format!("BonusCharisma({})", (idx % 3 + 1) as i32));
                }
            },
            _ => {
                modifiers.push(format!("BonusStrength({})", level as i32));
                if level >= 5 {
                    modifiers.push(format!("BonusConstitution({})", (idx % 3 + 1) as i32));
                }
                if level >= 10 {
                    modifiers.push(format!("BonusDefense({})", (idx % 3 + 1) as i32));
                }
            },
        }
        let mods_str = modifiers.join(", ");

        let mut effects = Vec::new();
        if level >= 8 {
            match kind {
                "Assassination" => {
                    if idx % 2 == 0 {
                        effects.push("Blind(miss_chance: 0.15, duration: 3.0)".to_string());
                    } else {
                        effects.push("Lifesteal(percentage: 0.05)".to_string());
                    }
                },
                "Bulwark" => {
                    if idx % 2 == 0 {
                        effects.push("Thorns(damage_reflected: 6, duration: 4.0)".to_string());
                    } else {
                        effects.push("Fortify(armor_bonus_pct: 0.10, duration: 5.0)".to_string());
                    }
                },
                _ => {
                    if idx % 2 == 0 {
                        effects.push("Regen(heal_per_sec: 4, duration: 6.0)".to_string());
                    } else {
                        effects.push("Clearcasting(duration: 3.0)".to_string());
                    }
                },
            }
        }
        let effects_str = effects.join(", ");

        armor_ron.push_str(&format!(
            "    (
        name: \"{}\",
        image: \"images/inventory/equipment/{}/{}\",
        kind: {},
        level: {},
        price: {},
        slot: {},
        base_defense: {},
        modifiers: [{}],
        effects: [{}],
    ),
",
            name, folder, filename, kind, level, price, slot, base_defense, mods_str, effects_str
        ));
    }
    armor_ron.push_str("]\n");
    let mut file = File::create("assets/inventory/armor.ron").unwrap();
    file.write_all(armor_ron.as_bytes()).unwrap();
    println!("Generated {} armor pieces in armor.ron", total_arm);
}
