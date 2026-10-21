use serde::Serialize;

#[derive(Serialize)]
pub struct FeeOutput{
    pub base:f32
}


impl FeeOutput {

    pub fn new(base:f32)-> Self{
        Self{base}
    }
}