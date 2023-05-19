pub struct InputFormatter {}

impl InputFormatter {
    pub fn input_to_vec(value: String) -> Vec<String> {
        value.split(',').map(|e| e.trim().to_string()).collect()
    }

    pub fn string_nullifier(value: String) -> Option<String> {
        match value.len() > 0 {
            true => Some(value.trim().to_string()),
            false => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InputFormatter;

    #[test]
    fn input_to_vec_test() {
        let value = "a,b,c".to_string();

        let result = InputFormatter::input_to_vec(value);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "a".to_string());
    }
}
