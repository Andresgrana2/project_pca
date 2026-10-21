use crate::dto::discount_type::DiscountType;
use crate::dto::order_type_dto::OrderType;

/*pub fn create_order() -> Result<T, E> {



}*/

fn order_type(value: f32, type_ord : OrderType) -> f32{
    match type_ord {
        OrderType::STANDARD => value,
        OrderType::EXPRESS => value * 1.25
    }
}

fn discount(value: f32, disc_type : DiscountType) -> f32{

    match disc_type {
        DiscountType::NEW => value,
        DiscountType::FREQUENT => value * 0.90,
        DiscountType::VIP => value * 0.80
    }
}