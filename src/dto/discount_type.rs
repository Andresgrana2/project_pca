use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DiscountType{
    NEW,
    FREQUENT,
    VIP
}
