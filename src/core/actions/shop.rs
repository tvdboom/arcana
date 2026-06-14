use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::build::equipment::{Equipment, Kind};
use crate::core::build::weapons::{Category, Hand};
use crate::core::build::wearables::WearableSlot;
use crate::core::catalog::{all_equipment, get_equipment};
use crate::core::constants::{
    BUTTON_BORDER_COLOR, BUTTON_TEXT_COLOR, HOVERED_BUTTON_COLOR, PRESSED_BUTTON_COLOR,
};
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_text, recolor};
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::core::ui::tooltip::{spawn_item_tooltip, TooltipNode};
use crate::core::ui::utils::*;
use crate::core::utils::cursor;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Resource, Clone, Serialize, Deserialize, Default, Debug)]
pub struct ShopInventory {
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShopTab {
    #[default]
    Weapons,
    Helmets,
    Chestplates,
    Boots,
    Gloves,
    Accessories,
    Consumables,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WeaponTypeFilter {
    #[default]
    All,
    Weapons,
    Shields,
    Books,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopFilters {
    pub tab: ShopTab,
    pub weapon_hand: Option<Hand>,
    pub weapon_type: WeaponTypeFilter,
    pub weapon_category: Option<Category>,
    pub kind: Option<Kind>,
}

#[derive(Component)]
pub struct ShopGoldLabel;

impl Default for ShopFilters {
    fn default() -> Self {
        Self {
            tab: ShopTab::Weapons,
            weapon_hand: None,
            weapon_type: WeaponTypeFilter::All,
            weapon_category: None,
            kind: None,
        }
    }
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                1
            } else {
                seed
            },
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state >> 32) as u32
    }

    fn random_bool(&mut self, p: f64) -> bool {
        let val = self.next_u32() as f64 / u32::MAX as f64;
        val < p
    }
}

pub fn generate_deterministic_shop(player_name: &str, player_level: u32) -> Vec<String> {
    let mut hasher = DefaultHasher::new();
    player_name.hash(&mut hasher);
    let seed = hasher.finish();
    let mut rng = DeterministicRng::new(seed);

    let mut items = Vec::new();
    for eq in all_equipment() {
        if eq.level() >= 1 && eq.level() <= player_level {
            if rng.random_bool(0.5) {
                items.push(eq.name().to_string());
            }
        }
    }
    items
}

#[derive(Component)]
pub struct ShopContentWrapper;

pub fn setup_shop_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    mut shop_inventory: ResMut<ShopInventory>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }

    shop_inventory.items = generate_deterministic_shop(&player.name, player.level as u32);

    if let Some(container_entity) = columns_container_q.iter().next() {
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_shop");
        commands.entity(panel_entity).with_children(|parent| {
            build_shop_content(
                parent,
                &assets,
                &localization,
                settings.language,
                &shop_inventory,
                ShopFilters::default(),
                player.gold,
            );
        });
    }
}

pub fn update_shop_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    shop_inventory: Res<ShopInventory>,
    filters: Res<ShopFilters>,
    wrapper_q: Query<Entity, With<ShopContentWrapper>>,
    children_q: Query<&Children>,
) {
    if filters.is_changed() || shop_inventory.is_changed() {
        if let Some(wrapper_entity) = wrapper_q.iter().next() {
            despawn_descendants_manual(&mut commands, wrapper_entity, &children_q);
            commands.entity(wrapper_entity).with_children(|parent| {
                build_shop_content_inner(
                    parent,
                    &assets,
                    &localization,
                    settings.language,
                    &shop_inventory,
                    *filters,
                    player.gold,
                );
            });
        }
    }
}

pub fn build_shop_content(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    shop_inventory: &ShopInventory,
    filters: ShopFilters,
    player_gold: u32,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                height: percent(100.),
                padding: UiRect::all(percent(5.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ShopContentWrapper,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|parent| {
                    build_shop_content_inner(
                        parent,
                        assets,
                        localization,
                        lang,
                        shop_inventory,
                        filters,
                        player_gold,
                    );
                });

            parent.spawn((
                Node {
                    width: percent(100.),
                    height: Val::Px(50.),
                    margin: UiRect::top(Val::Px(10.)),
                    ..default()
                },
                ImageNode::new(assets.image("banner")).with_mode(NodeImageMode::Stretch),
            ));
        });
}

