use bevy::prelude::*;
use crate::core::assets::WorldAssets;
use crate::core::menu::utils::add_text;

#[derive(Component)]
pub struct GoldToast {
    pub timer: f32,
}

#[derive(Component)]
pub struct ToastContainer;

pub fn spawn_toast(
    commands: &mut Commands,
    assets: &WorldAssets,
    msg: String,
    bg: Color,
    border: Color,
    text_color: Color,
    container: Entity,
) {
    let toast_text = msg.clone();
    commands.entity(container).with_children(|parent| {
        parent
            .spawn((
                Node {
                    padding: UiRect::axes(Val::Px(14.), Val::Px(9.)),
                    border: UiRect::all(Val::Px(2.)),
                    border_radius: BorderRadius::all(Val::Px(8.)),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border),
                GoldToast { timer: 3.5 },
            ))
            .with_children(|parent| {
                parent.spawn((
                    add_text(toast_text, "bold", 2.2, assets),
                    TextColor(text_color),
                ));
            });
    });
}

pub fn tick_gold_toasts(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut GoldToast)>,
) {
    for (entity, mut toast) in &mut q {
        toast.timer -= time.delta_secs();
        if toast.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}
