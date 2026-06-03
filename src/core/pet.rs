use crate::core::player::Pet;

impl Pet {
    pub fn max_health(&self) -> i32 {
        (self.vitality * 12) as i32 + 40
    }

    pub fn damage(&self) -> i32 {
        (self.strength * 2) as i32 + 5
    }

    pub fn evasion_rate(&self) -> f32 {
        let mut rate = self.dexterity as f32 * 0.005;
        if rate > 0.40 { rate = 0.40; } // Cap pet evasion at 40%
        rate
    }

    pub fn accuracy_rate(&self) -> f32 {
        1.0 + (self.dexterity as f32 * 0.005)
    }

    pub fn critical_rate(&self) -> f32 {
        self.dexterity as f32 * 0.003
    }

    pub fn train_strength(&mut self, cost: u32, gold: &mut u32) -> Result<(), String> {
        if *gold < cost {
            return Err("Insufficient Gold to train pet.".to_string());
        }
        *gold -= cost;
        self.strength += 1;
        Ok(())
    }

    pub fn train_dexterity(&mut self, cost: u32, gold: &mut u32) -> Result<(), String> {
        if *gold < cost {
            return Err("Insufficient Gold to train pet.".to_string());
        }
        *gold -= cost;
        self.dexterity += 1;
        Ok(())
    }

    pub fn train_vitality(&mut self, cost: u32, gold: &mut u32) -> Result<(), String> {
        if *gold < cost {
            return Err("Insufficient Gold to train pet.".to_string());
        }
        *gold -= cost;
        self.vitality += 1;
        let old_max = self.max_health();
        self.max_health = self.max_health();
        self.current_health += self.max_health() - old_max;
        Ok(())
    }

    pub fn level_up(&mut self) {
        self.level += 1;
        // Auto-increment stats slightly on level up
        self.strength += 1;
        self.dexterity += 1;
        self.vitality += 1;
        self.max_health = self.max_health();
        self.current_health = self.max_health();
    }
}
