use crate::dto::fee_dto::FeeOutput;



#[derive(Clone)]
pub struct FeeService {}


impl FeeService {
    pub fn new() -> Self {
        Self {}
    }


    pub async fn get_fee(&self) -> FeeOutput {

        FeeOutput::new(5.00)
    }

}

