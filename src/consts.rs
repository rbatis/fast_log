pub enum LogSize {
    KB(usize),
    MB(usize),
    GB(usize),
}

impl LogSize {
    pub fn get_len(&self) -> usize {
        match self {
            Self::KB(kb) => {
                 kb * 1024
            }
            Self::MB(mb) => {
                 mb * 1024 * 1024
            }
            Self::GB(gb) => {
                 gb * 1024 * 1024 * 1024
            }
        }
    }
}
