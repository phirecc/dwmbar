use crate::module::{Module, ModuleResult};
pub struct Combined {
    m1: Box<dyn Module>,
    m2: Box<dyn Module>,
}
impl Combined {
    pub fn new(m1: Box<dyn Module>, m2: Box<dyn Module>) -> Self {
        Combined {
            m1,
            m2,
        }
    }
}
impl Module for Combined {
    fn eval(&self) -> ModuleResult {
        let res1 = self.m1.eval()?;
        let res2 = self.m2.eval()?;
        if res1.is_none() || res2.is_none() {
            Ok(None)
        }
        else {
            Ok(Some(format!("{} {}", res1.unwrap(), res2.unwrap())))
        }
    }
}
