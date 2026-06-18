use crate::core::assets::WorldAssets;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::ui::playing::InfoTooltip;
use crate::core::ui::utils::*;
use bevy::prelude::*;

pub fn setup_duel_ui(
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
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_duel");
        commands.entity(panel_entity).with_children(|parent| {
            build_action_panel(
                parent,
                &assets,
                &localization,
                settings.language,
                &player,
                "duel",
                "action_duel",
                "duel_desc",
            );
        });
    }
}

fn build_action_panel(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    title_key: &str,
    image_key: &str,
    desc_key: &str,
) {
    parent
        .spawn(Node {
            width: percent(100.),
            height: percent(100.),
            padding: UiRect::all(percent(5.)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
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
                        add_text(localization.get(title_key, lang), "bold", 3.6, assets),
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
                .spawn((
                    Node {
                        width: percent(60.),
                        height: percent(42.),
                        align_self: AlignSelf::Center,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(12.),
                        padding: UiRect::all(percent(3.)),
                        ..default()
                    },
                    ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(120.),
                            height: Val::Px(120.),
                            ..default()
                        },
                        ImageNode::new(assets.image(image_key)).with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(localization.get(desc_key, lang), "medium", 2.2, assets),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}
