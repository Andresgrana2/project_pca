use serde::Serialize;

#[derive(Serialize)]
pub enum OrderType{
    STANDARD,
    EXPRESS
}