use std::convert::TryInto;

use sha3::{Digest, Sha3_256};

use crate::protocol::{Framing, Result};

#[derive(Debug)]
pub struct User {
    name: String,
    password: Vec<u8>
}


impl User {
    //password is hashed value.
    //name에 대한 검증 필요.
    pub fn new(name: &str, password: &str) -> Self {
        let mut hasher = Sha3_256::new();
        let name = String::from(name);
        let password = String::from(password);
        hasher.update(password.as_bytes());
        let password = hasher.finalize().into_iter().collect();
        User {
            name,
            password
        }
    }
    fn get_user_id(&self) {
        unimplemented!();
    }
}

impl Framing for User {

    fn encode_data(&self) -> Vec<u8> { 
        let mut encoded = Vec::new();
        //name not include special chactor
        let name = self.name.as_bytes();
        encoded.extend_from_slice(name);
        encoded.push(0); //include null byte.
        encoded.extend_from_slice(&self.password);
        encoded
    }

    fn decode(data: &[u8]) -> Result<User> {
        //user id-> 8byte, name.len >= 2(include null byte), password == 32
        assert!(data.len() >= 2 + 32); 
        let first_null_position = data.iter().position(|x| *x == 0).expect("wrong username");
        println!("{}", first_null_position);
        let name = String::from_utf8(data[0..first_null_position].to_vec()).unwrap();
        let password = data[first_null_position+1..first_null_position+33].to_vec();
        Ok(User {
            name,
            password
        })
    }
}

#[test]
fn test_encode_decode() {
    let user = User::new("fuck", "password");
    let User { name, password} = User::decode(&user.encode_data()).unwrap();
    assert_eq!(name ,user.name);
    assert_eq!(password ,user.password);
}