use crate::core::player::Player;
use rand::{rng, RngExt};

pub fn handle_hunt(player: &mut Player) {
    let mut rng = rng();
    let gold_earned = rng.random_range(10..=20);
    player.gold += gold_earned;
}
