const fs = require('fs');
const path = require('path');

// Helper to capitalize first letter
function capitalize(str) {
    if (!str) return '';
    return str.charAt(0).toUpperCase() + str.slice(1);
}

// Recursively get all png files from a directory, relative to assets folder
function getPngFiles(dir, baseDir, files = [], isWeaponDir = false) {
    const fileList = fs.readdirSync(dir);
    for (const file of fileList) {
        const fullPath = path.join(dir, file);
        if (fs.statSync(fullPath).isDirectory()) {
            getPngFiles(fullPath, baseDir, files, isWeaponDir);
        } else {
            if (file.toLowerCase().endsWith('.png')) {
                if (isWeaponDir) {
                    const l = file.toLowerCase();
                    // FILTER OUT ARROWS, QUIVERS AND BULLETS/BOLTS
                    if (l.includes('arrow') || l.includes('quiver') || l.includes('bullet') || l.includes('bolt')) {
                        continue;
                    }
                }
                // Get path relative to baseDir (e.g. assets)
                const rel = path.relative(baseDir, fullPath).replace(/\\/g, '/');
                files.push(rel);
            }
        }
    }
    return files;
}

const assetsDir = path.join(__dirname, 'assets');
const allIconsDir = path.join(assetsDir, 'images', 'all_icons');

if (!fs.existsSync(allIconsDir)) {
    console.error(`Error: 'all_icons' directory not found at ${allIconsDir}`);
    process.exit(1);
}

// Categorize all icons source paths
const weaponIconsDir = path.join(allIconsDir, 'WeaponIcons');
const skillsIconsDir = path.join(allIconsDir, 'SkillsIcons');
const accessoryIconsDir = path.join(allIconsDir, 'ArmorIcons', 'RingAndNeck_Icons');

const weapon_sources = fs.existsSync(weaponIconsDir) ? getPngFiles(weaponIconsDir, assetsDir, [], true) : [];
const ability_sources = fs.existsSync(skillsIconsDir) ? getPngFiles(skillsIconsDir, assetsDir, [], false) : [];
const accessory_sources = fs.existsSync(accessoryIconsDir) ? getPngFiles(accessoryIconsDir, assetsDir, [], false) : [];

// Perk sources will be everything under all_icons except WeaponIcons, SkillsIcons and RingAndNeck_Icons
const perk_sources = [];
const allDirs = fs.readdirSync(allIconsDir);
for (const d of allDirs) {
    if (d !== 'WeaponIcons' && d !== 'SkillsIcons') {
        const fullPath = path.join(allIconsDir, d);
        if (fs.statSync(fullPath).isDirectory()) {
            // Include everything except RingAndNeck_Icons folder inside ArmorIcons
            if (d === 'ArmorIcons') {
                const subdirs = fs.readdirSync(fullPath);
                for (const sd of subdirs) {
                    if (sd !== 'RingAndNeck_Icons') {
                        const fp = path.join(fullPath, sd);
                        if (fs.statSync(fp).isDirectory()) {
                            perk_sources.push(...getPngFiles(fp, assetsDir, [], false));
                        }
                    }
                }
            } else {
                perk_sources.push(...getPngFiles(fullPath, assetsDir, [], false));
            }
        }
    }
}

// Group all files into categories
const all_png_sources = getPngFiles(allIconsDir, assetsDir, [], false);
const category_sources = {
    helmet: [],
    armor: [],
    boots: [],
    one_hand_weapon: [],
    two_hand_weapon: [],
    offhand: [],
    consumable: []
};

