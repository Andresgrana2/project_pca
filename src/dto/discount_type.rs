use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscountType{
    NEW,
    FREQUENT,
    VIP
}
