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
    pub contains: Vec<String>
}

impl Filter for ModuleFilter {
    fn filter(&self, module: &str) -> bool {
        for x in &self.contains {
            if module.contains(x) {
                //not filter
                return false;
            }
        }
        //filter
        return true;
    }
}