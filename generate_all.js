const fs = require('fs');
const path = require('path');
const zlib = require('zlib');

function hasTransparency(filePath) {
    try {
        const buf = fs.readFileSync(filePath);
        if (buf.readUInt32BE(0) !== 0x89504E47) return false;
        let offset = 8;
        let colorType = 0, bitDepth = 0, width = 0, height = 0;
        let idatBuffers = [];
        while (offset < buf.length) {
            if (offset + 8 > buf.length) break;
            const length = buf.readUInt32BE(offset);
            const type = buf.toString('ascii', offset + 4, offset + 8);
            if (type === 'IHDR') {
                width = buf.readUInt32BE(offset + 8);
                height = buf.readUInt32BE(offset + 12);
                bitDepth = buf.readUInt8(offset + 16);
                colorType = buf.readUInt8(offset + 17);
            } else if (type === 'IDAT') {
                idatBuffers.push(buf.subarray(offset + 8, offset + 8 + length));
            } else if (type === 'IEND') {
                break;
            }
            offset += 12 + length;
        }
        if (colorType !== 4 && colorType !== 6) return false;
        const compressed = Buffer.concat(idatBuffers);
        const decompressed = zlib.inflateSync(compressed);
        let bytesPerPixel = colorType === 4 ? 2 : 4;
        let ptr = 0;
        const scanlineLength = 1 + width * bytesPerPixel;
        for (let y = 0; y < height; y++) {
            const lineData = decompressed.subarray(ptr + 1, ptr + scanlineLength);
            ptr += scanlineLength;
            for (let x = 0; x < width; x++) {
                let alpha = lineData[x * bytesPerPixel + (colorType === 6 ? 3 : 1)];
                if (alpha < 240) return true;
            }
        }
    } catch (e) {}
    return false;
}

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

const raw_weapon_sources = fs.existsSync(weaponIconsDir) ? getPngFiles(weaponIconsDir, assetsDir, [], true) : [];
const weapon_sources = raw_weapon_sources.filter(src => hasTransparency(path.join(assetsDir, src)));
const ability_sources = fs.existsSync(skillsIconsDir) ? getPngFiles(skillsIconsDir, assetsDir, [], false) : [];
const raw_accessory_sources = fs.existsSync(accessoryIconsDir) ? getPngFiles(accessoryIconsDir, assetsDir, [], false) : [];
const accessory_sources = raw_accessory_sources.filter(src => hasTransparency(path.join(assetsDir, src)));

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
    consumable: [],
    gloves: []
};

for (const src of all_png_sources) {
    if (!hasTransparency(path.join(assetsDir, src))) {
        continue;
    }
    const fileName = path.basename(src).toLowerCase();
    if (fileName.startsWith('helm_') || fileName.startsWith('helms')) {
        category_sources.helmet.push(src);
    } else if (fileName.startsWith('chest_') || fileName === 'cuirass.png') {
        category_sources.armor.push(src);
    } else if (fileName.startsWith('boots_') || fileName === 'bootss.png') {
        category_sources.boots.push(src);
    } else if (fileName.startsWith('shield_') || fileName.startsWith('shields') || fileName.startsWith('book_')) {
        category_sources.offhand.push(src);
    } else if (fileName.startsWith('potion_') || (src.toLowerCase().includes('/alchemy/') && (fileName.includes('potion') || fileName.includes('flask') || fileName.includes('mixture') || fileName === 'holywater.png'))) {
        category_sources.consumable.push(src);
    } else if (fileName.startsWith('dagger_') || fileName.startsWith('sword_') || fileName.startsWith('axe_') || fileName.startsWith('hammer_')) {
        category_sources.one_hand_weapon.push(src);
    } else if (fileName.startsWith('bow_') || fileName.startsWith('crossbow_') || fileName.startsWith('staff_') || fileName.startsWith('spear_') || fileName.startsWith('scythe_')) {
        category_sources.two_hand_weapon.push(src);
    } else if (fileName.startsWith('gloves_')) {
        category_sources.gloves.push(src);
    }
}

