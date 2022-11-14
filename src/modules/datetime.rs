use crate::module::{Module, ModuleResult};
#[derive(Debug)]
pub struct DateTime {}
impl Module for DateTime {
    fn eval(&self) -> ModuleResult {
        let dt = chrono::Local::now();
        Ok(Some(dt.format("%d/%m/%y %a %H:%M").to_string()))
    }
}
