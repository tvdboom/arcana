use crate::core::assets::WorldAssets;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::ui::playing::InfoTooltip;
use crate::core::ui::utils::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct QuestContentWrapper;

pub fn setup_quest_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }

    if let Some(container_entity) = columns_container_q.iter().next() {
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_quest");
        commands.entity(panel_entity).with_children(|parent| {
            build_quest_content(parent, &assets, &localization, settings.language, &player);
        });
    }
}

fn build_quest_content(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
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
            QuestContentWrapper,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    height: Val::Px(75.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Relative,
                    margin: UiRect::bottom(Val::Px(10.)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(30.),
                            ..default()
                        },
                        add_text(localization.get("quest", lang), "bold", 3.6, assets),
                        TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                    ));

                    parent
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                right: Val::Px(30.),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.),
                                ..default()
                            },
                            Interaction::default(),
                            Pickable::default(),
                            InfoTooltip::ActionPoints,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    width: Val::Vw(2.4),
                                    height: Val::Vw(2.4),
                                    ..default()
                                },
                                ImageNode::new(assets.image("ap"))
                                    .with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text(player.ap.to_string(), "bold", 2.4, assets),
                                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                            ));
                        });
                });

            parent
                .spawn(Node {
                    width: percent(100.),
                    height: percent(78.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.),
                    margin: UiRect::top(Val::Px(15.)),
                    ..default()
                })
                .with_children(|parent| {
                    let title1 = localization.get("errand_title", lang);
                    spawn_quest_card(
                        parent,
                        assets,
                        &title1,
                        "action_errand",
                        1,
                        false,
                        false,
                    );
                    let title2 = localization.get("expedition_title", lang);
                    spawn_quest_card(
                        parent,
                        assets,
                        &title2,
                        "action_expedition",
                        2,
                        true,
                        false,
                    );
                    let title3 = localization.get("odyssey_title", lang);
                    spawn_quest_card(
                        parent,
                        assets,
                        &title3,
                        "action_odyssey",
                        3,
                        true,
                        true,
                    );
                });
        });
}

fn spawn_quest_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    title: &str,
    image_key: &str,
    ap_cost: u32,
    show_mana: bool,
    show_health: bool,
) {
    parent
        .spawn((Node {
            width: percent(30.),
            height: percent(98.),
            position_type: PositionType::Relative,
            margin: UiRect::horizontal(percent(1.)),
            top: percent(-2.),
            ..default()
        }, BackgroundColor(crate::core::constants::NORMAL_BUTTON_COLOR)))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    height: percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::all(percent(1.5)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: percent(100.),
                            height: percent(50.),
                            ..default()
                        },
                        ImageNode::new(assets.image(image_key)).with_mode(NodeImageMode::Stretch),
                    ));

                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: percent(50.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::FlexStart,
                                ..default()
                            },
                            ImageNode::new(assets.image("stone"))
                                .with_mode(NodeImageMode::Stretch),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    margin: UiRect::vertical(percent(4.5)),
                                    ..default()
                                },
                                add_text(title, "bold", 2.2, assets),
                                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                            ));
                        });
                });

            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.),
                    right: Val::Vw(1.9),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.),
                    ..default()
                })
                .with_children(|parent| {
                    if show_mana {
                        spawn_cost_badge(parent, assets, "mana", "?");
                    }
                    if show_health {
                        spawn_cost_badge(parent, assets, "health", "?");
                    }
                    spawn_cost_badge(parent, assets, "ap", &ap_cost.to_string());
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(110.),
                        height: percent(110.),
                        left: percent(-5.),
                        top: percent(-5.),
                        ..default()
                    },
                    ImageNode::new(assets.image("border")).with_mode(NodeImageMode::Stretch),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                ))
                .observe(crate::core::menu::utils::reimage::<Over>(assets.image("border_hover")))
                .observe(crate::core::menu::utils::reimage::<Out>(assets.image("border")))
                .observe(crate::core::utils::cursor::<Over>(bevy::window::SystemCursorIcon::Pointer))
                .observe(crate::core::utils::cursor::<Out>(bevy::window::SystemCursorIcon::Default));
        });
}

fn spawn_cost_badge(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    icon_key: &str,
    label: &str,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.),
                padding: UiRect::axes(Val::Px(8.), Val::Px(4.)),
                border_radius: BorderRadius::all(Val::Px(6.)),
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(20.),
                    height: Val::Px(20.),
                    ..default()
                },
                ImageNode::new(assets.image(icon_key)).with_mode(NodeImageMode::Stretch),
            ));
            parent.spawn((
                add_text(label, "bold", 1.6, assets),
                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
            ));
        });
}