pub fn build_shop_content_inner(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    shop_inventory: &ShopInventory,
    filters: ShopFilters,
    player_gold: u32,
) {
    parent
        .spawn(Node {
            width: percent(100.),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.),
                    ..default()
                })
                .with_children(|parent| {
                    for tab in [
                        ShopTab::Weapons,
                        ShopTab::Helmets,
                        ShopTab::Chestplates,
                        ShopTab::Boots,
                        ShopTab::Gloves,
                        ShopTab::Accessories,
                        ShopTab::Consumables,
                    ] {
                        let is_active = tab == filters.tab;
                        let bg_color = if is_active {
                            PRESSED_BUTTON_COLOR
                        } else {
                            Color::srgba_u8(20, 20, 35, 200)
                        };
                        let label = match tab {
                            ShopTab::Weapons => "Weapons",
                            ShopTab::Helmets => "Helmets",
                            ShopTab::Chestplates => "Chestplates",
                            ShopTab::Boots => "Boots",
                            ShopTab::Gloves => "Gloves",
                            ShopTab::Accessories => "Accessories",
                            ShopTab::Consumables => "Consumables",
                        };
                        parent
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                                    border: UiRect::all(Val::Px(1.)),
                                    ..default()
                                },
                                BackgroundColor(bg_color),
                                BorderColor::all(BUTTON_BORDER_COLOR),
                                Button,
                                Interaction::default(),
                                Pickable::default(),
                                ShopTabButton(tab),
                            ))
                            .observe(handle_shop_tab_click)
                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                            .with_children(|parent| {
                                parent.spawn((
                                    add_text(label, "bold", 1.8, assets),
                                    TextColor(BUTTON_TEXT_COLOR),
                                ));
                            });
                    }
                });

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.),
                        ..default()
                    },
                    Interaction::default(),
                    Pickable::default(),
                    crate::core::ui::playing::InfoTooltip::Gold,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Vw(2.4),
                            height: Val::Vw(2.4),
                            ..default()
                        },
                        ImageNode::new(assets.image("gold")).with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(player_gold.to_string(), "bold", 2.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        ShopGoldLabel,
                    ));
                });
        });

    if filters.tab == ShopTab::Weapons {
        parent
            .spawn(Node {
                width: percent(100.),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.),
                padding: UiRect::all(Val::Px(6.)),
                margin: UiRect::bottom(Val::Px(10.)),
                ..default()
            })
            .insert(BackgroundColor(Color::srgba_u8(10, 10, 20, 150)))
            .insert(BorderColor::all(BUTTON_BORDER_COLOR))
            .insert(Node {
                border: UiRect::all(Val::Px(1.)),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn(Node {
                        width: percent(100.),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(add_text("Hand: ", "bold", 1.4, assets));
                                for (opt, label) in [
                                    (None, "Both"),
                                    (Some(Hand::OneHand), "1-Hand"),
                                    (Some(Hand::TwoHand), "2-Hand"),
                                ] {
                                    let is_active = filters.weapon_hand == opt;
                                    let bg_color = if is_active {
                                        HOVERED_BUTTON_COLOR
                                    } else {
                                        Color::srgba_u8(10, 15, 30, 200)
                                    };
                                    parent
                                        .spawn((
                                            Node {
                                                padding: UiRect::axes(Val::Px(6.), Val::Px(3.)),
                                                border: UiRect::all(Val::Px(1.)),
                                                ..default()
                                            },
                                            BackgroundColor(bg_color),
                                            BorderColor::all(BUTTON_BORDER_COLOR),
                                            Button,
                                            Interaction::default(),
                                            Pickable::default(),
                                            ShopHandFilterButton(opt),
                                        ))
                                        .observe(handle_shop_hand_filter_click)
                                        .with_children(|parent| {
                                            parent.spawn((
                                                add_text(label, "medium", 1.2, assets),
                                                TextColor(BUTTON_TEXT_COLOR),
                                            ));
                                        });
                                }
                            });

                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(add_text("Type: ", "bold", 1.4, assets));
                                for (opt, label) in [
                                    (WeaponTypeFilter::All, "All"),
                                    (WeaponTypeFilter::Weapons, "Weapons"),
                                    (WeaponTypeFilter::Shields, "Shields"),
                                    (WeaponTypeFilter::Books, "Books"),
                                ] {
                                    let is_active = filters.weapon_type == opt;
                                    let bg_color = if is_active {
                                        HOVERED_BUTTON_COLOR
                                    } else {
                                        Color::srgba_u8(10, 15, 30, 200)
                                    };
                                    parent
                                        .spawn((
                                            Node {
                                                padding: UiRect::axes(Val::Px(6.), Val::Px(3.)),
                                                border: UiRect::all(Val::Px(1.)),
                                                ..default()
                                            },
                                            BackgroundColor(bg_color),
                                            BorderColor::all(BUTTON_BORDER_COLOR),
                                            Button,
                                            Interaction::default(),
                                            Pickable::default(),
                                            ShopTypeFilterButton(opt),
                                        ))
                                        .observe(handle_shop_type_filter_click)
                                        .with_children(|parent| {
                                            parent.spawn((
                                                add_text(label, "medium", 1.2, assets),
                                                TextColor(BUTTON_TEXT_COLOR),
                                            ));
                                        });
                                }
                            });
                    });

                parent
                    .spawn(Node {
                        width: percent(100.),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(add_text("Category: ", "bold", 1.4, assets));
                                for (opt, label) in [
                                    (None, "All"),
                                    (Some(Category::Melee), "Melee"),
                                    (Some(Category::Finesse), "Finesse"),
                                    (Some(Category::Range), "Range"),
                                    (Some(Category::Magical), "Magical"),
                                ] {
                                    let is_active = filters.weapon_category == opt;
                                    let bg_color = if is_active {
                                        HOVERED_BUTTON_COLOR
                                    } else {
                                        Color::srgba_u8(10, 15, 30, 200)
                                    };
                                    parent
                                        .spawn((
                                            Node {
                                                padding: UiRect::axes(Val::Px(6.), Val::Px(3.)),
                                                border: UiRect::all(Val::Px(1.)),
                                                ..default()
                                            },
                                            BackgroundColor(bg_color),
                                            BorderColor::all(BUTTON_BORDER_COLOR),
                                            Button,
                                            Interaction::default(),
                                            Pickable::default(),
                                            ShopCategoryFilterButton(opt),
                                        ))
                                        .observe(handle_shop_category_filter_click)
                                        .with_children(|parent| {
                                            parent.spawn((
                                                add_text(label, "medium", 1.2, assets),
                                                TextColor(BUTTON_TEXT_COLOR),
                                            ));
                                        });
                                }
                            });

                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(add_text("Kind: ", "bold", 1.4, assets));
                                for (opt, label) in [
                                    (None, "All"),
                                    (Some(Kind::Physical), "Physical"),
                                    (Some(Kind::Fire), "Fire"),
                                    (Some(Kind::Ice), "Ice"),
                                    (Some(Kind::Nature), "Nature"),
                                    (Some(Kind::Holy), "Holy"),
                                    (Some(Kind::Shadow), "Shadow"),
                                ] {
                                    let is_active = filters.kind == opt;
                                    let bg_color = if is_active {
                                        HOVERED_BUTTON_COLOR
                                    } else {
                                        Color::srgba_u8(10, 15, 30, 200)
                                    };
                                    parent
                                        .spawn((
                                            Node {
                                                padding: UiRect::axes(Val::Px(6.), Val::Px(3.)),
                                                border: UiRect::all(Val::Px(1.)),
                                                ..default()
                                            },
                                            BackgroundColor(bg_color),
                                            BorderColor::all(BUTTON_BORDER_COLOR),
                                            Button,
                                            Interaction::default(),
                                            Pickable::default(),
                                            ShopKindFilterButton(opt),
                                        ))
                                        .observe(handle_shop_kind_filter_click)
                                        .with_children(|parent| {
                                            parent.spawn((
                                                add_text(label, "medium", 1.2, assets),
                                                TextColor(BUTTON_TEXT_COLOR),
                                            ));
                                        });
                                }
                            });
                    });
            });
    }

    parent
        .spawn(Node {
            width: percent(100.),
            height: percent(100.),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(10.),
            row_gap: Val::Px(10.),
            overflow: Overflow::clip(),
            ..default()
        })
        .with_children(|parent| {
            let mut empty = true;
            for item_key in &shop_inventory.items {
                if let Some(equipment) = get_equipment(item_key) {
                    let matches_tab = match filters.tab {
                        ShopTab::Weapons => matches!(equipment, Equipment::Weapon(_)),
                        ShopTab::Helmets => match &equipment {
                            Equipment::Wearable(w) => w.slot == WearableSlot::Helmet,
                            _ => false,
                        },
                        ShopTab::Chestplates => match &equipment {
                            Equipment::Wearable(w) => w.slot == WearableSlot::Chestplate,
                            _ => false,
                        },
                        ShopTab::Boots => match &equipment {
                            Equipment::Wearable(w) => w.slot == WearableSlot::Boots,
                            _ => false,
                        },
                        ShopTab::Gloves => match &equipment {
                            Equipment::Wearable(w) => w.slot == WearableSlot::Gloves,
                            _ => false,
                        },
                        ShopTab::Accessories => match &equipment {
                            Equipment::Wearable(w) => w.slot == WearableSlot::Accessory,
                            _ => false,
                        },
                        ShopTab::Consumables => match &equipment {
                            Equipment::Wearable(w) => w.slot == WearableSlot::Consumable,
                            _ => false,
                        },
                    };
                    if !matches_tab {
                        continue;
                    }

                    if filters.tab == ShopTab::Weapons {
                        if let Equipment::Weapon(ref w) = equipment {
                            if let Some(hand) = filters.weapon_hand {
                                if w.hand != hand {
                                    continue;
                                }
                            }
                            match filters.weapon_type {
                                WeaponTypeFilter::Weapons => {
                                    if w.category == Category::Shield
                                        || w.category == Category::Book
                                    {
                                        continue;
                                    }
                                },
                                WeaponTypeFilter::Shields => {
                                    if w.category != Category::Shield {
                                        continue;
                                    }
                                },
                                WeaponTypeFilter::Books => {
                                    if w.category != Category::Book {
                                        continue;
                                    }
                                },
                                WeaponTypeFilter::All => {},
                            }
                            if let Some(category) = filters.weapon_category {
                                if w.category != category {
                                    continue;
                                }
                            }
                            if let Some(kind) = filters.kind {
                                if w.kind != kind {
                                    continue;
                                }
                            }
                        } else {
                            continue;
                        }
                    }

                    empty = false;
                    spawn_shop_item_card(parent, assets, localization, lang, &equipment);
                }
            }

            if empty {
                parent.spawn((
                    add_text("No items fit these conditions.", "bold", 2.0, assets),
                    TextColor(Color::WHITE),
                ));
            }
        });
}

