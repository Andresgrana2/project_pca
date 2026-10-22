use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum OrderType{
    STANDARD,
    EXPRESS
}