for (const src of all_png_sources) {
    const fileName = path.basename(src).toLowerCase();
    if (fileName.startsWith('helm_') || fileName.startsWith('helms')) {
        category_sources.helmet.push(src);
    } else if (fileName.startsWith('chest_') || fileName === 'cuirass.png') {
        category_sources.armor.push(src);
    } else if (fileName.startsWith('boots_') || fileName === 'bootss.png') {
        category_sources.boots.push(src);
    } else if (fileName.startsWith('shield_') || fileName.startsWith('shields') || fileName.startsWith('book_')) {
        category_sources.offhand.push(src);
    } else if (fileName.startsWith('potion_')) {
        category_sources.consumable.push(src);
    } else if (fileName.startsWith('dagger_') || fileName.startsWith('sword_') || fileName.startsWith('axe_') || fileName.startsWith('hammer_')) {
        category_sources.one_hand_weapon.push(src);
    } else if (fileName.startsWith('bow_') || fileName.startsWith('crossbow_') || fileName.startsWith('staff_') || fileName.startsWith('spear_') || fileName.startsWith('scythe_')) {
        category_sources.two_hand_weapon.push(src);
    }
}

// Fallbacks
if (category_sources.helmet.length === 0) category_sources.helmet = weapon_sources;
if (category_sources.armor.length === 0) category_sources.armor = weapon_sources;
if (category_sources.boots.length === 0) category_sources.boots = weapon_sources;
if (category_sources.offhand.length === 0) category_sources.offhand = weapon_sources;
if (category_sources.consumable.length === 0) category_sources.consumable = weapon_sources;
if (category_sources.one_hand_weapon.length === 0) category_sources.one_hand_weapon = weapon_sources;
if (category_sources.two_hand_weapon.length === 0) category_sources.two_hand_weapon = weapon_sources;

console.log(`Loaded sources:
 - Weapons: ${weapon_sources.length} sources
 - Abilities: ${ability_sources.length} sources
 - Accessories: ${accessory_sources.length} sources
 - Perks: ${perk_sources.length} sources
 - Categorized Helmets: ${category_sources.helmet.length}
 - Categorized Armors: ${category_sources.armor.length}
 - Categorized Boots: ${category_sources.boots.length}
 - Categorized Offhands: ${category_sources.offhand.length}
 - Categorized Consumables: ${category_sources.consumable.length}
 - Categorized 1H Weapons: ${category_sources.one_hand_weapon.length}
 - Categorized 2H Weapons: ${category_sources.two_hand_weapon.length}`);

if (weapon_sources.length === 0 || ability_sources.length === 0 || perk_sources.length === 0 || accessory_sources.length === 0) {
    console.error('Error: Insufficient sources to generate icons!');
    process.exit(1);
}

// Clean up existing generated files in assets/images/equipment, abilities, perks
function cleanGeneratedImages(dir, prefix) {
    if (!fs.existsSync(dir)) return;
    const files = fs.readdirSync(dir);
    let removed = 0;
    for (const f of files) {
        if (f.startsWith(prefix)) {
            try {
                fs.unlinkSync(path.join(dir, f));
                removed++;
            } catch (e) {
                // Ignore errors
            }
        }
    }
    console.log(`Cleaned up ${removed} matching files in ${dir}`);
}

const destEquipmentDir = path.join(assetsDir, 'images', 'equipment');
const destAbilitiesDir = path.join(assetsDir, 'images', 'abilities');
const destPerksDir = path.join(assetsDir, 'images', 'perks');

const subfolders = ['helmet', 'armor', 'boots', 'weapon', 'accessory', 'consumable'];
for (const sub of subfolders) {
    fs.mkdirSync(path.join(destEquipmentDir, sub), { recursive: true });
}
fs.mkdirSync(destAbilitiesDir, { recursive: true });
fs.mkdirSync(destPerksDir, { recursive: true });

// Constants for generation
const classes = ['warrior', 'mage', 'rogue', 'druid'];
const kinds = ['helmet', 'armor', 'boots', 'one_hand_weapon', 'two_hand_weapon', 'offhand'];

const abilityNamesAdjs = {
    warrior: ["Cleaving", "Savage", "Furious", "Shield", "Mighty", "Ironclad", "Heavy", "Devastating", "Relentless", "Colossal"],
    mage: ["Arcane", "Pyro", "Frost", "Static", "Spell", "Mana", "Gravity", "Lightning", "Chilling", "Cosmic"],
    rogue: ["Shadow", "Silent", "Devious", "Toxic", "Vanish", "Swift", "Phantom", "Precision", "Cunning", "Venomous"],
    druid: ["Verdant", "Gale", "Root", "Sunfire", "Forest", "Wild", "Bear", "Hurricane", "Vine", "Thorn"]
};

