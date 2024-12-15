use crate::config::{TABLE_PATH, TABLE_USERS_NAME};
use crate::errors::my_errors::MyError;
use crate::utils::get_reader;
use std::io::BufRead;

/// Enum que representa los rolos que puede tener un usuario.
#[derive(Debug, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

/// User struct that represents the user of the application.
/// Used for Authentication and Authoriazation.
///
/// User: Can only do SELECT queries
/// Admin: Can do SELECT, INSERT, UPDATE, DELETE queries
#[derive(Debug)]
pub struct User {
    pub username: String,
    pub password: String,
    pub role: Role,
}

impl User {
    pub fn new(username: String, password: String) -> Result<Self, MyError> {
        let role = match User::get_role(username.as_str(), password.as_str()) {
            Ok(role) => role,
            Err(e) => return Err(e),
        };

        Ok(User {
            username: username.to_string(),
            password: password.to_string(),
            role,
        })
    }

    fn get_role(username: &str, password: &str) -> Result<Role, MyError> {
        let rdr = get_reader(TABLE_PATH, TABLE_USERS_NAME)?;

        let mut lines = rdr.lines();
        lines.next(); // Skip header

        for line in lines {
            let line = line.map_err(|e| MyError::FileError(e.to_string()))?;
            let mut values: Vec<&str> = line.split(",").collect();

            if values.len() < 3 {
                return Err(MyError::UserError("Invalid user file".to_string()));
            }

            let role: &str = values.pop().unwrap_or("user");
            let pass: &str = values.pop().unwrap_or("pass");
            let user: &str = values.pop().unwrap_or("user");

            if user == username && pass == password {
                return match role {
                    "admin" => Ok(Role::Admin),
                    "user" => Ok(Role::User),
                    _ => Err(MyError::UserError("Invalid role".to_string())),
                };
            }
        }
        Err(MyError::UserError("User not found".to_string()))
    }
}
