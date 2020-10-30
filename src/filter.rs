///log filter
pub trait Filter: Send + Sync {
    //return is filter
    fn filter(&self, record: &log::Record) -> bool;
}

pub struct NoFilter {}

impl Filter for NoFilter {
    fn filter(&self, module: &log::Record) -> bool {
        return false;
    }
}

pub struct ModuleFilter {
    //include
    pub contains: Option<Vec<String>>,
    //exclude
    pub exclude_contains: Option<Vec<String>>,
}

impl Filter for ModuleFilter {
    fn filter(&self, record: &log::Record) -> bool {
        let module = record.module_path().unwrap_or("");
        if self.contains.is_some() {
            for x in self.contains.as_ref().unwrap() {
                if module.contains(x) {
                    //not filter
                    return false;
                }
            }
            //filter
            return true;
        }
        if self.exclude_contains.is_some() {
            for x in self.exclude_contains.as_ref().unwrap() {
                if module.contains(x) {
                    //filter
                    return true;
                }
            }
            //not filter
            return false;
        }
        //not filter
        return false;
    }
}