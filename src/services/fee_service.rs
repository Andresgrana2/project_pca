use crate::dto::fee_dto::FeeOutput;

pub async fn get_fee() -> FeeOutput {

    FeeOutput::new(5.00)
}
