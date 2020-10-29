///log filter
pub trait Filter:Send+Sync {
    //return is filter
    fn filter(&self, module: &str) -> bool;
}

pub struct NoFilter {

}
impl Filter for NoFilter{
    fn filter(&self, module: &str) -> bool {
        return false;
    }
}

pub struct ModuleFilter {
    pub modules: Vec<String>
}

impl Filter for ModuleFilter {
    fn filter(&self, module: &str) -> bool {
        for x in &self.modules {
            if module == x {
                return false;
            }
        }
        return true;
    }
}