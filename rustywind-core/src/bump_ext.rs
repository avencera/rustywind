use bumpalo::collections::String;
use bumpalo::collections::Vec;

pub trait JoinIn {
    fn join_in<'a>(self, separator: &'static str, bump: &'a bumpalo::Bump) -> String<'a>;
}

impl JoinIn for Vec<'_, &str> {
    fn join_in<'a>(self, separator: &'static str, bump: &'a bumpalo::Bump) -> String<'a> {
        let mut string = String::new_in(bump);

        match self.len() {
            0 => (),
            1 => {
                string.push_str(self[0]);
            }
            _ => {
                for class in self[0..self.len() - 1].iter() {
                    string.push_str(class);
                    string.push_str(separator);
                }
            }
        }

        string
    }
}

impl JoinIn for Vec<'_, String<'_>> {
    fn join_in<'a>(self, separator: &'static str, bump: &'a bumpalo::Bump) -> String<'a> {
        let mut string = String::new_in(bump);

        match self.len() {
            0 => (),
            1 => {
                string.push_str(&self[0]);
            }
            _ => {
                for class in self[0..self.len() - 1].iter() {
                    string.push_str(class);
                    string.push_str(separator);
                }
            }
        }

        string
    }
}
