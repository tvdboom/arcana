use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pet {
    #[default]
    Bear,
    Eagle,
    Snake,
    Wolf,
}
