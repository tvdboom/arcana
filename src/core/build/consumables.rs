use crate::core::build::effects::Effect;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Consumable {
    /// Name of the consumable (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the armor corresponds to
    pub image: String,

    /// Level or upgrade tier of the wearable
    pub level: u32,

    /// Gold value for buying and selling at merchants
    pub price: u32,

    /// Optional passive effect triggered when struck by an enemy
    pub effects: Vec<Effect>,
}
