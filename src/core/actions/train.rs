use crate::core::assets::WorldAssets;
use crate::core::ui::utils::*;
use bevy::prelude::*;

pub fn setup_train_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }

    if let Some(container_entity) = columns_container_q.iter().next() {
        // Just spawn base panel with train background image, and leave contents empty
        spawn_panel_base(&mut commands, &assets, container_entity, "bg_train");
    }
}