#[derive(Component)]
pub struct ShopItemCard {
    pub key: String,
    pub price: u32,
}

#[derive(Component)]
pub struct ShopItemTooltip(pub String);

fn spawn_shop_item_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    item: &Equipment,
) {
    let name_str = item.name().to_string();
    let prefix = match item {
        Equipment::Weapon(_) => "weapon",
        Equipment::Wearable(_) => "wearable",
    };
    let name = crate::core::ui::playing::name_with_level(
        item.name(),
        prefix,
        item.level() as u8,
        localization,
        lang,
    );

    parent
        .spawn((
            Node {
                width: Val::Px(160.),
                height: Val::Px(160.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::all(Val::Px(1.5)),
                padding: UiRect::all(Val::Px(4.)),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(10, 15, 30, 240)),
            BorderColor::all(BUTTON_BORDER_COLOR),
            Interaction::default(),
            Button,
            Pickable::default(),
            ShopItemCard {
                key: name_str.clone(),
                price: item.price(),
            },
            ShopItemTooltip(name_str.clone()),
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(Color::srgba_u8(10, 15, 30, 240)))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(handle_shop_item_card_click)
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(2.)),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((add_text(name, "bold", 1.3, assets), TextColor(BUTTON_TEXT_COLOR)));

                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(2.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    width: Val::Px(14.),
                                    height: Val::Px(14.),
                                    ..default()
                                },
                                ImageNode::new(assets.image("gold"))
                                    .with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text(format!("{}", item.price()), "bold", 1.3, assets),
                                TextColor(BUTTON_TEXT_COLOR),
                            ));
                        });
                });

            parent.spawn((
                Node {
                    width: Val::Px(100.),
                    height: Val::Px(100.),
                    border: UiRect::all(Val::Px(1.)),
                    margin: UiRect::bottom(Val::Px(4.)),
                    ..default()
                },
                BorderColor::all(BUTTON_BORDER_COLOR),
                ImageNode::new(assets.image(format!("build_{}", item.name())))
                    .with_mode(NodeImageMode::Stretch),
            ));
        });
}