const abilityNamesNouns = {
    warrior: ["Strike", "Leap", "Smash", "Slam", "Overpower", "Whirlwind", "Charge", "Block", "Pummel", "Rend"],
    mage: ["Bolt", "Blast", "Shard", "Nova", "Beam", "Barrier", "Storm", "Touch", "Singularity", "Flash"],
    rogue: ["Thrust", "Dart", "Slash", "Vanish", "Assault", "Pierce", "Cut", "Gambit", "Toxin", "Stab"],
    druid: ["Mend", "Grasp", "Wolf", "Howl", "Armor", "Burst", "Wrath", "Embrace", "Roar", "Twister"]
};

const perkNamesClasses = {
    warrior: ["Vanguard", "Executioner", "Bastion", "Battlerage", "Juggernaut", "Centurion", "Gladiator", "Overlord", "Conqueror", "Titan"],
    mage: ["Archmage", "Acolyte", "Evoker", "Spellcraft", "Sage", "Pyromancy", "Cryomancy", "Aether", "Scribe", "Cosmologist"],
    rogue: ["Dreadblade", "Infiltrator", "Stalker", "Duellist", "Scoundrel", "Cutthroat", "Spectre", "Trickster", "Acrobat", "Shinobi"],
    druid: ["Beastmaster", "Wildshaper", "Stormcaller", "Dryad", "Primalist", "Animist", "Woodsman", "Earthkeeper", "Ancient", "Grovewarden"]
};

const perkSubWords = [
    "Mastery", "Resilience", "Reflexes", "Harmony", "Might",
    "Affinity", "Fortitude", "Insight", "Purity", "Vigor",
    "Wisdom", "Agility", "Bravery", "Sentry", "Vigilance",
    "Aura", "Guile", "Tenacity", "Luck", "Fortification"
];

const accessoryWords = ["Ring", "Bracelet", "Collar", "Necklace", "Amulet", "Choker", "Bangle", "Talisman", "Band", "Medallion"];

const magicTypes = ["Physical", "Fire", "Ice", "Dark", "Nature", "Holy"];

const perkThemes = [
    "defense", "mana", "speed", "nature", "offense", "magic",
    "stealth", "animals", "vitality", "wealth", "survival", "precision"
];

const levelAdjectives = [
    "Novice", "Initiate", "Apprentice", "Journeyman", "Adept",
    "Expert", "Elite", "Veteran", "Master", "Grandmaster",
    "Sovereign", "Exalted", "Gladiator", "Heroic", "Champion",
    "Overlord", "Epic", "Eldritch", "Mythic", "Legendary"
];

const qualityAdjectives = [
    "Swift", "Infused", "Clandestine", "Sovereign", "Hallowed",
    "Vile", "Exalted", "Primal", "Dread", "Ethereal",
    "Radiant", "Grave", "Savage", "Noble", "Gargoyle",
    "Wild", "Mystic", "Ashen", "Blood", "Runic",
    "Void", "Volcanic", "Elder", "Stormy", "Ancient",
    "Enchanted", "Abyssal", "Astral", "Zealous", "Haunted"
];

const finalEquipment = [];
const finalAbilities = [];
const finalPerks = [];

const totalLevels = 20;
const itemsPerLevel = 30;
const accessoriesPerLevel = 10;

// Helper to get subfolder of an equipment kind
function getSubfolderOfKind(kind) {
    if (kind === 'one_hand_weapon' || kind === 'two_hand_weapon' || kind === 'offhand') {
        return 'weapon';
    }
    return kind; // 'helmet', 'armor', 'boots', 'accessory', 'consumable'
}

