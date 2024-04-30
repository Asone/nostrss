use std::str::FromStr;

use bech32::Hrp;
use url::Url;

pub struct InputValidators {}

impl InputValidators {
    pub fn required_input_validator(value: String) -> bool {
        if value.is_empty() {
            return false;
        }

        true
    }

    pub fn url_validator(value: String) -> bool {
        Url::parse(&value).is_ok()
    }

    pub fn cron_pattern_validator(value: String) -> bool {
        cron::Schedule::from_str(&value).is_ok()
    }

    pub fn key_validator(value: String) -> bool {
        let decoded = bech32::decode(value.trim());

        match decoded {
            Ok(result) => {
                if result.0 != Hrp::parse("nsec").unwrap() {
                    return false;
                }
            }
            Err(_) => {
                let key_bytes = value.trim().as_bytes();

                // Validate key length
                if key_bytes.len() != 64 {
                    return false;
                }

                // Validate key contains only hexadecimal characters
                for &byte in key_bytes.iter() {
                    if !byte.is_ascii_hexdigit() {
                        return false;
                    }
                }
            }
        };

        true
    }

    pub fn default_guard_validator(value: String) -> bool {
        if value == "default".to_string() {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::input::input::InputValidators;

    #[test]
    fn required_input_validator_test() {
        let value = "abc".to_string();
        let result = InputValidators::required_input_validator(value);

        assert_eq!(result, true);

        let value = "".to_string();
        let result = InputValidators::required_input_validator(value);

        assert_eq!(result, false);
    }

    #[test]
    fn url_validator_test() {
        let value = "https://www.domain.org".to_string();
        let result = InputValidators::url_validator(value);

        assert_eq!(result, true);

        let value = "invalid_url".to_string();
        let result = InputValidators::url_validator(value);

        assert_eq!(result, false);
    }

    #[test]
    fn cron_pattern_validator_test() {
        let value = "1/10 * * * * *".to_string();
        let result = InputValidators::cron_pattern_validator(value);

        assert_eq!(result, true);

        let value = "1/10 * * *".to_string();
        let result = InputValidators::cron_pattern_validator(value);

        assert_eq!(result, false);

        let value = "1/10 * * * * * * * *".to_string();
        let result = InputValidators::cron_pattern_validator(value);

        assert_eq!(result, false);
    }

    #[test]
    fn key_validator_test() {
        let value = "6789abcdef0123456789abcdef0123456789abcdef0123456789abcdef012345".to_string();

        let result = InputValidators::key_validator(value);

        assert_eq!(result, true);

        let value = "6789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();

        let result = InputValidators::key_validator(value);

        assert_eq!(result, false);

        let value = "6789abcdef0123456789abcdef0123456789abcdef0123456789abkdef012345".to_string();

        let result = InputValidators::key_validator(value);

        assert_eq!(result, false);

        let value = "nsec14uuscmj9ac0f3lqfq33cuq6mu8q7sscvpyyhsjn5r8q9w5pdafgq0qrj8a".to_string();

        let result = InputValidators::key_validator(value);

        assert_eq!(result, true);

        let value = "nsec14uuscmj9ac0f3lqfq33cuq6mu8q7sscvpyyhsjn5r8q9w5pdafgq0qrj8d".to_string();

        let result = InputValidators::key_validator(value);

        assert_eq!(result, false);
    }
}
