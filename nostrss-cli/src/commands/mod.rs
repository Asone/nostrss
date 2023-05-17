use std::io::{self, stdin, Write};

use tabled::{Table, Tabled};
use tonic::async_trait;

/// Common trait for sub-handlers.
#[async_trait]
pub trait CommandsHandler {
    // A default helper to get input data from user.
    fn get_input(&self, label: &str, validator: Option<fn(String) -> bool>) -> String {
        let mut data = String::new();
        print!("{}", label);
        _ = io::stdout().flush();
        _ = stdin().read_line(&mut data);

        match validator {
            Some(validator) => match validator(data.clone()) {
                true => data,
                false => {
                    println!("Invalid value provided.");
                    self.get_input(label, Some(validator))
                }
            },
            None => data,
        }
    }

    fn print(&self, data: Vec<impl Tabled>) {
        let table = Table::new(data).to_string();
        println!("{}", table);
    }
}

pub mod feed;
pub mod profile;
pub mod relay;
