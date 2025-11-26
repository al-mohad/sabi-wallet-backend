use anyhow::{anyhow, Result};
use phonenumber::{country, parse, PhoneNumber};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate)]
pub struct NigerianPhoneNumber(
    #[validate(custom(function = "validate_nigerian_phone_number"))]
    pub String,
);

impl NigerianPhoneNumber {
    /// Canonicalizes a given phone number to the E.164 format for Nigeria (+234...).
    /// Returns an error if the number is invalid or not Nigerian.
    pub fn new(raw_phone_number: &str) -> Result<Self> {
        let phone_number = parse(Some(country::NG), raw_phone_number)
            .map_err(|e| anyhow!("Failed to parse phone number: {}", e))?;

        if phone_number.country() != country::NG {
            return Err(anyhow!("Phone number is not a Nigerian number."));
        }

        Ok(Self(phone_number.format().display_e164().to_string()))
    }

    /// Returns the canonicalized E.164 string representation of the phone number.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Custom validation function for the `NigerianPhoneNumber` struct
fn validate_nigerian_phone_number(phone_number: &str) -> Result<(), validator::ValidationError> {
    match NigerianPhoneNumber::new(phone_number) {
        Ok(_) => Ok(()),
        Err(e) => {
            let mut err = validator::ValidationError::new("invalid_nigerian_phone_number");
            err.add_param("message", &e.to_string());
            Err(err)
        }
    }
}


impl std::fmt::Display for NigerianPhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implement `From` trait for convenience
impl From<NigerianPhoneNumber> for String {
    fn from(phone: NigerianPhoneNumber) -> Self {
        phone.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_nigerian_numbers() {
        assert_eq!(
            NigerianPhoneNumber::new("08012345678").unwrap().as_str(),
            "+2348012345678"
        );
        assert_eq!(
            NigerianPhoneNumber::new("+2348012345678")
                .unwrap()
                .as_str(),
            "+2348012345678"
        );
        assert_eq!(
            NigerianPhoneNumber::new("2348012345678").unwrap().as_str(),
            "+2348012345678"
        );
    }

    #[test]
    fn test_invalid_nigerian_numbers() {
        assert!(NigerianPhoneNumber::new("0701234567").is_err()); // Too short
        assert!(NigerianPhoneNumber::new("12345").is_err()); // Not a phone number
        assert!(NigerianPhoneNumber::new("+12025550100").is_err()); // Not Nigerian
    }
}
