use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::{all_equipment, get_equipment};
use crate::core::catalog::equipment::{Equipment, Kind};
use crate::core::catalog::weapons::{Category, Hand};
use crate::core::catalog::wearables::WearableSlot;
use crate::core::constants::*;
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_text, recolor};
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::ui::dropdown::{
    spawn_dropdown_category, spawn_dropdown_hand, spawn_dropdown_kind, spawn_dropdown_type,
    OpenDropdown,
};
use crate::core::ui::playing::name_with_level;
use crate::core::ui::scrollbar::{
    on_scrollbar_thumb_drag, ScrollableContainer, ScrollbarThumb, ScrollbarTrack,
};
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::core::ui::tooltip::{spawn_item_tooltip, TooltipNode};
use crate::core::ui::utils::*;
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Component)]
pub struct ShopScrollContainer;

#[derive(Component)]
pub struct ShopItemsScroll;

#[derive(Resource, Clone, Serialize, Deserialize, Default, Debug)]
pub struct ShopInventory {
    pub items: Vec<String>,
    #[serde(default)]
    pub allowed_weapons: Vec<String>,
    #[serde(default)]
    pub allowed_artifacts: Vec<String>,
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
    Artifacts,
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

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ShopTabClickGuard {
    pub suppress_next_item_click: bool,
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

#[allow(dead_code)]
pub fn generate_deterministic_shop(player_name: &str, player_level: u32) -> Vec<String> {
    let mut hasher = DefaultHasher::new();
    player_name.hash(&mut hasher);
    let seed = hasher.finish();
    let mut rng = DeterministicRng::new(seed);

    let mut items = Vec::new();
    for eq in all_equipment() {
        if matches!(eq, Equipment::Artifact(_)) {
            continue;
        }
        if eq.level() >= 1 && eq.level() <= player_level
            && rng.random_bool(0.5) {
                items.push(eq.name().to_string());
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

    let player_level = player.level();

    if shop_inventory.allowed_weapons.is_empty() {
        let mut hasher = DefaultHasher::new();
        player.name.hash(&mut hasher);
        let seed = hasher.finish();
        let mut rng = DeterministicRng::new(seed);

        let mut allowed = Vec::new();
        for eq in all_equipment() {
            if matches!(eq, Equipment::Artifact(_)) {
                continue;
            }
            if rng.random_bool(0.5) {
                allowed.push(eq.name().to_string());
            }
        }
        shop_inventory.allowed_weapons = allowed;
    }

    if shop_inventory.allowed_artifacts.is_empty() {
        let mut hasher = DefaultHasher::new();
        player.name.hash(&mut hasher);
        "artifacts".hash(&mut hasher);
        let seed = hasher.finish();
        let mut rng = DeterministicRng::new(seed);

        let mut allowed = Vec::new();
        for art in crate::core::catalog::catalog::all_artifacts() {
            if rng.random_bool(0.5) {
                allowed.push(art.name.clone());
            }
        }
        shop_inventory.allowed_artifacts = allowed;
    }

    shop_inventory.items = shop_inventory
        .allowed_weapons
        .iter()
        .filter_map(|name| {
            if let Some(eq) = get_equipment(name) {
                if eq.level() >= 1 && eq.level() <= player_level {
                    return Some(name.clone());
                }
            }
            None
        })
        .collect();

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
                player.level(),
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
    open_dropdown: Res<OpenDropdown>,
    wrapper_q: Query<Entity, With<ShopContentWrapper>>,
    children_q: Query<&Children>,
    scroll_q: Query<&ScrollPosition, With<ShopItemsScroll>>,
) {
    if filters.is_changed() || shop_inventory.is_changed() || open_dropdown.is_changed() {
        let current_scroll = scroll_q.iter().next().map_or(0.0, |s| s.0.y);
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
                    player.level(),
                    *open_dropdown,
                    current_scroll,
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
    player_level: u32,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                height: percent(100.),
                padding: UiRect::all(percent(4.)),
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
                        player_level,
                        OpenDropdown::None,
                        0.0,
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
    player_level: u32,
    open_dropdown: OpenDropdown,
    initial_scroll: f32,
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
                    margin: UiRect {
                        left: Val::Px(15.),
                        top: Val::Px(15.),
                        ..default()
                    },
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
                        ShopTab::Artifacts,
                    ] {
                        parent
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                                    border: UiRect::all(Val::Px(1.)),
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON_COLOR),
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
                                    add_text(tab.to_name(), "bold", 1.8, assets),
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
                        margin: UiRect {
                            right: Val::Px(45.),
                            top: Val::Px(15.),
                            ..default()
                        },
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
                        add_text(player_gold.to_string(), "bold", 3.0, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        ShopGoldLabel,
                    ));
                });
        });

    parent.spawn((
        Node {
            width: percent(100.),
            height: Val::Px(1.),
            margin: UiRect {
                top: Val::Px(10.),
                bottom: Val::Px(18.),
                ..default()
            },
            ..default()
        },
        BackgroundColor(BUTTON_BORDER_COLOR),
    ));

    let mut container_entity = Entity::PLACEHOLDER;
    parent
        .spawn((
            Node {
                width: percent(100.),
                flex_grow: 1.0,
                height: percent(70.),
                min_height: Val::Px(0.),
                flex_direction: FlexDirection::Row,
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            },
            ShopScrollContainer,
        ))
        .with_children(|parent| {
            let mut container_cmd = parent.spawn((
                Node {
                    width: percent(85.),
                    height: percent(100.),
                    min_height: Val::Px(0.),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.),
                    padding: UiRect::all(Val::Px(15.)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollableContainer,
                ScrollPosition(Vec2::new(0., initial_scroll)),
                ShopItemsScroll,
                Interaction::default(),
                bevy::ui::RelativeCursorPosition::default(),
            ));
            container_entity = container_cmd.id();
            container_cmd.with_children(|parent| {
                let mut matching: Vec<Equipment> = Vec::new();
                if filters.tab == ShopTab::Artifacts {
                    let mut seen = std::collections::HashSet::new();
                    for artifact in crate::core::catalog::catalog::all_artifacts() {
                        if artifact.level <= player_level
                            && filters.kind.is_none_or(|k| artifact.kind == k)
                            && shop_inventory.allowed_artifacts.contains(&artifact.name)
                            && seen.insert(artifact.name.clone()) {
                                matching.push(Equipment::Artifact(artifact.clone()));
                            }
                    }
                } else {
                    for item_key in &shop_inventory.items {
                        if let Some(equipment) = get_equipment(item_key) {
                            let matches_tab = match filters.tab {
                                ShopTab::Weapons => match &equipment {
                                    Equipment::Weapon(w) => {
                                        let matches_hand =
                                            filters.weapon_hand.is_none_or(|h| w.hand == h);
                                        let matches_type = match filters.weapon_type {
                                            WeaponTypeFilter::All => true,
                                            WeaponTypeFilter::Weapons => {
                                                w.category != Category::Shield
                                                    && w.category != Category::Book
                                            },
                                            WeaponTypeFilter::Shields => {
                                                w.category == Category::Shield
                                            },
                                            WeaponTypeFilter::Books => w.category == Category::Book,
                                        };
                                        let matches_category = filters
                                            .weapon_category
                                            .is_none_or(|c| w.category == c);
                                        let matches_kind =
                                            filters.kind.is_none_or(|k| w.kind == k);
                                        matches_hand
                                            && matches_type
                                            && matches_category
                                            && matches_kind
                                    },
                                    _ => false,
                                },
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
                                ShopTab::Consumables => {
                                    matches!(equipment, Equipment::Consumable(_))
                                },
                                ShopTab::Artifacts => false,
                            };
                            if matches_tab {
                                matching.push(equipment);
                            }
                        }
                    }
                }

                matching.sort_by_key(|b| std::cmp::Reverse(b.price()));

                for chunk in matching.chunks(4) {
                    parent
                        .spawn(Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Row,
                            flex_shrink: 0.,
                            align_items: AlignItems::FlexStart,
                            column_gap: percent(1.5),
                            overflow: Overflow::clip(),
                            ..default()
                        })
                        .with_children(|parent| {
                            for equipment in chunk {
                                spawn_shop_item_card(parent, assets, localization, lang, equipment);
                            }
                        });
                }

                if matching.is_empty() {
                    parent.spawn((
                        add_text("No items available.", "bold", 2.0, assets),
                        TextColor(Color::WHITE),
                    ));
                }
            });

            // Sidebar for weapon filters
            parent
                .spawn(Node {
                    width: percent(11.),
                    height: percent(100.),
                    margin: UiRect {
                        left: Val::Px(5.),
                        right: Val::Px(10.),
                        ..default()
                    },
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.),
                    align_items: AlignItems::Stretch,
                    ..default()
                })
                .with_children(|parent| {
                    if filters.tab == ShopTab::Weapons {
                        spawn_dropdown_type(parent, assets, filters.weapon_type, open_dropdown);
                        spawn_dropdown_kind(parent, assets, filters.kind, open_dropdown);
                        spawn_dropdown_category(
                            parent,
                            assets,
                            filters.weapon_category,
                            open_dropdown,
                        );
                        spawn_dropdown_hand(parent, assets, filters.weapon_hand, open_dropdown);
                    } else if filters.tab == ShopTab::Artifacts {
                        spawn_dropdown_kind(parent, assets, filters.kind, open_dropdown);
                    }
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(10.),
                        top: Val::Px(0.),
                        bottom: Val::Px(0.),
                        right: Val::Px(0.),
                        border_radius: BorderRadius::all(Val::Px(5.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba_u8(0, 0, 0, 170)),
                    Visibility::Hidden,
                    ScrollbarTrack {
                        container: container_entity,
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: percent(100.),
                                height: Val::Px(32.),
                                top: Val::Px(0.),
                                border_radius: BorderRadius::all(Val::Px(5.)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba_u8(230, 205, 120, 240)),
                            Button,
                            Interaction::default(),
                            Pickable::default(),
                            ScrollbarThumb {
                                container: container_entity,
                            },
                        ))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(on_scrollbar_thumb_drag);
                });
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
    let name = name_with_level(
        item.name(),
        item.to_lowername().as_str(),
        item.level() as u8,
        localization,
        lang,
    );

    parent
        .spawn((
            Node {
                width: percent(23.0),
                aspect_ratio: Some(1.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::FlexEnd,
                border: UiRect::all(Val::Px(1.5)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
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
        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(handle_shop_item_card_click)
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.),
                    height: percent(100.),
                    ..default()
                },
                ImageNode::new(assets.image(format!("build_{}", item.name())))
                    .with_mode(NodeImageMode::Stretch),
            ));

            parent
                .spawn(Node {
                    width: percent(100.),
                    height: Val::Px(34.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(4.)),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgba_u8(0, 0, 0, 180)))
                .with_children(|parent| {
                    parent
                        .spawn((add_text(name, "bold", 1.5, assets), TextColor(BUTTON_TEXT_COLOR)));

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
                                    width: Val::Px(16.),
                                    height: Val::Px(16.),
                                    ..default()
                                },
                                ImageNode::new(assets.image("gold"))
                                    .with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text(format!("{}", item.price()), "bold", 1.5, assets),
                                TextColor(BUTTON_TEXT_COLOR),
                            ));
                        });
                });
        });
}

