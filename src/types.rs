pub type Id = i32;

pub struct EAN13Barcode {
    pub value: String,
}

pub enum EAN13Error {
    InvalidLength,
    InvalidCharacter,
}

impl TryFrom<String> for EAN13Barcode {
    type Error = EAN13Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 13 {
            return Err(EAN13Error::InvalidLength);
        }
        if !value.chars().all(|c| c.is_digit(10)) {
            return Err(EAN13Error::InvalidCharacter);
        }
        Ok(Self { value })
    }
}
