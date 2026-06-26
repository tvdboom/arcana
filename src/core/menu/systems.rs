use bevy::prelude::*;

use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::buttons::*;
use crate::core::menu::settings::{spawn_label, spawn_volume_slider, SettingsBtn};
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::player::Player;
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
#[derive(Message)]
pub struct StartNewCharacterMsg;

#[derive(Resource)]
pub struct PendingGameStart {
    pub target_game_state: GameState,
}

#[derive(Resource)]
pub struct LoadingDelay(pub Timer);

#[derive(Component)]
pub struct MenuContentRoot;

#[derive(Component)]
pub struct LoadingCmp;

#[derive(Component)]
pub struct LoadingText;

pub fn setup_menu(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
) {
    let lang = settings.language;
    let menu_bg_ready = asset_server.is_loaded_with_dependencies(assets.image("bg").id());
    commands
        .spawn((
            add_root_node(true),
            ImageNode::new(assets.image("bg")).with_mode(NodeImageMode::Stretch),
            MenuCmp,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        height: percent(100.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::top(percent(12.)),
                        ..default()
                    },
                    if menu_bg_ready {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    MenuContentRoot,
                ))
                .with_children(|parent| match app_state.get() {
                    AppState::MainMenu => {
                        spawn_menu_button(
                            parent,
                            MenuBtn::NewCharacter,
                            &assets,
                            &localization,
                            lang,
                        );
                        spawn_menu_button(
                            parent,
                            MenuBtn::LoadCharacter,
                            &assets,
                            &localization,
                            lang,
                        );
                        spawn_menu_button(parent, MenuBtn::Settings, &assets, &localization, lang);
                        #[cfg(not(target_arch = "wasm32"))]
                        spawn_menu_button(parent, MenuBtn::Quit, &assets, &localization, lang);
                    },
                    AppState::Settings => {
                        parent
                            .spawn((Node {
                                width: percent(40.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },))
                            .with_children(|parent| {
                                spawn_label(
                                    parent,
                                    "language",
                                    vec![
                                        SettingsBtn::English,
                                        SettingsBtn::Spanish,
                                        SettingsBtn::Dutch,
                                    ],
                                    &settings,
                                    &assets,
                                    &localization,
                                );
                                spawn_label(
                                    parent,
                                    "audio",
                                    vec![SettingsBtn::Mute, SettingsBtn::Sound, SettingsBtn::Music],
                                    &settings,
                                    &assets,
                                    &localization,
                                );
                                spawn_volume_slider(parent, &settings, &assets, &localization);
                                spawn_label(
                                    parent,
                                    "autosave",
                                    vec![SettingsBtn::True, SettingsBtn::False],
                                    &settings,
                                    &assets,
                                    &localization,
                                );
                            });

                        // Spacer to push the back button lower down
                        parent.spawn(Node {
                            height: Val::Px(50.),
                            ..default()
                        });

                        spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);
                    },
                    _ => (),
                });

            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: percent(3.),
                    bottom: percent(4.),
                    ..default()
                })
                .with_children(|parent| {
                    let credit = localization.get("created_by", lang);
                    parent.spawn((
                        add_text(credit, "medium", SUBTITLE_TEXT_SIZE, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        LocalizedText("created_by".to_string()),
                    ));
                });
        });
}

pub fn reveal_menu_content_when_bg_ready(
    asset_server: Res<AssetServer>,
    assets: Res<WorldAssets>,
    mut content_q: Query<&mut Visibility, With<MenuContentRoot>>,
) {
    if !asset_server.is_loaded_with_dependencies(assets.image("bg").id()) {
        return;
    }
    for mut visibility in &mut content_q {
        *visibility = Visibility::Inherited;
    }
}

pub fn setup_loading_screen(
    mut commands: Commands,
    assets: Res<WorldAssets>,
) {
    commands.insert_resource(LoadingDelay(Timer::from_seconds(0.35, TimerMode::Once)));
    commands
        .spawn((
            add_root_node(true),
            ImageNode::new(assets.image("bg")).with_mode(NodeImageMode::Stretch),
            LoadingCmp,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(8.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text("Loading", "bold", 4.0, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        LoadingText,
                    ));
                });
        });
}

