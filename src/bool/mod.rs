use crate::Error;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bool {
    False,
    True,
}

impl Bool {
    pub fn parse(s: &str) -> Result<Self, Error> {
        match s {
            "false" => Ok(Self::False),
            "true" => Ok(Self::True),
            _ => Err(Error::InvalidLiteral)
        }
    }

    pub fn value(self) -> bool {
        match self {
            Self::False => false,
            Self::True => true,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::False => "false",
            Self::True => "true",
        }
    }
}


#[cfg(test)]
mod tests;
