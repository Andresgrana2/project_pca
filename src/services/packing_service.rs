use crate::dto::packing_dto::{PackingError};

pub fn calculate(value: f32) -> Result<f32, PackingError> {

    if !value.is_finite() || value < 0.01 {
        Err(PackingError::InvalidWeight)
    } else if value <= 5.00 {
        Ok(value * 2.0)
    } else if value <= 20.0 {
        Ok(value * 4.0)
    } else {
        Err(PackingError::InvalidPacking)
    }
}