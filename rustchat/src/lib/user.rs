#[derive(Debug)]
pub struct User {
    id: u64,
    name: String,
    password: Vec<u8>
}

impl User {
    fn new(id: u64, name: String, password: Vec<u8>) -> Self {
        User {
            id,
            name,
            password
        }
    }
}