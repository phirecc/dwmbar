use std::error::Error;
pub type ModuleResult = Result<Option<String>, Box<dyn Error>>;

pub trait Module: Sync + Send {
    fn eval(&self) -> ModuleResult;
}