// Completely wipe out previous folders' generated files to ensure a clean slate!
for (const adj of levelAdjectives) {
    for (const sub of subfolders) {
        cleanGeneratedImages(path.join(destEquipmentDir, sub), adj.toLowerCase());
    }
    cleanGeneratedImages(destAbilitiesDir, adj.toLowerCase());
    cleanGeneratedImages(destPerksDir, adj.toLowerCase());
}

function getItemWord(kind_eq, src_eq, index) {
    const fileName = path.basename(src_eq).toLowerCase();

    if (kind_eq === 'helmet') {
        const options = ["Helm", "Helmet", "Cap", "Visor", "Crown", "Hood"];
        return options[index % options.length];
    }
    if (kind_eq === 'armor') {
        const options = ["Plates", "Chestplate", "Robes", "Mail", "Tunic", "Cuirass"];
        return options[index % options.length];
    }
    if (kind_eq === 'boots') {
        const options = ["Boots", "Greaves", "Shoes", "Slippers", "Soles", "Treads"];
        return options[index % options.length];
    }
    if (kind_eq === 'consumable') {
        const options = ["Potion", "Elixir", "Vial", "Scroll", "Flask", "Tonic"];
        return options[index % options.length];
    }
    if (kind_eq === 'offhand') {
        if (fileName.includes('book')) {
            const options = ["Tome", "Grimoire", "Book", "Scroll", "Tome of Fire", "Tome of Ice"];
            return options[index % options.length];
        } else {
            const options = ["Shield", "Buckler", "Aegis", "Bulwark", "Kite Shield", "Greatshield"];
            return options[index % options.length];
        }
    }
    if (kind_eq === 'one_hand_weapon') {
        if (fileName.includes('dagger')) {
            const options = ["Dagger", "Knife", "Dirk", "Stiletto", "Striker"];
            return options[index % options.length];
        }
        if (fileName.includes('sword')) {
            const options = ["Sword", "Blade", "Saber", "Rapier", "Cutlass"];
            return options[index % options.length];
        }
        if (fileName.includes('axe')) {
            const options = ["Axe", "Hatchet", "Cleaver", "Handaxe"];
            return options[index % options.length];
        }
        if (fileName.includes('hammer')) {
            const options = ["Hammer", "Mace", "Warhammer", "Flail", "Club"];
            return options[index % options.length];
        }
        const options = ["Blade", "Sword", "Dagger", "Saber", "Axe", "Mace", "Hammer"];
        return options[index % options.length];
    }
    if (kind_eq === 'two_hand_weapon') {
        if (fileName.includes('bow') || fileName.includes('crossbow')) {
            const options = ["Bow", "Longbow", "Shortbow", "Crossbow", "Recurve Bow"];
            return options[index % options.length];
        }
        if (fileName.includes('staff')) {
            const options = ["Staff", "Greatstaff", "Crook", "Spire"];
            return options[index % options.length];
        }
        if (fileName.includes('spear')) {
            const options = ["Spear", "Halberd", "Pike", "Lance", "Trident"];
            return options[index % options.length];
        }
        if (fileName.includes('scythe')) {
            const options = ["Scythe", "Reaper", "Harvester"];
            return options[index % options.length];
        }
        const options = ["Staff", "Greatsword", "Warhammer", "Bow", "Halberd", "Scythe"];
        return options[index % options.length];
    }
    return "Item";
}

function getAccessoryWord(src_acc, index) {
    const fileName = path.basename(src_acc).toLowerCase();
    if (fileName.includes('ring')) {
        const options = ["Ring", "Band", "Signet", "Loop"];
        return options[index % options.length];
    }
    if (fileName.includes('necklace') || fileName.includes('neck_')) {
        const options = ["Necklace", "Amulet", "Choker", "Collar", "Talisman", "Medallion"];
        return options[index % options.length];
    }
    if (fileName.includes('bracelet')) {
        const options = ["Bracelet", "Bangle", "Bracer", "Cuff"];
        return options[index % options.length];
    }
    const options = ["Ring", "Bracelet", "Collar", "Necklace", "Amulet", "Choker", "Bangle", "Talisman", "Band", "Medallion"];
    return options[index % options.length];
}

