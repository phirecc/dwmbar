use crate::module::{Module, ModuleResult};
use std::fs;
#[derive(Debug)]
pub struct Battery {}
impl Module for Battery {
    fn eval(&self) -> ModuleResult {
        let base = "/sys/class/power_supply/BAT0/";
        let charge_now = fs::read_to_string(base.to_owned() + "charge_now")?;
        let charge_full = fs::read_to_string(base.to_owned() + "charge_full")?;
        let charge_now: f64 = charge_now.trim().parse()?;
        let charge_full: f64 = charge_full.trim().parse()?;
        let mut status = "";
        if fs::read_to_string(base.to_owned() + "status")?.trim() == "Charging" {
            status = "CHR ";
        }
        Ok(Some(format!(
            "{}{}%",
            status,
            ((charge_now as f64 / charge_full as f64) * 100.0) as usize
        )))
    }
}
