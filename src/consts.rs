pub enum LogSize {
    KB(usize),
    MB(usize),
    GB(usize),
    TB(usize),
    EB(usize),
}

impl LogSize {
    pub fn get_len(&self) -> usize {
        match self {
            Self::KB(kb) => kb * 1024,
            Self::MB(mb) => mb * 1024 * 1024,
            Self::GB(gb) => gb * 1024 * 1024 * 1024,
            Self::TB(tb) => tb * 1024 * 1024 * 1024 * 1024,
            Self::EB(eb) => eb * 1024 * 1024 * 1024 * 1024 * 1024,
        }
    }
}
