use std::io::{Bytes, Read};

use sha3::{Digest, Sha3_256};

use crate::protocol::{Framing, PacketError};

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
        let name = String::from(name.trim());
        let password = String::from(password.trim());
        hasher.update(password.as_bytes());
        let password = hasher.finalize().into_iter().collect();
        User {
            name,
            password
        }
    }
    pub fn get_user_id(&self) {
        unimplemented!();
    }
    pub fn get_username(&self) -> &str{
        &self.name
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

    fn decode<T: Read>(data: &mut Bytes<T>) -> Result<User, PacketError> {
        //user id-> 8byte, name.len >= 2(include null byte), password == 32
        let name = data.take_while(|x| !x.contains(&0))
            .collect();
        let name = match name {
            Ok(result) => String::from_utf8(result).unwrap(),
            Err(_) => return Err(PacketError::ParseError)
        };

        let password = 
            data.collect::<Result<Vec<u8>,_>>();
        let password = match password {
            Ok(v) => v,
            Err(_) => return Err(PacketError::ParseError)
        };

        Ok( User {
            name,
            password
        })
    }
}

#[test]
fn test_encode_decode() {
    let user = User::new("fuck", "password");
    let User { name, password} = User::decode(&mut user.encode_data().bytes()).unwrap();
    assert_eq!(name ,user.name);
    assert_eq!(password ,user.password);
}