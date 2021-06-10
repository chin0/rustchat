use core::panic;
use std::fmt;

use crate::{data::Message, user::User};

#[derive(Debug,Clone)]
pub struct WrongPacketError;

//에러 세분화 해야함.(CannatParse, WrongCommand, ...)
impl fmt::Display for WrongPacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid packet!")
    }
}
enum Command {
    Register(User),  //회원가입
    Login(User),
    JoinChatRoom, //채팅방에 접속
    CreateChatRoom,
    Message(Message), //메시지 전송
}

pub type Result<T> = std::result::Result<T, WrongPacketError>;
pub trait Framing {
    //type T to byte stream data.
    fn encode_data(&self) -> Vec<u8>;
    fn decode(data: &[u8]) -> Result<Self> where Self: Sized;
}

impl Framing for Command {
    fn encode_data(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        match self {
            &Command::Register(ref user) => {
                encoded.push(0x30);
                encoded.extend(user.encode_data().iter());
            },
            &Command::Login(ref user) => {
                encoded.push(0x31);
                encoded.extend(user.encode_data().iter());
            }
            &Command::Message(ref msg) => {
                encoded.push(0x35);
                encoded.extend(msg.encode_data().iter());
            }
            _ => {

            }
        }
        encoded
    }

    fn decode(data: &[u8]) -> Result<Self> {
        let command_code = data[0];
        match command_code {
            0x30 => {
                let decoded = User::decode(&data[1..]).unwrap();
                Ok(Self::Register(decoded))
            },
            0x31 => {
                //얘네도 다 Err로 바꿔줘야함.
                let decoded = User::decode(&data[1..]).unwrap();
                Ok(Self::Login(decoded))
            },
            0x35 => {
                let decoded = Message::decode(&data[1..]).unwrap();
                Ok(Self::Message(decoded))
            }
            _ => {
                Err(WrongPacketError)
            }
        }
    }
}