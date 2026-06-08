use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::systems::StartNewCharacterMsg;
use crate::core::menu::utils::{add_text, recolor};
#[cfg(not(target_arch = "wasm32"))]
use crate::core::persistence::{LoadCharacterMsg, SaveCharacterMsg};
use crate::core::settings::Language;
use crate::core::states::{AppState, GameState};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

#[derive(Component)]
pub struct MenuCmp;

#[derive(Component, Clone, Debug, PartialEq)]
pub enum MenuBtn {
    NewCharacter,
    #[cfg(not(target_arch = "wasm32"))]
    LoadCharacter,
    Back,
    Continue,
    #[cfg(not(target_arch = "wasm32"))]
    SaveCharacter,
    Settings,
    Quit,
}

#[derive(Component)]
pub struct DisabledButton;

pub fn on_click_menu_button(
    event: On<Pointer<Click>>,
    btn_q: Query<(Option<&DisabledButton>, &MenuBtn)>,
    mut start_new_char_msg: MessageWriter<StartNewCharacterMsg>,
    #[cfg(not(target_arch = "wasm32"))] mut load_game_msg: MessageWriter<LoadCharacterMsg>,
    #[cfg(not(target_arch = "wasm32"))] mut save_game_msg: MessageWriter<SaveCharacterMsg>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    let (disabled, btn) = btn_q.get(event.entity).unwrap();

    if disabled.is_some() {
        return;
    }

    play_audio_msg.write(PlayAudioMsg::new("button"));

    match btn {
        MenuBtn::NewCharacter => {
            start_new_char_msg.write(StartNewCharacterMsg);
        },
        #[cfg(not(target_arch = "wasm32"))]
        MenuBtn::LoadCharacter => {
            load_game_msg.write(LoadCharacterMsg);
        },
        MenuBtn::Back => match *app_state.get() {
            AppState::Settings => next_app_state.set(AppState::MainMenu),
            AppState::Game => match *game_state.get() {
                GameState::CreateCharacter => {
                    next_app_state.set(AppState::MainMenu);
                },
                GameState::ChooseRace => {
                    next_game_state.set(GameState::CreateCharacter);
                },
                GameState::ChooseClass => {
                    next_game_state.set(GameState::ChooseRace);
                },
                GameState::ChooseSubClass => {
                    next_game_state.set(GameState::ChooseClass);
                },
                GameState::Settings => {
                    next_game_state.set(GameState::GameMenu);
                },
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        MenuBtn::Continue => {
            next_game_state.set(GameState::Playing);
        },
        #[cfg(not(target_arch = "wasm32"))]
        MenuBtn::SaveCharacter => {
            save_game_msg.write(SaveCharacterMsg(false));
        },
        MenuBtn::Settings => {
            if *game_state.get() == GameState::GameMenu {
                next_game_state.set(GameState::Settings);
            } else {
                next_app_state.set(AppState::Settings);
            }
        },
        MenuBtn::Quit => match *app_state.get() {
            AppState::Game => {
                next_game_state.set(GameState::default());
                next_app_state.set(AppState::MainMenu)
            },
            AppState::MainMenu => std::process::exit(0),
            _ => unreachable!(),
        },
    }
}

pub fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    btn: MenuBtn,
    assets: &WorldAssets,
    localization: &Localization,
    language: Language,
) {
    let key = btn.to_lowername();
    let label = localization.get(&key, language);

    let (width, height) = match btn {
        MenuBtn::Back => (Val::Vh(22.22), Val::Vh(5.0)),
        MenuBtn::NewCharacter
        | MenuBtn::LoadCharacter
        | MenuBtn::Settings
        | MenuBtn::Quit
        | MenuBtn::Continue
        | MenuBtn::SaveCharacter => (Val::Vh(46.67), Val::Vh(8.33)),
        #[allow(unreachable_patterns)]
        _ => (Val::Vh(33.33), Val::Vh(6.11)),
    };

    let margin = UiRect::all(Val::Vh(0.89));

    parent
        .spawn((
            Node {
                width,
                height,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin,
                border: UiRect::all(Val::Vh(0.22)),
                border_radius: BorderRadius::all(Val::Vh(0.44)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            btn.clone(),
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(cursor::<Release>(SystemCursorIcon::Default))
        .observe(on_click_menu_button)
        .with_children(|parent| {
            parent.spawn((
                add_text(label, "bold", BUTTON_TEXT_SIZE, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(key),
            ));
        });
}
