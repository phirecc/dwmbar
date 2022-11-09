use crate::module::{Module, ModuleResult};
use std::path::Path;
pub struct Vpn {
}
impl Module for Vpn {
    fn eval(&self) -> ModuleResult {
        let vpns = ["wg0", "wgnord", "wg-mullvad"];
        for vpn in vpns {
            if Path::new(&("/sys/class/net/".to_owned() + vpn)).exists() {
                return Ok(Some("VPN".to_string()));
            }
        }
        Ok(None)
    }
}