// Fallbacks
if (category_sources.helmet.length === 0) category_sources.helmet = weapon_sources;
if (category_sources.armor.length === 0) category_sources.armor = weapon_sources;
if (category_sources.boots.length === 0) category_sources.boots = weapon_sources;
if (category_sources.offhand.length === 0) category_sources.offhand = weapon_sources;
if (category_sources.consumable.length === 0) category_sources.consumable = weapon_sources;
if (category_sources.gloves.length === 0) category_sources.gloves = weapon_sources;
if (category_sources.one_hand_weapon.length === 0) category_sources.one_hand_weapon = weapon_sources;
if (category_sources.two_hand_weapon.length === 0) category_sources.two_hand_weapon = weapon_sources;

// Alphabetical sort of all files so progression is deterministic and follows design
category_sources.helmet.sort();
category_sources.armor.sort();
category_sources.boots.sort();
category_sources.offhand.sort();
category_sources.consumable.sort();
category_sources.gloves.sort();
category_sources.one_hand_weapon.sort();
category_sources.two_hand_weapon.sort();
accessory_sources.sort();

const pools = {
    helmet: [...category_sources.helmet],
    armor: [...category_sources.armor],
    boots: [...category_sources.boots],
    one_hand_weapon: [...category_sources.one_hand_weapon],
    two_hand_weapon: [...category_sources.two_hand_weapon],
    offhand: [...category_sources.offhand],
    accessory: [...accessory_sources],
    consumable: [...category_sources.consumable],
    gloves: [...category_sources.gloves]
};

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
 - Categorized 2H Weapons: ${category_sources.two_hand_weapon.length}
 - Categorized Gloves: ${category_sources.gloves.length}`);

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

const subfolders = ['helmet', 'armor', 'boots', 'weapon', 'accessory', 'consumable', 'gloves'];
for (const sub of subfolders) {
    fs.mkdirSync(path.join(destEquipmentDir, sub), { recursive: true });
}
fs.mkdirSync(destAbilitiesDir, { recursive: true });
fs.mkdirSync(destPerksDir, { recursive: true });

// Constants for generation
const classes = ['warrior', 'mage', 'rogue', 'druid'];
const kinds = ['helmet', 'armor', 'boots', 'one_hand_weapon', 'two_hand_weapon', 'offhand', 'gloves'];

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

function analyzeAccessory(fileName, index) {
    fileName = fileName.toLowerCase();
    let mat = "Bronze"; // default
    if (fileName.includes("gold") || fileName.includes("princess") || fileName.includes("yellow")) {
        mat = "Gold";
    } else if (fileName.includes("silver") || fileName.includes("white")) {
        mat = "Silver";
    } else if (fileName.includes("platinum")) {
        mat = "Platinum";
    } else if (fileName.includes("copper")) {
        mat = "Copper";
    } else if (fileName.includes("pewter")) {
        mat = "Pewter";
    } else if (fileName.includes("crystalblue") || fileName.includes("water")) {
        mat = "Sapphire";
    } else if (fileName.includes("crystalgreen") || fileName.includes("tree")) {
        mat = "Emerald";
    } else if (fileName.includes("crystalpurple") || fileName.includes("purple")) {
        mat = "Amethyst";
    } else if (fileName.includes("red") || fileName.includes("fire")) {
        mat = "Ruby";
    } else if (fileName.includes("black") || fileName.includes("dark") || fileName.includes("necromantic") || fileName.includes("demonic") || fileName.includes("warlock")) {
        mat = "Obsidian";
    } else {
        const mats = ["Bronze", "Copper", "Pewter", "Iron", "Silver", "Gold", "Platinum"];
        mat = mats[index % mats.length];
    }

    let word = "Ring";
    if (fileName.includes("necklace") || fileName.includes("neck_") || fileName.includes("pendant")) {
        word = "Necklace";
        if (fileName.includes("scull") || fileName.includes("skull")) {
            word = "Skull Pendant";
        } else if (fileName.includes("cross")) {
            word = "Crucifix Pendant";
        }
    } else if (fileName.includes("bracelet") || fileName.includes("bracer") || fileName.includes("bangle") || fileName.includes("cuff")) {
        if (fileName.includes("cuff")) word = "Cuff";
        else if (fileName.includes("bangle")) word = "Bangle";
        else if (fileName.includes("bracer")) word = "Bracer";
        else word = "Bracelet";
    } else if (fileName.includes("ring")) {
        word = "Ring";
        if (fileName.includes("lion")) {
            word = "Lion Ring";
        } else if (fileName.includes("dragon")) {
            word = "Dragon Ring";
        } else if (fileName.includes("flower")) {
            word = "Flower Ring";
        }
    }
    return { mat, word };
}

function classifyAccessory(fileName, index, lvl) {
    return analyzeAccessory(fileName, index);
}

function classifyHelmet(fileName, index, lvl) {
    fileName = fileName.toLowerCase();
    let mat = "";
    let word = "";

    if (fileName.includes("gold") || fileName.includes("golden") || fileName.includes("king") || fileName.includes("queen") || fileName.includes("crown")) {
        mat = "Golden";
    } else if (fileName.includes("leather") || fileName.includes("farmer") || fileName.includes("wanderer") || fileName.includes("rogue") || fileName.includes("robber") || fileName.includes("cowl") || fileName.includes("hood")) {
        mat = "Leather";
    } else if (fileName.includes("samurai") || fileName.includes("knight") || fileName.includes("crusader") || fileName.includes("guard") || fileName.includes("footman") || fileName.includes("spearman")) {
        mat = "Steel";
    } else if (fileName.includes("green") || fileName.includes("priest") || fileName.includes("mage")) {
        mat = "Ritual";
    } else {
        if (lvl <= 5) mat = "Bronze";
        else if (lvl <= 12) mat = "Iron";
        else mat = "Plated";
    }

    if (fileName.includes("crown") || fileName.includes("king") || fileName.includes("queen")) {
        mat = "Regal";
        word = "Crown";
        if (fileName.includes("queen")) word = "Queen's Golden Crown";
        else if (fileName.includes("king")) word = "King's Crown";
    } else if (fileName.includes("horn")) {
        word = "Horned Greathelm";
    } else if (fileName.includes("mask")) {
        word = "Visor Mask";
    } else if (fileName.includes("hood") || fileName.includes("cowl")) {
        word = "Hood";
    } else if (fileName.includes("cap") || fileName.includes("farmer")) {
        word = "Cap";
    } else if (fileName.includes("samurai")) {
        word = "Kabuto";
    } else if (fileName.includes("crusader")) {
        word = "Crusader Greathelm";
    } else if (fileName.includes("knight")) {
        word = "Knightly Helm";
    } else if (fileName.includes("guard") || fileName.includes("footman")) {
        word = "Guard's Bascinet";
    } else {
        const options = ["Helm", "Helmet", "Bascinet", "Bascinet", "Bascinet", "Bascinet"];
        word = options[index % options.length];
    }

    return { mat, word };
}

function classifyArmor(fileName, index, lvl) {
    fileName = fileName.toLowerCase();
    let mat = "";
    let word = "";

    if (fileName.includes("leather") || fileName.includes("scout") || fileName.includes("adventure")) {
        mat = "Reinforced Leather";
    } else if (fileName.includes("cloth") || fileName.includes("farmer") || fileName.includes("citizen") || fileName.includes("trader")) {
        mat = "Quilted Cloth";
    } else if (fileName.includes("mage") || fileName.includes("wizard") || fileName.includes("priest") || fileName.includes("archimage")) {
        mat = "Silk";
    } else if (fileName.includes("gold") || fileName.includes("golden")) {
        mat = "Golden";
    } else if (fileName.includes("green")) {
        mat = "Verdant";
    } else if (fileName.includes("knight") || fileName.includes("cuirass") || fileName.includes("milita") || fileName.includes("warchief")) {
        mat = "Steel";
    } else {
        if (lvl <= 5) mat = "Bronze";
        else if (lvl <= 12) mat = "Iron";
        else mat = "Plated";
    }

    if (fileName.includes("robe") || fileName.includes("mage") || fileName.includes("wizard") || fileName.includes("priest") || fileName.includes("archimage")) {
        word = "Robes";
    } else if (fileName.includes("cuirass") || fileName.includes("chestplate") || fileName.includes("knight")) {
        word = "Cuirass";
    } else if (fileName.includes("leather") || fileName.includes("farmer") || fileName.includes("citizen") || fileName.includes("adventure") || fileName.includes("scout") || fileName.includes("trader")) {
        word = "Tunic";
        if (fileName.includes("leatherplus") || fileName.includes("warchief")) word = "Jerkin";
    } else {
        const options = ["Chestplate", "Plated Mail", "Scale Armor", "Hauberk", "Breastplate"];
        word = options[index % options.length];
    }

    return { mat, word };
}

function classifyBoots(fileName, index, lvl) {
    fileName = fileName.toLowerCase();
    let mat = "";
    let word = "";

    if (fileName.includes("gold") || fileName.includes("golden")) {
        mat = "Golden";
    } else if (fileName.includes("paladin") || fileName.includes("knight")) {
        mat = "Steel Plated";
    } else if (fileName.includes("speed") || fileName.includes("magic")) {
        mat = "Enchanted";
    } else if (fileName.includes("leather") || fileName.includes("common") || fileName.includes("ogre")) {
        mat = "Heavy Leather";
    } else {
        if (lvl <= 5) mat = "Rawhide";
        else if (lvl <= 12) mat = "Iron";
        else mat = "Heavy Steel";
    }

    if (fileName.includes("slipper") || fileName.includes("magic")) {
        word = "Slippers";
    } else if (fileName.includes("greave") || fileName.includes("paladin") || fileName.includes("knight") || fileName.includes("boots_22") || fileName.includes("boots_35")) {
        word = "Sabatons";
    } else if (fileName.includes("shoe")) {
        word = "Shoes";
    } else {
        const options = ["Boots", "Greaves", "Treads", "Soles"];
        word = options[index % options.length];
    }

    return { mat, word };
}

function classifyOneHandWeapon(fileName, index, lvl) {
    fileName = fileName.toLowerCase();
    let mat = "";
    let word = "";

    if (fileName.includes("gold") || fileName.includes("golden")) {
        mat = "Gold-Embossed";
    } else if (fileName.includes("copper")) {
        mat = "Copper";
    } else if (fileName.includes("bronze")) {
        mat = "Bronze";
    } else if (fileName.includes("iron")) {
        mat = "Iron";
    } else if (fileName.includes("shadow")) {
        mat = "Shadow";
    } else {
        if (lvl <= 5) mat = "Worn";
        else if (lvl <= 12) mat = "Tempered";
        else mat = "Mighty";
    }

    if (fileName.includes("dagger") || fileName.includes("knife") || fileName.includes("stiletto") || fileName.includes("striker")) {
        word = "Dagger";
        if (fileName.includes("knife")) word = "Knife";
        else if (fileName.includes("stiletto")) word = "Stiletto";
        else if (fileName.includes("striker")) word = "Striker";
    } else if (fileName.includes("sword") || fileName.includes("blade") || fileName.includes("saber") || fileName.includes("rapier") || fileName.includes("cutlass")) {
        word = "Sword";
        if (fileName.includes("blade")) word = "Blade";
        else if (fileName.includes("saber")) word = "Saber";
        else if (fileName.includes("rapier")) word = "Rapier";
        else if (fileName.includes("cutlass")) word = "Cutlass";
    } else if (fileName.includes("axe") || fileName.includes("hatchet") || fileName.includes("cleaver") || fileName.includes("handaxe")) {
        word = "Axe";
        if (fileName.includes("hatchet")) word = "Hatchet";
        else if (fileName.includes("cleaver")) word = "Cleaver";
        else if (fileName.includes("handaxe")) word = "Handaxe";
    } else if (fileName.includes("hammer") || fileName.includes("mace") || fileName.includes("warhammer") || fileName.includes("flail") || fileName.includes("club") || fileName.includes("mace")) {
        word = "Mace";
        if (fileName.includes("hammer") || fileName.includes("warhammer")) word = "Hammer";
        else if (fileName.includes("flail")) word = "Flail";
        else if (fileName.includes("club")) word = "Club";
    } else {
        const options = ["Sword", "Blade", "Axe", "Mace", "Dagger"];
        word = options[index % options.length];
    }

    return { mat, word };
}

function classifyTwoHandWeapon(fileName, index, lvl) {
    fileName = fileName.toLowerCase();
    let mat = "";
    let word = "";

    if (fileName.includes("gold") || fileName.includes("golden")) {
        mat = "Golden";
    } else if (fileName.includes("elder") || fileName.includes("ancient")) {
        mat = "Elder";
    } else if (fileName.includes("primal") || fileName.includes("verdant")) {
        mat = "Primal";
    } else {
        if (lvl <= 5) mat = "Bronze";
        else if (lvl <= 12) mat = "Ironed";
        else mat = "Masterwork";
    }

    if (fileName.includes("bow") || fileName.includes("crossbow") || fileName.includes("longbow") || fileName.includes("shortbow")) {
        word = "Bow";
        if (fileName.includes("crossbow")) word = "Crossbow";
        else if (fileName.includes("longbow")) word = "Longbow";
        else if (fileName.includes("shortbow")) word = "Shortbow";
        else if (fileName.includes("recurve")) word = "Recurve Bow";
    } else if (fileName.includes("staff") || fileName.includes("greatstaff") || fileName.includes("crook") || fileName.includes("spire")) {
        word = "Staff";
        if (fileName.includes("greatstaff")) word = "Greatstaff";
        else if (fileName.includes("crook")) word = "Crook Staff";
        else if (fileName.includes("spire")) word = "Spire Staff";
    } else if (fileName.includes("spear") || fileName.includes("halberd") || fileName.includes("pike") || fileName.includes("lance") || fileName.includes("trident")) {
        word = "Spear";
        if (fileName.includes("halberd")) word = "Halberd";
        else if (fileName.includes("pike")) word = "Pike";
        else if (fileName.includes("lance")) word = "Lance";
        else if (fileName.includes("trident")) word = "Trident";
    } else if (fileName.includes("scythe") || fileName.includes("reaper") || fileName.includes("harvester")) {
        word = "Scythe";
        if (fileName.includes("reaper")) word = "Reaper Scythe";
        else if (fileName.includes("harvester")) word = "Harvester Scythe";
    } else {
        const options = ["Greatsword", "Warhammer", "Staff", "Bow", "Halberd", "Scythe"];
        word = options[index % options.length];
    }

    return { mat, word };
}

function classifyOffhand(fileName, index, lvl) {
    fileName = fileName.toLowerCase();
    let mat = "";
    let word = "";

    if (fileName.includes("gold") || fileName.includes("golden")) {
        mat = "Golden";
    } else if (fileName.includes("book") || fileName.includes("tome")) {
        mat = "Enchanted";
    } else if (fileName.includes("wood") || fileName.includes("wooden")) {
        mat = "Wooden";
    } else {
        if (lvl <= 5) mat = "Bronze";
        else if (lvl <= 12) mat = "Iron";
        else mat = "Heavily Plated";
    }

    if (fileName.includes("book") || fileName.includes("tome") || fileName.includes("grimoire") || fileName.includes("scroll")) {
        word = "Tome";
        if (fileName.includes("grimoire")) word = "Grimoire";
        else if (fileName.includes("scroll")) word = "Scroll";
    } else if (fileName.includes("shield") || fileName.includes("buckler") || fileName.includes("aegis") || fileName.includes("bulwark") || fileName.includes("greatshield")) {
        word = "Shield";
        if (fileName.includes("buckler")) word = "Buckler";
        else if (fileName.includes("aegis")) word = "Aegis";
        else if (fileName.includes("bulwark")) word = "Bulwark";
        else if (fileName.includes("greatshield")) word = "Greatshield";
    } else {
        const options = ["Shield", "Buckler", "Aegis", "Tome", "Grimoire"];
        word = options[index % options.length];
    }

    return { mat, word };
}

function classifyGloves(fileName, index, lvl) {
    fileName = fileName.toLowerCase();
    let mat = "";
    const words = ["Gloves", "Gauntlets", "Handwraps", "Vambraces"];
    let word = words[index % words.length];

    if (fileName.includes("dragon")) {
        mat = "Dragon";
        word = "Gauntlets";
    } else if (fileName.includes("witch")) {
        mat = "Arcane";
        word = "Handwraps";
    } else if (fileName.includes("death")) {
        mat = "Death";
        word = "Gauntlets";
    } else if (fileName.includes("archer")) {
        mat = "Ranger";
        word = "Gloves";
    } else {
        const mats = ["Leather", "Iron", "Steel", "Mithril", "Bronze", "Silver"];
        mat = mats[lvl % mats.length];
    }
    return { mat, word };
}

function classifyConsumable(fileName, index, lvl) {
    fileName = fileName.toLowerCase();
    let word = "Potion";

    if (fileName.includes("holywater")) {
        return { mat: "Sacred", word: "Holy Water" };
    }
    if (fileName.includes("tea")) {
        return { mat: "Soothe", word: "Medicinal Tea" };
    }
    if (fileName.includes("mortar")) {
        return { mat: "Crushed", word: "Herbal Paste" };
    }

    let mat = "Novice";
    if (fileName.includes("huge") || fileName.includes("big") || fileName.includes("philosopher") || fileName.includes("immortal")) {
        if (lvl <= 5) mat = "Potent";
        else if (lvl <= 12) mat = "Greater";
        else mat = "Sovereign";
    } else if (fileName.includes("middle") || fileName.includes("magic")) {
        if (lvl <= 8) mat = "Standard";
        else mat = "Adept";
    } else if (fileName.includes("little") || fileName.includes("mini") || fileName.includes("tea")) {
        mat = "Minor";
    } else {
        const mats = ["Novice", "Minor", "Standard", "Potent", "Major", "Greater"];
        mat = mats[lvl % mats.length];
    }

    if (fileName.includes("heal") || fileName.includes("blood")) {
        word = "Health Potion";
    } else if (fileName.includes("mana") || fileName.includes("blue_potion") || fileName.includes("blue_mixture")) {
        word = "Mana Potion";
    } else if (fileName.includes("poison") || fileName.includes("plague")) {
        word = "Antidote Vial";
    } else if (fileName.includes("energy") || fileName.includes("stamina")) {
        word = "Stamina Potion";
    } else if (fileName.includes("rejuvenation") || fileName.includes("magic") || fileName.includes("spiritual")) {
        word = "Rejuvenation Elixir";
    } else if (fileName.includes("strength")) {
        word = "Strength Potion";
    } else if (fileName.includes("dexterity") || fileName.includes("speed")) {
        word = "Dexterity Potion";
    } else if (fileName.includes("constitution") || fileName.includes("reactive")) {
        word = "Constitution Potion";
    } else if (fileName.includes("intelligence") || fileName.includes("wisdom") || fileName.includes("philosopher")) {
        word = "Intelligence Potion";
    } else if (fileName.includes("charisma") || fileName.includes("invisibility")) {
        word = "Charisma Potion";
    } else {
        const options = ["Health Potion", "Mana Potion", "Rejuvenation Elixir", "Antidote Vial", "Stamina Potion"];
        word = options[index % options.length];
    }

    return { mat, word };
}

function getRealisticItemName(kind, fileName, index, lvl) {
    let res;
    if (kind === 'helmet') {
        res = classifyHelmet(fileName, index, lvl);
    } else if (kind === 'armor') {
        res = classifyArmor(fileName, index, lvl);
    } else if (kind === 'boots') {
        res = classifyBoots(fileName, index, lvl);
    } else if (kind === 'one_hand_weapon') {
        res = classifyOneHandWeapon(fileName, index, lvl);
    } else if (kind === 'two_hand_weapon') {
        res = classifyTwoHandWeapon(fileName, index, lvl);
    } else if (kind === 'offhand') {
        res = classifyOffhand(fileName, index, lvl);
    } else if (kind === 'accessory') {
        res = classifyAccessory(fileName, index, lvl);
    } else if (kind === 'consumable') {
        res = classifyConsumable(fileName, index, lvl);
    } else if (kind === 'gloves') {
        res = classifyGloves(fileName, index, lvl);
    } else {
        res = { mat: "Standard", word: "Item" };
    }

    const levelAdj = levelAdjectives[lvl - 1];

    if (kind === 'accessory') {
        const class_hint = classes[index % classes.length];
        const classPrefixes = {
            warrior: ["Valor", "Might", "Fortitude", "Brutality"],
            skin: ["Valor", "Might", "Fortitude", "Brutality"],
            mage: ["Arcana", "Focus", "Intelligence", "Evocation"],
            rogue: ["Cunning", "Velocity", "Deception", "Shadows"],
            druid: ["Thorns", "Wilds", "Restoration", "Earth"]
        };
        const classAdjs = classPrefixes[class_hint] || classPrefixes.warrior;
        const classAdj = classAdjs[index % classAdjs.length];
        return `${levelAdj} ${res.mat} ${res.word} of ${classAdj}`.toLowerCase();
    } else if (kind === 'consumable') {
        return `${levelAdj} ${res.mat} ${res.word}`.toLowerCase();
    } else {
        const class_hint_eq = classes[index % classes.length];
        const classAdjectives = {
            warrior: ["Vanguard", "Warmonger", "Guardian", "Bulwark", "Mighty", "Challenger"],
            mage: ["Arcane", "Pyromancer", "Spellweaver", "Sorcerer", "Evoker", "Acolyte"],
            rogue: ["Assassin", "Shadow", "Silent", "Phantom", "Infiltrator", "Stalker"],
            druid: ["Barkskin", "Wildheart", "Forest", "Primal", "Nature", "Verdant"]
        };
        const classAdjs = classAdjectives[class_hint_eq] || classAdjectives.warrior;
        const classAdj = classAdjs[index % classAdjs.length];

        return `${levelAdj} ${res.mat} ${classAdj} ${res.word}`.toLowerCase();
    }
}

for (let lvl = 1; lvl <= totalLevels; lvl++) {
    const levelStr = lvl.toString().padStart(2, '0');
    const levelAdj = levelAdjectives[lvl - 1];

    // 30 base equipment pieces
    for (let j = 0; j < itemsPerLevel; j++) {
        const class_hint_eq = classes[j % classes.length];
        const kind_eq = kinds[j % kinds.length];

        const src_eq = pools[kind_eq].shift();
        if (!src_eq) {
            continue; // Out of unique images for this kind, skip!
        }

        const name_eq = getRealisticItemName(kind_eq, path.basename(src_eq), j, lvl);

        let attack = 0;
        let armor = 0;
        let crit = 0;
        let initiative = j % 5 - 1;
        let attack_speed = 0.0;

        if (kind_eq === 'one_hand_weapon') {
            attack = Math.floor(lvl) + 2;
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

        if (kind_eq === 'gloves') {
            armor = Math.floor(lvl * 0.3) + 1;
            crit = 2;
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
        const src_acc = pools.accessory.shift();
        if (!src_acc) {
            continue; // Skip if accessory_pool is empty!
        }
        const class_hint = classes[a % classes.length];
        const name_acc = getRealisticItemName("accessory", path.basename(src_acc), a, lvl);

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
    for (let c = 0; c < 10; c++) {
        const src_con = pools.consumable.shift();
        if (!src_con) {
            continue; // Skip if consumable pool is empty!
        }
        const name_con = getRealisticItemName("consumable", path.basename(src_con), c, lvl);
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