pub fn animate_loading_text(
    time: Res<Time>,
    mut text_q: Query<&mut Text, With<LoadingText>>,
) {
    let dot_count = ((time.elapsed_secs() * 3.0) as usize) % 4;
    let dots = ".".repeat(dot_count);
    for mut text in &mut text_q {
        text.0 = format!("Loading{dots}");
    }
}

pub fn complete_loading_when_ready(
    mut commands: Commands,
    time: Res<Time>,
    mut delay: Option<ResMut<LoadingDelay>>,
    pending: Option<Res<PendingGameStart>>,
    assets: Res<WorldAssets>,
    asset_server: Res<AssetServer>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let Some(pending) = pending else {
        return;
    };

    if let Some(ref mut delay) = delay {
        delay.0.tick(time.delta());
        if !delay.0.is_finished() {
            return;
        }
    }

    let images_ready = assets
        .images
        .values()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));
    let fonts_ready = assets
        .fonts
        .values()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));
    let audio_ready = assets
        .audio
        .values()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));

    if !(images_ready && fonts_ready && audio_ready) {
        return;
    }

    next_game_state.set(pending.target_game_state);
    next_app_state.set(AppState::Game);
    commands.remove_resource::<PendingGameStart>();
    commands.remove_resource::<LoadingDelay>();
}

pub fn setup_game_menu(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
) {
    let lang = settings.language;
    commands
        .spawn((add_root_node(true), MenuCmp, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6))))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Vh(66.67),
                        height: Val::Vh(62.22),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Vh(2.78)),
                        ..default()
                    },
                    ImageNode::new(assets.image("banner_large")).with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    spawn_menu_button(parent, MenuBtn::Continue, &assets, &localization, lang);
                    #[cfg(not(target_arch = "wasm32"))]
                    spawn_menu_button(parent, MenuBtn::SaveCharacter, &assets, &localization, lang);
                    spawn_menu_button(parent, MenuBtn::Settings, &assets, &localization, lang);
                    spawn_menu_button(parent, MenuBtn::Quit, &assets, &localization, lang);
                });
        });
}

pub fn setup_game_settings(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
) {
    let lang = settings.language;
    commands
        .spawn((add_root_node(true), MenuCmp, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6))))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Vh(64.44),
                        height: Val::Vh(75.56),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Vh(2.78)),
                        ..default()
                    },
                    ImageNode::new(assets.image("banner_large")).with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    spawn_label(
                        parent,
                        "language",
                        vec![SettingsBtn::English, SettingsBtn::Spanish, SettingsBtn::Dutch],
                        &settings,
                        &assets,
                        &localization,
                    );
                    spawn_label(
                        parent,
                        "audio",
                        vec![SettingsBtn::Mute, SettingsBtn::Sound, SettingsBtn::Music],
                        &settings,
                        &assets,
                        &localization,
                    );
                    spawn_volume_slider(parent, &settings, &assets, &localization);
                    spawn_label(
                        parent,
                        "autosave",
                        vec![SettingsBtn::True, SettingsBtn::False],
                        &settings,
                        &assets,
                        &localization,
                    );

                    // Spacer to push the back button lower down
                    parent.spawn(Node {
                        height: Val::Vh(3.0),
                        ..default()
                    });

                    spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);
                });
        });
}

pub fn start_new_game_message(
    mut commands: Commands,
    mut start_new_char_msg: MessageReader<StartNewCharacterMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut player: ResMut<Player>,
) {
    if !start_new_char_msg.is_empty() {
        *player = Player::default();
        commands.insert_resource(PendingGameStart {
            target_game_state: GameState::CreateCharacter,
        });
        next_app_state.set(AppState::Loading);

        start_new_char_msg.clear();
    }
}
