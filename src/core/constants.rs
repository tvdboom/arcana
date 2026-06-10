use bevy::prelude::{Color, Val};

/// UI
pub const TITLE_TEXT_SIZE: f32 = 5.;
pub const SUBTITLE_TEXT_SIZE: f32 = 4.;
pub const BUTTON_TEXT_SIZE: f32 = 3.;
pub const LABEL_TEXT_SIZE: f32 = 2.;
pub const NORMAL_BUTTON_COLOR: Color = Color::srgba_u8(10, 18, 45, 230);
pub const HOVERED_BUTTON_COLOR: Color = Color::srgb_u8(20, 45, 110);
pub const PRESSED_BUTTON_COLOR: Color = Color::srgb_u8(35, 85, 175);
pub const BUTTON_BORDER_COLOR: Color = Color::srgb_u8(170, 140, 55);
pub const BUTTON_TEXT_COLOR: Color = Color::srgb_u8(230, 205, 145);
pub const DISABLED_BUTTON_COLOR: Color = Color::srgba_u8(10, 18, 45, 80);
pub const DISABLED_BORDER_COLOR: Color = Color::srgba_u8(170, 140, 55, 80);
pub const PLACEHOLDER_COLOR: Color = Color::srgba_u8(40, 40, 55, 220);
pub const BAR_BG_COLOR: Color = Color::srgba_u8(0, 0, 0, 160);
pub const ICON_ITEM: Val = Val::Vw(3.2);

/// Game
pub const START_CHARACTERISTIC: u32 = 10;

pub const NAMES: &[&str] = &[
    "Eldrin",
    "Zephyrus",
    "Thorne",
    "Kaelen",
    "Valerius",
    "Sylas",
    "Baelor",
    "Garrick",
    "Cedric",
    "Gideon",
    "Alistair",
    "Ronan",
    "Dorian",
    "Lucian",
    "Tristan",
    "Percival",
    "Alaric",
    "Orpheus",
    "Tyrion",
    "Ignis",
    "Vaelen",
    "Elidor",
    "Malakor",
    "Rhaegar",
    "Viserys",
    "Jorah",
    "Daario",
    "Arthur",
    "Lancelot",
    "Merlin",
    "Kenneth",
    "Raymond",
    "Jonan",
    "Bran",
    "Sanson",
    "Loras",
    "Oberyn",
    "Theon",
    "Stannis",
    "Davos",
    "Barristan",
    "Joffrey",
    "Sandor",
    "Gregor",
    "Renly",
    "Eddard",
    "Robert",
    "Tywin",
    "Jaime",
    "Ramsay",
];

pub const PET_NAMES: &[&str] = &[
    "Ash", "Bramble", "Cinder", "Dusk", "Echo", "Ember", "Fable", "Fern", "Frost", "Glimmer",
    "Hazel", "Ivy", "Jasper", "Koda", "Luna", "Milo", "Misty", "Nimble", "Nova", "Onyx", "Pip",
    "Quill", "Raven", "River", "Rune", "Sable", "Sage", "Shadow", "Skye", "Soot", "Spark", "Sprig",
    "Storm", "Sunny", "Talon", "Thistle", "Timber", "Toffee", "Twig", "Vale", "Whisper", "Willow",
    "Wisp", "Yara", "Zephyr", "Biscuit", "Copper", "Maple", "Pebble", "Scout",
];
