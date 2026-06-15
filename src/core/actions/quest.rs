use crate::core::catalog::catalog::all_equipment;
use crate::core::player::Player;
use crate::core::ui::playing::reward_equipment;
use rand::{rng, seq::IndexedRandom, RngExt};

pub fn handle_quest(player: &mut Player) {
    let mut rng = rng();
    let gold_earned = rng.random_range(20..=40);
    if rng.random_bool(0.5) {
        let items: Vec<_> =
            all_equipment().iter().filter(|eq| eq.level() <= player.level as u32).collect();

        if let Some(item) = items.choose(&mut rng) {
            reward_equipment(player, item.name().to_string());
        }
    }
    player.gold += gold_earned;
}
