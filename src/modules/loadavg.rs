use crate::module::{Module, ModuleResult};
use std::fs;
pub struct LoadAvg {
}
impl Module for LoadAvg {
    fn eval(&self) -> ModuleResult {
        let s = fs::read_to_string("/proc/loadavg")?;
        Ok(Some(
            s.split(' ')
                .next()
                .ok_or("loadavg missing first part")?
                .to_string(),
        ))
    }
}
