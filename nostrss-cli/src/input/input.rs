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
