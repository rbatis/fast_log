use arc_swap::ArcSwap;

///log filter
pub trait Filter: Send + Sync {
    /// if return true=do_log/false=not_log
    fn do_log(&self, record: &log::Record) -> bool;
}

/// an Module Filter
/// ```rust
/// fn main(){
///    use fast_log::Config;
///    use fast_log::filter::ModuleFilter;
///    let filter = ModuleFilter::with(vec![module_path!().to_string()]);
///    fast_log::init(Config::new().console().add_filter(filter)).unwrap();
/// }
/// ```
pub struct ModuleFilter {
    /// copy-on-write modules, support dynamic append after new
    pub modules: ArcSwap<Vec<String>>,
}

impl ModuleFilter {
    pub fn new() -> Self {
        Self { modules: ArcSwap::from_pointee(Vec::new()) }
    }

    pub fn with(modules: Vec<String>) -> Self {
        Self { modules: ArcSwap::from_pointee(modules) }
    }
}

impl Filter for ModuleFilter {
    fn do_log(&self, record: &log::Record) -> bool {
        let module = record.module_path().unwrap_or("");
        let modules = self.modules.load();
        if !modules.is_empty() {
            for x in modules.iter() {
                if module == x {
                    return false;
                }
            }
        }
        return true;
    }
}