use crate::error::LogError;

pub enum LogSize {
    B(usize),
    KB(usize),
    MB(usize),
    GB(usize),
    TB(usize),
    EB(usize),
}

impl LogSize {
    pub fn len(&self) -> usize {
        match self {
            Self::B(b) => *b,
            Self::KB(kb) => kb * 1024,
            Self::MB(mb) => mb * 1024 * 1024,
            Self::GB(gb) => gb * 1024 * 1024 * 1024,
            Self::TB(tb) => tb * 1024 * 1024 * 1024 * 1024,
            Self::EB(eb) => eb * 1024 * 1024 * 1024 * 1024 * 1024,
        }
    }

    pub fn get_len(&self) -> usize {
        self.len()
    }

    pub fn parse(value: &str) -> Result<Self, LogError> {
        if value.ends_with("EB") {
            Ok(Self::EB(
                value.trim_end_matches("EB").parse().unwrap_or_default(),
            ))
        } else if value.ends_with("TB") {
            Ok(Self::TB(
                value.trim_end_matches("TB").parse().unwrap_or_default(),
            ))
        } else if value.ends_with("GB") {
            Ok(Self::GB(
                value.trim_end_matches("GB").parse().unwrap_or_default(),
            ))
        } else if value.ends_with("MB") {
            Ok(Self::MB(
                value.trim_end_matches("MB").parse().unwrap_or_default(),
            ))
        } else if value.ends_with("KB") {
            Ok(Self::KB(
                value.trim_end_matches("KB").parse().unwrap_or_default(),
            ))
        } else if value.ends_with("B") {
            Ok(Self::B(
                value.trim_end_matches("B").parse().unwrap_or_default(),
            ))
        } else {
            Err(LogError::from("unknow LogSize"))
        }
    }
}
