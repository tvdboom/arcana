use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_root_node;
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::core::ui::button::spawn_action_button;
use bevy::prelude::*;

#[derive(Component)]
pub struct CombatCmp;

pub fn setup_combat_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    play_audio_msg.write(PlayAudioMsg::new("horn"));

    let lang = settings.language;
    let (mut root_node, pickable) = add_root_node(true);
    root_node.padding = UiRect::all(Val::Px(0.));

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode {
                image: assets.image("bg_combat"),
                image_mode: NodeImageMode::Stretch,
                color: Color::srgba(0.40, 0.40, 0.40, 1.0),
                ..default()
            },
            GlobalZIndex(980),
            CombatCmp,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(5.),
                    bottom: Val::Percent(5.),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_action_button(&mut *parent, &assets, localization.get("general.forfeit_combat", lang))
                        .observe(handle_forfeit_combat_click);
                });
        });
}

pub fn handle_forfeit_combat_click(
    _event: On<Pointer<Click>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    play_audio_msg.write(PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}

