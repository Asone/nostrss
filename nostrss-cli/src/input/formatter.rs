pub struct InputFormatter {}

impl InputFormatter {
    pub fn input_to_vec(value: String) -> Vec<String> {
        value.split(',').map(|e| e.trim().to_string()).collect()
    }
}
