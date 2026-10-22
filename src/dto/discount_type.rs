use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum DiscountType{
    NEW,
    FREQUENT,
    VIP
}
