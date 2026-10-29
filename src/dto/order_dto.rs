use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::dto::discount_type::DiscountType;
use crate::dto::order_type_dto::OrderType;

#[derive(Serialize)]
pub struct OrderOutput{
    pub order_type: OrderType,
    pub discount: DiscountType,
    pub total_value: f32,
}


#[derive(Deserialize, Serialize, Validate)]
pub struct OrderInput{
    pub order_type: OrderType,
    pub discount: DiscountType,
    #[validate(range(
        exclusive_min = 0.0,
        message = "El peso debe ser mayor que cero"
    ))]
    pub weight: f32,
}
