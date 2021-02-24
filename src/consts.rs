pub enum LogSize {
    KB(usize),
    MB(usize),
    GB(usize),
}

impl LogSize {
    pub fn get_len(&self) -> usize {
        match self {
            Self::KB(kb) => {
                return kb * 1024;
            }
            Self::MB(mb) => {
                return mb * 1024 * 1024;
            }
            Self::GB(gb) => {
                return gb * 1024 * 1024 * 1024;
            }
        }
    }
}
