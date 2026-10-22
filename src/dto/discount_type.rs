use serde::Serialize;

#[derive(Serialize)]
pub enum DiscountType{
    NEW,
    FREQUENT,
    VIP
}