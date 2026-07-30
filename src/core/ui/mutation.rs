//! Post-combat mutation offer presentation and choice handling.

use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use rand::{rng, RngExt};

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::{
    BUTTON_BORDER_COLOR, BUTTON_TEXT_COLOR, BUTTON_TEXT_SIZE, HOVERED_BUTTON_COLOR,
    NORMAL_BUTTON_COLOR, PRESSED_BUTTON_COLOR, SUBTITLE_TEXT_SIZE,
};
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::player::Player;
use crate::core::races::Mutation;
use crate::core::settings::Settings;
use crate::core::ui::utils::ResponsiveOverlayCard;
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;

/// The mutation that an eligible completed combat is offering to the player.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingMutationOffer(pub Mutation);

/// Marker for all entities belonging to the mutation choice overlay.
#[derive(Component)]
pub struct MutationCmp;

/// The decision represented by one mutation choice button.
#[derive(Component, Clone, Copy)]
pub enum MutationChoiceBtn {
    Accept,
    Reject,
}

const TEST_MUTATIONS: [Mutation; 5] = [
    Mutation::Werewolf,
    Mutation::Wererat,
    Mutation::Werebear,
    Mutation::Vampire,
    Mutation::Undead,
];

/// Opens a uniformly random mutation offer when Ctrl+Shift+M is pressed.
pub fn open_random_mutation_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    offer: Option<Res<PendingMutationOffer>>,
    mut commands: Commands,
) {
    let control_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    if offer.is_some()
        || !control_pressed
        || !shift_pressed
        || !keyboard.just_pressed(KeyCode::KeyM)
    {
        return;
    }

    let index = rng().random_range(0..TEST_MUTATIONS.len());
    commands.insert_resource(PendingMutationOffer(test_mutation_at(index)));
}

/// Returns the mutation at a wrapped test-selection index.
fn test_mutation_at(index: usize) -> Mutation {
    TEST_MUTATIONS[index % TEST_MUTATIONS.len()]
}

/// Creates or removes the full-screen mutation overlay as its pending offer changes.
pub fn manage_mutation_overlay(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    offer: Option<Res<PendingMutationOffer>>,
    overlay_q: Query<Entity, With<MutationCmp>>,
) {
    let Some(offer) = offer else {
        for entity in &overlay_q {
            commands.entity(entity).try_despawn();
        }
        return;
    };
    if !overlay_q.is_empty() {
        return;
    }

    setup_mutation_overlay(&mut commands, &assets, &localization, &settings, offer.0);
}

/// Builds the full-screen mutation decision in the same visual language as defeat.
fn setup_mutation_overlay(
    commands: &mut Commands,
    assets: &WorldAssets,
    localization: &Localization,
    settings: &Settings,
    mutation: Mutation,
) {
    let lang = settings.language;
    let mutation_key = mutation.to_lowername();
    let mutation_name = localization.get(format!("mutation.{mutation_key}"), lang);
    let body =
        localization.get("general.mutation_offer_body", lang).replace("{mutation}", &mutation_name);
    let effect = localization.get(format!("mutation.{mutation_key}_effect"), lang);
    let (root_node, _) = add_root_node(true);

    commands
        .spawn((
            root_node,
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            },
            MutationCmp,
            GlobalZIndex(1010),
            ImageNode::new(assets.image("bg_mutation")).with_mode(NodeImageMode::Stretch),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Vw(64.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::VMin(3.),
                        padding: UiRect::all(Val::VMin(4.)),
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ResponsiveOverlayCard {
                        desktop_width: Val::Vw(64.),
                        desktop_height: Val::Auto,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            localization.get("general.mutation", lang),
                            "bold",
                            SUBTITLE_TEXT_SIZE,
                            assets,
                        ),
                        TextColor(Color::srgb_u8(155, 70, 190)),
                        TextLayout::justify(Justify::Center),
                    ));
                    parent.spawn((
                        add_text(format!("{body}\n\n{effect}"), "medium", 2.6, assets),
                        TextColor(Color::srgb_u8(230, 220, 200)),
                        TextLayout::justify(Justify::Center),
                    ));
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Vw(3.),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_choice_button(
                                parent,
                                assets,
                                localization.get("general.accept_mutation", lang),
                                MutationChoiceBtn::Accept,
                            );
                            spawn_choice_button(
                                parent,
                                assets,
                                localization.get("general.reject_mutation", lang),
                                MutationChoiceBtn::Reject,
                            );
                        });
                });
        });
}

/// Spawns one styled mutation choice button.
fn spawn_choice_button(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    label: String,
    choice: MutationChoiceBtn,
) {
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Vw(2.5), Val::VMin(1.5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.)),
                border_radius: BorderRadius::all(Val::Px(4.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            Button,
            choice,
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(cursor::<Release>(SystemCursorIcon::Default))
        .observe(handle_mutation_choice_click)
        .with_children(|parent| {
            parent.spawn((
                add_text(label, "bold", BUTTON_TEXT_SIZE, assets),
                TextColor(BUTTON_TEXT_COLOR),
            ));
        });
}

/// Applies or rejects the mutation selected with a pointer.
pub fn handle_mutation_choice_click(
    event: On<Pointer<Click>>,
    choice_q: Query<&MutationChoiceBtn>,
    offer: Option<Res<PendingMutationOffer>>,
    mut commands: Commands,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    let (Ok(choice), Some(offer)) = (choice_q.get(event.entity), offer) else {
        return;
    };
    resolve_mutation_choice(
        &mut commands,
        &mut player,
        &mut play_audio_msg,
        offer.0,
        matches!(choice, MutationChoiceBtn::Accept),
    );
}

/// Supports Y/Enter to accept and N/Escape to reject the active mutation offer.
pub fn handle_mutation_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    offer: Option<Res<PendingMutationOffer>>,
    mut commands: Commands,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    let Some(offer) = offer else {
        return;
    };
    let accept = keyboard.just_released(KeyCode::KeyY)
        || keyboard.just_released(KeyCode::Enter)
        || keyboard.just_released(KeyCode::NumpadEnter);
    let reject = keyboard.just_released(KeyCode::KeyN) || keyboard.just_released(KeyCode::Escape);
    if accept || reject {
        resolve_mutation_choice(&mut commands, &mut player, &mut play_audio_msg, offer.0, accept);
    }
}

/// Resolves one mutation decision while keeping current health and mana valid.
fn resolve_mutation_choice(
    commands: &mut Commands,
    player: &mut Player,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
    mutation: Mutation,
    accept: bool,
) {
    if accept {
        let old_max_health = player.max_health();
        let old_max_mana = player.max_mana();
        player.mutation = Some(mutation);
        player.update_health_mana(old_max_health, old_max_mana);
    }
    commands.remove_resource::<PendingMutationOffer>();
    play_audio_msg.write(PlayAudioMsg::new("button"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Verifies the testing shortcut's random range covers every mutation exactly once.
    fn testing_mutation_range_contains_every_form() {
        assert_eq!(test_mutation_at(0), Mutation::Werewolf);
        assert_eq!(test_mutation_at(1), Mutation::Wererat);
        assert_eq!(test_mutation_at(2), Mutation::Werebear);
        assert_eq!(test_mutation_at(3), Mutation::Vampire);
        assert_eq!(test_mutation_at(4), Mutation::Undead);
        assert_eq!(test_mutation_at(5), Mutation::Werewolf);
    }
}
