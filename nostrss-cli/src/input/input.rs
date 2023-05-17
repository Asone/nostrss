use std::str::FromStr;

use url::Url;

pub struct InputValidators {}

impl InputValidators {
    pub fn required_input_validator(value: String) -> bool {
        if value.len() == 0 {
            return false;
        }

        return true;
    }

    pub fn url_validator(value: String) -> bool {
        let r = Url::parse(&value);

        match r {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn cron_pattern_validator(value: String) -> bool {
        let r = cron::Schedule::from_str(&value);

        match r {
            Ok(_) => true,
            Err(_) => false,
        }
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
}