#[derive(Component, Clone, Copy)]
pub struct ShopTabButton(pub ShopTab);

pub fn handle_shop_tab_click(
    event: On<Pointer<Click>>,
    mut filters: ResMut<ShopFilters>,
    mut tab_click_guard: ResMut<ShopTabClickGuard>,
    btn_q: Query<&ShopTabButton>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        let clicking_current_tab = filters.tab == btn.0;
        if filters.tab != btn.0 {
            filters.kind = None;
            filters.weapon_type = WeaponTypeFilter::All;
            filters.weapon_category = None;
            filters.weapon_hand = None;
        }
        filters.tab = btn.0;
        tab_click_guard.suppress_next_item_click = clicking_current_tab;
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
}

pub fn handle_shop_item_card_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut tab_click_guard: ResMut<ShopTabClickGuard>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    card_q: Query<&ShopItemCard>,
    scroll_q: Query<&bevy::ui::RelativeCursorPosition, With<ShopItemsScroll>>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
) {
    if tab_click_guard.suppress_next_item_click {
        tab_click_guard.suppress_next_item_click = false;
        return;
    }

    // Reject clicks whose cursor isn't actually over the visible (non-clipped)
    // scroll viewport. Scrolled-out cards can remain pickable at screen positions
    // outside the container (e.g. over the tabs), which would otherwise buy them.
    if let Some(rel) = scroll_q.iter().next() {
        if !rel.cursor_over() {
            return;
        }
    }

    let Ok(card) = card_q.get(event.entity) else {
        return;
    };

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
        let auto_equip = get_equipment(&card.key)
            .map(|eq| !matches!(eq, Equipment::Consumable(_) | Equipment::Artifact(_)))
            .unwrap_or(false);
        if auto_equip {
            crate::core::ui::playing::equip_item(&mut player, &card.key);
        }
    } else {
        player.inventory.push(card.key.clone());
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
    card_q: Query<(&Interaction, &ShopItemTooltip)>,
    changed_card_q: Query<(), (With<ShopItemTooltip>, Changed<Interaction>)>,
    tooltip_node_q: Query<Entity, With<TooltipNode>>,
) {
    if changed_card_q.is_empty() {
        return;
    }

    let mut hovered_item = None;
    for (interaction, tooltip) in &card_q {
        if *interaction == Interaction::Hovered {
            hovered_item = Some(tooltip.0.clone());
            break;
        }
    }

    for entity in &tooltip_node_q {
        commands.entity(entity).try_despawn();
    }

    if let Some(item_key) = hovered_item {
        if let Some(equipment) = get_equipment(&item_key) {
            let lang = settings.language;
            let title = name_with_level(
                equipment.name(),
                equipment.to_lowername().as_str(),
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
                if matches!(equipment, Equipment::Artifact(_)) {
                    64.0
                } else {
                    0.0
                },
            );
        }
    }
}

