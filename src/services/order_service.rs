use crate::dto::discount_type::DiscountType;
use crate::dto::order_dto::{OrderInput, OrderOutput};
use crate::dto::order_type_dto::OrderType;
use crate::error::AppError;
use crate::services::fee_service::FeeService;
use crate::services::packing_service::calculate;

#[derive(Clone)]
pub struct OrderService{}


impl OrderService{
    pub fn new() -> Self{
        Self {}
    }

    pub async fn create_order(&self, order_input: OrderInput,
                              fee_service: &FeeService) -> Result<OrderOutput, AppError> {

        let order_type = order_input.order_type;

        let discount_type = order_input.discount;

        let mut total_value = fee_service.get_fee().await.base;


        total_value += calculate(order_input.weight)?;

        total_value = self.order_type(total_value, &order_type);

        total_value = self.discount(total_value, &discount_type);


        Ok(
            OrderOutput{
                order_type,
                discount: discount_type,
                total_value,
            }
        )
    }

    fn order_type(&self, value: f32, type_ord: &OrderType) -> f32{
        match type_ord {
            OrderType::STANDARD => value,
            OrderType::EXPRESS => value * 1.25
        }
    }

    fn discount(&self, value: f32, disc_type: &DiscountType) -> f32{

        match disc_type {
            DiscountType::NEW => value,
            DiscountType::FREQUENT => value * 0.90,
            DiscountType::VIP => value * 0.80
        }
    }

}