for (let lvl = 1; lvl <= totalLevels; lvl++) {
    const levelStr = lvl.toString().padStart(2, '0');
    const levelAdj = levelAdjectives[lvl - 1];

    // 30 base equipment pieces
    for (let j = 0; j < itemsPerLevel; j++) {
        const overallIndex = (lvl - 1) * itemsPerLevel + j;

        const class_hint_eq = classes[j % classes.length];
        const kind_eq = kinds[j % kinds.length];

        // UNIQUE COMBINATIONS
        const mats = ["Bronze", "Copper", "Simple", "Worn"];
        const mat = mats[j % mats.length];

        const src_eq = category_sources[kind_eq][overallIndex % category_sources[kind_eq].length];
        const itemWord = getItemWord(kind_eq, src_eq, j);

        const classAdjectives = {
            warrior: ["Vanguard", "Warmonger", "Guardian", "Bulwark", "Mighty", "Challenger"],
            mage: ["Arcane", "Pyromancer", "Spellweaver", "Sorcerer", "Evoker", "Acolyte"],
            rogue: ["Assassin", "Shadow", "Silent", "Phantom", "Infiltrator", "Stalker"],
            druid: ["Barkskin", "Wildheart", "Forest", "Primal", "Nature", "Verdant"]
        };
        const classAdjs = classAdjectives[class_hint_eq];
        const classAdj = classAdjs[j % classAdjs.length];

        const name_eq = `${levelAdj} ${mat} ${classAdj} ${itemWord}`.toLowerCase();

        let attack = 0;
        let armor = 0;
        let crit = 0;
        let initiative = j % 5 - 1;
        let attack_speed = 0.0;

        if (kind_eq === 'one_hand_weapon') {
            attack = Math.floor(lvl * 1.0) + 2;
            crit = 2;
            attack_speed = 0.9 + (j % 5) * 0.1;
        } else if (kind_eq === 'two_hand_weapon') {
            attack = Math.floor(lvl * 1.2) + 3;
            crit = 4;
            attack_speed = 0.7 + (j % 5) * 0.1;
        }

        if (['helmet', 'armor', 'boots', 'offhand'].includes(kind_eq)) {
            armor = Math.floor(lvl * 0.51) + 1;
        }

        if (kind_eq === 'boots') {
            initiative = 2;
        }

        const price_eq = lvl * 30 + 10 + (j % 5) * 5;
        const subfolder_eq = getSubfolderOfKind(kind_eq);

        finalEquipment.push({
            name: name_eq,
            level: lvl,
            class_hint: class_hint_eq,
            icon_path: `images/equipment/${subfolder_eq}/${name_eq}.png`,
            kind: kind_eq,
            price: price_eq,
            stats: {
                attack,
                armor,
                crit,
                initiative,
                attack_speed: parseFloat(attack_speed.toFixed(2))
            }
        });

        fs.copyFileSync(path.join(assetsDir, src_eq), path.join(destEquipmentDir, subfolder_eq, `${name_eq}.png`));
    }

    // 10 accessories per level
    for (let a = 0; a < accessoriesPerLevel; a++) {
        const overallIndex = (lvl - 1) * accessoriesPerLevel + a;

        const class_hint = classes[a % classes.length];
        const mats = ["Copper", "Bronze", "Pewter", "Silver", "Gold", "Platinum"];
        const mat = mats[a % mats.length];
        const src_acc = accessory_sources[overallIndex % accessory_sources.length];
        const word = getAccessoryWord(src_acc, a);

        const classPrefixes = {
            warrior: ["Valor", "Might", "Fortitude", "Brutality"],
            mage: ["Arcana", "Focus", "Intelligence", "Evocation"],
            rogue: ["Cunning", "Velocity", "Deception", "Shadows"],
            druid: ["Thorns", "Wilds", "Restoration", "Earth"]
        };
        const classAdjs = classPrefixes[class_hint];
        const classAdj = classAdjs[a % classAdjs.length];

        const name_acc = `${levelAdj} ${mat} ${word} of ${classAdj}`.toLowerCase();

        const attack = Math.floor(lvl * 0.41);
        const armor = Math.floor(lvl * 0.41);
        const crit = (a % 3) * (lvl >= 5 ? 2 : 1);
        const initiative = (a % 5) - 2;
        const price_acc = lvl * 20 + 5 + (a % 5) * 5;
        const subfolder_acc = getSubfolderOfKind("accessory");

        finalEquipment.push({
            name: name_acc,
            level: lvl,
            class_hint,
            icon_path: `images/equipment/${subfolder_acc}/${name_acc}.png`,
            kind: "accessory",
            price: price_acc,
            stats: {
                attack,
                armor,
                crit,
                initiative,
                attack_speed: 0.0
            }
        });

        fs.copyFileSync(path.join(assetsDir, src_acc), path.join(destEquipmentDir, subfolder_acc, `${name_acc}.png`));
    }

    // 10 consumables per level
    const consumableTypes = ["Health Potion", "Mana Potion", "Strength Potion", "Dexterity Potion", "Constitution Potion", "Intelligence Potion", "Wisdom Potion", "Charisma Potion", "Rejuvenation Elixir", "Antidote Vial"];
    for (let c = 0; c < 10; c++) {
        const overallIndex = (lvl - 1) * 10 + c;
        const type = consumableTypes[c % consumableTypes.length];
        const name_con = `${levelAdj} ${type}`.toLowerCase();

        const src_con = category_sources.consumable[overallIndex % category_sources.consumable.length];
        const subfolder_con = getSubfolderOfKind("consumable");

        finalEquipment.push({
            name: name_con,
            level: lvl,
            class_hint: "none",
            icon_path: `images/equipment/${subfolder_con}/${name_con}.png`,
            kind: "consumable",
            price: lvl * 15 + 5,
            stats: {
                attack: 0,
                armor: 0,
                crit: 0,
                initiative: 0,
                attack_speed: 0.0
            }
        });

        fs.copyFileSync(path.join(assetsDir, src_con), path.join(destEquipmentDir, subfolder_con, `${name_con}.png`));
    }

    // 30 abilities per level
    for (let j = 0; j < itemsPerLevel; j++) {
        const overallIndex = (lvl - 1) * itemsPerLevel + j;

        const class_hint_ab = classes[j % classes.length];
        const listADesc = abilityNamesAdjs[class_hint_ab];
        const listNDesc = abilityNamesNouns[class_hint_ab];
        const adj = listADesc[j % listADesc.length];
        const noun = listNDesc[j % listNDesc.length];
        const qual = qualityAdjectives[j];

        const name_ab = `${levelAdj} ${qual} ${adj} ${noun}`.toLowerCase();
        const src_ab = ability_sources[overallIndex % ability_sources.length];
        const magic_type = magicTypes[j % magicTypes.length];
        const mana_cost = magic_type === "Physical" ? 0 : lvl * 3 + (j % 5);
        const cooldown = (j % 5) + 1;

        finalAbilities.push({
            name: name_ab,
            level: lvl,
            class_hint: class_hint_ab,
            icon_path: `images/abilities/${name_ab}.png`,
            magic_type,
            mana_cost,
            cooldown
        });

        fs.copyFileSync(path.join(assetsDir, src_ab), path.join(destAbilitiesDir, `${name_ab}.png`));
    }

    // 30 perks per level
    for (let j = 0; j < itemsPerLevel; j++) {
        const overallIndex = (lvl - 1) * itemsPerLevel + j;

        const class_hint_pk = classes[j % classes.length];
        const classWords = perkNamesClasses[class_hint_pk];
        const clsW = classWords[j % classWords.length];
        const subW = perkSubWords[j % perkSubWords.length];
        const qual = qualityAdjectives[j];

        const name_pk = `${levelAdj} ${qual} ${clsW} ${subW}`.toLowerCase();
        const src_pk = perk_sources[overallIndex % perk_sources.length];
        const theme = perkThemes[j % perkThemes.length];

        finalPerks.push({
            name: name_pk,
            level: lvl,
            class_hint: class_hint_pk,
            icon_path: `images/perks/${name_pk}.png`,
            theme
        });

        fs.copyFileSync(path.join(assetsDir, src_pk), path.join(destPerksDir, `${name_pk}.png`));
    }
}