pub fn shop_tab_button_system(
    filters: Res<ShopFilters>,
    mut tab_btn_q: Query<(Entity, &ShopTabButton, &Interaction, &mut BackgroundColor)>,
    children_q: Query<&Children>,
    mut text_color_q: Query<&mut TextColor>,
) {
    for (entity, btn, interaction, mut bg) in &mut tab_btn_q {
        let active = btn.0 == filters.tab;
        *bg = match (active, interaction) {
            (_, Interaction::Pressed) => BackgroundColor(Color::srgba_u8(30, 30, 50, 240)),
            (true, Interaction::Hovered) => BackgroundColor(NORMAL_BUTTON_COLOR),
            (false, Interaction::Hovered) => BackgroundColor(BUTTON_TEXT_COLOR),
            (true, Interaction::None) => BackgroundColor(NORMAL_BUTTON_COLOR),
            (false, Interaction::None) => BackgroundColor(Color::srgba_u8(12, 12, 18, 240)),
        };

        if let Ok(children) = children_q.get(entity) {
            for child in children.iter() {
                if let Ok(mut txt_col) = text_color_q.get_mut(child) {
                    txt_col.0 = match (active, interaction) {
                        (_, Interaction::Pressed) => Color::srgba(1.0, 1.0, 1.0, 0.4),
                        (true, _) => BUTTON_TEXT_COLOR,
                        (false, Interaction::Hovered) => Color::BLACK,
                        (false, Interaction::None) => Color::srgba(1.0, 1.0, 1.0, 0.4),
                    };
                }
            }
        }
    }
}
