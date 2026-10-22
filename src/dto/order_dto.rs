use serde::Serialize;
use crate::dto::discount_type::DiscountType;
use crate::dto::order_type_dto::OrderType;

#[derive(Serialize)]
pub struct OrderOutput{
    pub order_type: OrderType,
    pub discount: DiscountType,
    pub total_value: f32,
}


#[derive(Serialize)]
pub struct OrderInput{
    pub order_type: OrderType,
    pub discount: DiscountType,
    pub weight: f32,
}