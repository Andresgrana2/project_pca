use crate::dto::packing_dto::{PackingError, PackingInput};

pub fn calculate(input: PackingInput) -> Result<f32, PackingError> {

    if 0.01 >= input.weight && input.weight < 5.00 {
        Ok(2.0)
    } else if 5.00 >= input.weight && input.weight < 20.0 {
        Ok(4.0)
    } else {
        Err(PackingError::InvalidPacking)
    }
}
