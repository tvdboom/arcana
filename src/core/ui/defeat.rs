use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::{
    BUTTON_BORDER_COLOR, BUTTON_TEXT_COLOR, BUTTON_TEXT_SIZE, HOVERED_BUTTON_COLOR,
    NORMAL_BUTTON_COLOR, PRESSED_BUTTON_COLOR, SUBTITLE_TEXT_SIZE,
};
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::core::utils::cursor;

/// Marker for all entities belonging to the defeat ("severely injured") screen.
#[derive(Component)]
pub struct DefeatCmp;

/// Marker for the bottom-right continue button on the defeat screen.
#[derive(Component)]
pub struct DefeatContinueBtn;

/// Records the context of the last lost combat so the defeat screen can tailor
/// its message (PvP duels do not incur the action-point penalty).
#[derive(Resource, Default, Clone, Copy)]
pub struct DefeatContext {
    pub was_pvp: bool,
}

/// Flag requesting that the Rest panel be opened automatically the next time the
/// player returns to the Playing screen (used after acknowledging a defeat).
#[derive(Resource, Default)]
pub struct PendingAutoRest(pub bool);

pub fn manage_defeat_overlay(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    context: Option<Res<DefeatContext>>,
    defeat_q: Query<Entity, With<DefeatCmp>>,
) {
    if let Some(ctx) = context {
        if defeat_q.is_empty() {
            setup_defeat_screen_inner(&mut commands, &assets, &localization, &settings, *ctx);
        }
    } else {
        for entity in &defeat_q {
            commands.entity(entity).try_despawn();
        }
    }
}

fn setup_defeat_screen_inner(
    commands: &mut Commands,
    assets: &WorldAssets,
    localization: &Localization,
    settings: &Settings,
    context: DefeatContext,
) {
    let lang = settings.language;
    let was_pvp = context.was_pvp;
    let body_key = if was_pvp {
        "general.defeat_screen_body_pvp"
    } else {
        "general.defeat_screen_body"
    };

    let (root_node, _) = add_root_node(true);
    commands
        .spawn((
            root_node,
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
            DefeatCmp,
            GlobalZIndex(1000),
            ImageNode::new(assets.image("defeat")).with_mode(NodeImageMode::Stretch),
        ))
        .with_children(|parent| {
            // Centered title + explanatory text over a darkened panel.
            parent
                .spawn((
                    Node {
                        width: Val::Vw(60.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Vh(3.),
                        padding: UiRect::all(Val::Vh(4.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            localization.get("general.severely_injured", lang),
                            "bold",
                            SUBTITLE_TEXT_SIZE,
                            assets,
                        ),
                        TextColor(Color::srgb_u8(200, 45, 45)),
                        TextLayout::justify(Justify::Center),
                    ));
                    parent.spawn((
                        add_text(localization.get(body_key, lang), "medium", 2.6, assets),
                        TextColor(Color::srgb_u8(230, 220, 200)),
                        TextLayout::justify(Justify::Center),
                    ));
                });

            // Continue button anchored to the bottom-right corner, moved inwards (8% right/bottom).
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Vw(8.),
                        bottom: Val::Vh(8.),
                        padding: UiRect::axes(Val::Vw(2.), Val::Vh(1.5)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.)),
                        border_radius: BorderRadius::all(Val::Px(4.)),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    Button,
                    DefeatContinueBtn,
                ))
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
                .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .observe(cursor::<Release>(SystemCursorIcon::Default))
                .observe(handle_defeat_continue_click)
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            localization.get("general.continue", lang),
                            "bold",
                            BUTTON_TEXT_SIZE,
                            assets,
                        ),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });
        });
}

pub fn handle_defeat_continue_click(
    _event: On<Pointer<Click>>,
    mut commands: Commands,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut pending_auto_rest: ResMut<PendingAutoRest>,
) {
    // Any lingering duel networking is torn down before returning to town.
    crate::core::network::teardown_duel(&mut commands);
    commands.remove_resource::<DefeatContext>();
    pending_auto_rest.0 = true;
    play_audio_msg.write(PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}

/// Listens for Enter or Escape keys while defeat is active to act as if Continue was clicked.
pub fn handle_defeat_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut pending_auto_rest: ResMut<PendingAutoRest>,
    context: Option<Res<DefeatContext>>,
) {
    if context.is_none() {
        return;
    }
    if keyboard.just_released(KeyCode::Enter)
        || keyboard.just_released(KeyCode::NumpadEnter)
        || keyboard.just_released(KeyCode::Escape)
    {
        crate::core::network::teardown_duel(&mut commands);
        commands.remove_resource::<DefeatContext>();
        pending_auto_rest.0 = true;
        play_audio_msg.write(PlayAudioMsg::new("button"));
        next_game_state.set(GameState::Playing);
    }
}

/// When returning to the Playing screen after a defeat, automatically open the
/// Rest panel so the player can recover before doing anything else.
pub fn auto_open_rest_after_defeat(
    mut pending_auto_rest: ResMut<PendingAutoRest>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if pending_auto_rest.0 {
        pending_auto_rest.0 = false;
        next_game_state.set(GameState::Rest);
    }
}

