// user model
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize, Serialize)]
pub struct AddUserRequest {
    #[validate(length(min = 1, message = "Fullname required"))]
    pub fullname: String,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct EditUserURL {
    pub uuid: String,
}

#[derive(Validate, Deserialize, Serialize, Debug)]
pub struct Users {
    pub uuid: String,
    pub fullname: String,
}

impl Users {
    pub fn new(uuid: String, fullname: String) -> Users {
        Users { uuid, fullname }
    }
}
