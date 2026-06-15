use crate::core::catalog::catalog::all_equipment;
use crate::core::player::Player;
use crate::core::ui::playing::reward_equipment;
use rand::{rng, seq::IndexedRandom};

pub fn handle_craft(player: &mut Player) {
    let mut rng = rng();
    let items: Vec<_> =
        all_equipment().iter().filter(|eq| eq.level() == player.level as u32).collect();

    if let Some(item) = items.choose(&mut rng) {
        reward_equipment(player, item.name().to_string());
    }
}
