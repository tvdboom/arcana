use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::modal::ActiveModal;
use crate::core::actions::shop::{ShopInventory, ShopFilters};
use crate::core::ui::dropdown::OpenDropdown;
use crate::core::actions::work::WorkSliderState;
use crate::core::actions::study::StudySliderState;
use crate::core::actions::train::TrainSliderState;
use crate::core::ui::playing::RightTab;

/// A bundled system parameter that provides access to all UI-related game state resources.
/// This allows systems to query a single `GameState` struct instead of listing many individual resources.
#[derive(SystemParam)]
#[allow(dead_code)]
pub struct GameState<'w> {
    pub level_up_pending: ResMut<'w, LevelUpPending>,
    pub active_modal: ResMut<'w, ActiveModal>,
    pub shop_inventory: ResMut<'w, ShopInventory>,
    pub open_dropdown: ResMut<'w, OpenDropdown>,
    pub shop_filters: ResMut<'w, ShopFilters>,
    pub work_slider_state: ResMut<'w, WorkSliderState>,
    pub study_slider_state: ResMut<'w, StudySliderState>,
    pub train_slider_state: ResMut<'w, TrainSliderState>,
    pub right_tab: ResMut<'w, RightTab>,
}
