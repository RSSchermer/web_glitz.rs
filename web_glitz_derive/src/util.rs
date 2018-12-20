use std::fmt::Display;


pub struct ErrorLog {
    errors: Vec<String>,
}

impl ErrorLog {
    pub fn new() -> Self {
        ErrorLog { errors: Vec::new() }
    }

    pub fn log_error<T>(&mut self, error: T)
    where
        T: Display,
    {
        self.errors.push(error.to_string());
    }

    pub fn compile(self) -> Result<(), String> {
        match self.errors.len() {
            0 => Ok(()),
            1 => Err(self.errors.into_iter().next().unwrap()),
            n => {
                let mut output = format!("{} errors:", n);

                for err in self.errors {
                    output.push_str("\n\t# ");
                    output.push_str(&err);
                }

                Err(output)
            }
        }
    }
}