// Write JSON catalogs
fs.writeFileSync(
    path.join(assetsDir, 'data', 'equipment_catalog.json'),
    JSON.stringify(finalEquipment, null, 2)
);
fs.writeFileSync(
    path.join(assetsDir, 'data', 'ability_catalog.json'),
    JSON.stringify(finalAbilities, null, 2)
);
fs.writeFileSync(
    path.join(assetsDir, 'data', 'perk_catalog.json'),
    JSON.stringify(finalPerks, null, 2)
);

console.log('Successfully wrote catalog JSON files.');

// --- Write src/core/catalog.rs ---
let rsContent = `// @generated by icon catalog generation. Do not edit by hand.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GeneratedEquipment { pub name: &'static str, pub level: u8, pub class_hint: &'static str, pub kind: &'static str, pub icon_path: &'static str, pub attack: i32, pub armor: i32, pub crit: i32, pub initiative: i32, pub attack_speed: f32, pub price: u32 }
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GeneratedAbility { pub name: &'static str, pub level: u8, pub class_hint: &'static str, pub magic_type: &'static str, pub mana_cost: u32, pub cooldown: u32, pub icon_path: &'static str }
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GeneratedPerk { pub name: &'static str, pub level: u8, pub class_hint: &'static str, pub theme: &'static str, pub icon_path: &'static str }
`;

