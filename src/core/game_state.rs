use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::actions::shop::{ShopFilters, ShopInventory, ShopTab, WeaponTypeFilter};
use crate::core::actions::study::StudySliderState;
use crate::core::actions::train::TrainSliderState;
use crate::core::actions::work::WorkSliderState;
use crate::core::catalog::equipment::Kind;
use crate::core::catalog::weapons::{Category, Hand};
use crate::core::ui::dropdown::OpenDropdown;
use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::modal::ActiveModal;
use crate::core::ui::playing::RightTab;

/// A bundled system parameter that provides access to all UI-related game state resources.
/// This allows systems to query a single `GameState` struct instead of listing many individual resources.
#[derive(SystemParam)]
#[allow(dead_code)]
pub struct GameState<'w> {
    pub level_up_pending: ResMut<'w, LevelUpPending>,
    pub active_modal: ResMut<'w, ActiveModal>,
    pub shop_inventory: ResMut<'w, ShopInventory>,
    pub shop_ui_state: ResMut<'w, ShopUiState>,
    pub open_dropdown: ResMut<'w, OpenDropdown>,
    pub work_slider_state: ResMut<'w, WorkSliderState>,
    pub study_slider_state: ResMut<'w, StudySliderState>,
    pub train_slider_state: ResMut<'w, TrainSliderState>,
    pub right_tab: ResMut<'w, RightTab>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShopTabState {
    pub weapon_hand: Option<Hand>,
    pub weapon_type: WeaponTypeFilter,
    pub weapon_category: Option<Category>,
    pub kind: Option<Kind>,
    pub scroll_y: f32,
}

impl Default for ShopTabState {
    fn default() -> Self {
        Self {
            weapon_hand: None,
            weapon_type: WeaponTypeFilter::All,
            weapon_category: None,
            kind: None,
            scroll_y: 0.0,
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ShopUiState {
    pub active_tab: ShopTab,
    pub weapons: ShopTabState,
    pub helmets: ShopTabState,
    pub chestplates: ShopTabState,
    pub boots: ShopTabState,
    pub gloves: ShopTabState,
    pub accessories: ShopTabState,
    pub consumables: ShopTabState,
    pub artifacts: ShopTabState,
}

impl Default for ShopUiState {
    fn default() -> Self {
        Self {
            active_tab: ShopTab::Weapons,
            weapons: ShopTabState::default(),
            helmets: ShopTabState::default(),
            chestplates: ShopTabState::default(),
            boots: ShopTabState::default(),
            gloves: ShopTabState::default(),
            accessories: ShopTabState::default(),
            consumables: ShopTabState::default(),
            artifacts: ShopTabState::default(),
        }
    }
}

impl ShopUiState {
    pub fn state_for(&self, tab: ShopTab) -> &ShopTabState {
        match tab {
            ShopTab::Weapons => &self.weapons,
            ShopTab::Helmets => &self.helmets,
            ShopTab::Chestplates => &self.chestplates,
            ShopTab::Boots => &self.boots,
            ShopTab::Gloves => &self.gloves,
            ShopTab::Accessories => &self.accessories,
            ShopTab::Consumables => &self.consumables,
            ShopTab::Artifacts => &self.artifacts,
        }
    }

    pub fn state_for_mut(&mut self, tab: ShopTab) -> &mut ShopTabState {
        match tab {
            ShopTab::Weapons => &mut self.weapons,
            ShopTab::Helmets => &mut self.helmets,
            ShopTab::Chestplates => &mut self.chestplates,
            ShopTab::Boots => &mut self.boots,
            ShopTab::Gloves => &mut self.gloves,
            ShopTab::Accessories => &mut self.accessories,
            ShopTab::Consumables => &mut self.consumables,
            ShopTab::Artifacts => &mut self.artifacts,
        }
    }

    pub fn current_filters(&self) -> ShopFilters {
        let state = self.state_for(self.active_tab);
        ShopFilters {
            tab: self.active_tab,
            weapon_hand: state.weapon_hand,
            weapon_type: state.weapon_type,
            weapon_category: state.weapon_category,
            kind: state.kind,
        }
    }
}