#[derive(Component, Clone, Copy)]
pub struct ShopTabButton(pub ShopTab);

#[derive(Component, Clone, Copy)]
pub struct ShopHandFilterButton(pub Option<Hand>);

#[derive(Component, Clone, Copy)]
pub struct ShopTypeFilterButton(pub WeaponTypeFilter);

#[derive(Component, Clone, Copy)]
pub struct ShopCategoryFilterButton(pub Option<Category>);

#[derive(Component, Clone, Copy)]
pub struct ShopKindFilterButton(pub Option<Kind>);

pub fn handle_shop_tab_click(
    event: On<Pointer<Click>>,
    mut filters: ResMut<ShopFilters>,
    btn_q: Query<&ShopTabButton>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        filters.tab = btn.0;
        play_audio_msg.write(PlayAudioMsg::new("click"));
    }
}

pub fn handle_shop_hand_filter_click(
    event: On<Pointer<Click>>,
    mut filters: ResMut<ShopFilters>,
    btn_q: Query<&ShopHandFilterButton>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        filters.weapon_hand = btn.0;
        play_audio_msg.write(PlayAudioMsg::new("click"));
    }
}

pub fn handle_shop_type_filter_click(
    event: On<Pointer<Click>>,
    mut filters: ResMut<ShopFilters>,
    btn_q: Query<&ShopTypeFilterButton>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        filters.weapon_type = btn.0;
        play_audio_msg.write(PlayAudioMsg::new("click"));
    }
}

