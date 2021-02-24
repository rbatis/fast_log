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
    //include contains
    pub include: Option<Vec<String>>,
    //exclude contains
    pub exclude: Option<Vec<String>>,
}

impl ModuleFilter {
    pub fn new_include(arg: Vec<String>) -> Self {
        Self {
            include: Some(arg),
            exclude: None,
        }
    }
    pub fn new_exclude(arg: Vec<String>) -> Self {
        Self {
            include: None,
            exclude: Some(arg),
        }
    }
    pub fn new(include: Option<Vec<String>>, exclude: Option<Vec<String>>) -> Self {
        Self { include, exclude }
    }
}

impl Filter for ModuleFilter {
    fn filter(&self, record: &log::Record) -> bool {
        let module = record.module_path().unwrap_or("");
        if self.include.is_some() {
            for x in self.include.as_ref().unwrap() {
                if module.contains(x) {
                    //not filter
                    return false;
                }
            }
            //filter
            return true;
        }
        if self.exclude.is_some() {
            for x in self.exclude.as_ref().unwrap() {
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