rsContent += `pub const GENERATED_EQUIPMENT: [GeneratedEquipment; ${finalEquipment.length}] = [\n`;
for (const eq of finalEquipment) {
    rsContent += `    GeneratedEquipment { name: "${eq.name.toLowerCase()}", level: ${eq.level}, class_hint: "${eq.class_hint}", kind: "${eq.kind}", icon_path: "${eq.icon_path}", attack: ${eq.stats.attack}, armor: ${eq.stats.armor}, crit: ${eq.stats.crit}, initiative: ${eq.stats.initiative}, attack_speed: ${eq.stats.attack_speed.toFixed(1)}, price: ${eq.price} },\n`;
}
rsContent += `];\n\n`;

rsContent += `pub const GENERATED_ABILITIES: [GeneratedAbility; ${finalAbilities.length}] = [\n`;
for (const ab of finalAbilities) {
    rsContent += `    GeneratedAbility { name: "${ab.name.toLowerCase()}", level: ${ab.level}, class_hint: "${ab.class_hint}", magic_type: "${ab.magic_type}", mana_cost: ${ab.mana_cost}, cooldown: ${ab.cooldown}, icon_path: "${ab.icon_path}" },\n`;
}
rsContent += `];\n\n`;

rsContent += `pub const GENERATED_PERKS: [GeneratedPerk; ${finalPerks.length}] = [\n`;
for (const pk of finalPerks) {
    rsContent += `    GeneratedPerk { name: "${pk.name.toLowerCase()}", level: ${pk.level}, class_hint: "${pk.class_hint}", theme: "${pk.theme}", icon_path: "${pk.icon_path}" },\n`;
}
rsContent += `];\n\n`;

// Add direct lookup helper systems
rsContent += `pub fn get_equipment(name: &str) -> Option<GeneratedEquipment> {
    GENERATED_EQUIPMENT.iter().find(|eq| eq.name == name).copied()
}

pub fn get_ability(name: &str) -> Option<GeneratedAbility> {
    GENERATED_ABILITIES.iter().find(|ab| ab.name == name).copied()
}

pub fn get_perk(name: &str) -> Option<GeneratedPerk> {
    GENERATED_PERKS.iter().find(|pk| pk.name == name).copied()
}
`;

fs.writeFileSync(path.join(__dirname, 'src', 'core', 'catalog.rs'), rsContent);
console.log('Successfully wrote src/core/catalog.rs.');