pub fn handle_shop_category_filter_click(
    event: On<Pointer<Click>>,
    mut filters: ResMut<ShopFilters>,
    btn_q: Query<&ShopCategoryFilterButton>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        filters.weapon_category = btn.0;
        play_audio_msg.write(PlayAudioMsg::new("click"));
    }
}

pub fn handle_shop_kind_filter_click(
    event: On<Pointer<Click>>,
    mut filters: ResMut<ShopFilters>,
    btn_q: Query<&ShopKindFilterButton>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        filters.kind = btn.0;
        play_audio_msg.write(PlayAudioMsg::new("click"));
    }
}

pub fn handle_shop_item_card_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    card_q: Query<&ShopItemCard>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
) {
    if let Ok(card) = card_q.get(event.entity) {
        let buy_price = card.price;
        if player.gold < buy_price {
            play_audio_msg.write(PlayAudioMsg::new("error"));
            if let Some(toast) = toast_container_q.iter().next() {
                spawn_toast(
                    &mut commands,
                    &assets,
                    "Not enough gold!".to_string(),
                    Color::srgba(0.20, 0.05, 0.05, 0.93),
                    Color::srgb(0.85, 0.20, 0.20),
                    Color::srgb(1.0, 0.80, 0.80),
                    toast,
                );
            }
            return;
        }

        player.gold -= buy_price;
        play_audio_msg.write(PlayAudioMsg::new("buy"));

        if event.button == PointerButton::Primary {
            player.inventory.push(card.key.clone());
            let is_consumable = get_equipment(&card.key)
                .map(|eq| match eq {
                    Equipment::Wearable(w) => w.slot == WearableSlot::Consumable,
                    _ => false,
                })
                .unwrap_or(false);
            if !is_consumable {
                crate::core::ui::playing::equip_item(&mut player, &card.key);
            }
        } else {
            player.inventory.push(card.key.clone());
        }
    }
}

pub fn update_shop_gold_system(
    player: Res<Player>,
    mut label_q: Query<&mut Text, With<ShopGoldLabel>>,
) {
    if player.is_changed() {
        for mut text in &mut label_q {
            **text = format!("{}", player.gold);
        }
    }
}

pub fn shop_tooltip_system(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    windows: Query<&Window>,
    hover_q: Query<(&Interaction, &ShopItemTooltip), Changed<Interaction>>,
    tooltip_node_q: Query<Entity, With<TooltipNode>>,
) {
    for (interaction, tooltip) in &hover_q {
        if *interaction == Interaction::Hovered {
            for entity in &tooltip_node_q {
                commands.entity(entity).try_despawn();
            }
            if let Some(equipment) = get_equipment(&tooltip.0) {
                let prefix = match equipment {
                    Equipment::Weapon(_) => "weapon",
                    Equipment::Wearable(_) => "wearable",
                };
                let lang = settings.language;
                let title = crate::core::ui::playing::name_with_level(
                    equipment.name(),
                    prefix,
                    equipment.level() as u8,
                    &localization,
                    lang,
                );
                let lines = equipment.full_description(lang, &localization);
                spawn_item_tooltip(
                    &mut commands,
                    &assets,
                    title,
                    lines,
                    &windows,
                    Some(equipment.price()),
                    Some(equipment.name().to_string()),
                );
            }
        } else if *interaction == Interaction::None {
            for entity in &tooltip_node_q {
                commands.entity(entity).try_despawn();
            }
        }
    }
}
