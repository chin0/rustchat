use std::{fmt, io::{Bytes, Read}};

use crate::{data::Message, user::User};

#[derive(Debug,Clone)]
pub enum PacketError{
    DisconnectError,
    ParseError,
}

//에러 세분화 해야함.(CannatParse, WrongCommand, ...)
impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Self::DisconnectError => {
                write!(f, "socket is disconnected.")
            },
            &Self::ParseError => {
                write!(f, "wrong packet data.")
            }
        }
    }
}
pub enum Command {
    Register(User),  //회원가입
    Login(User),
    JoinChatRoom, //채팅방에 접속
    CreateChatRoom,
    Message(Message), //메시지 전송
}

pub enum Response {

}

pub type Result<T> = std::result::Result<T, PacketError>;
pub trait Framing {
    //type T to byte stream data.
    fn encode_data(&self) -> Vec<u8>;
    fn decode<T: Read>(data: &mut Bytes<T>) -> Result<Self> where Self: Sized;
}

impl Framing for Response {
    fn encode_data(&self) -> Vec<u8> {
        todo!()
    }

    fn decode<T: Read>(data: &mut Bytes<T>) -> Result<Self> where Self: Sized {
        todo!()
    }
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

    fn decode<T: Read>(data: &mut Bytes<T>) -> Result<Self> {
        let command_code = data.next();
        if let None = command_code {
            return Err(PacketError::DisconnectError);
        }
        match command_code {
            Some(Ok(0x30)) => {
                let decoded = User::decode(data).unwrap();
                Ok(Self::Register(decoded))
            },
            Some(Ok(0x31)) => {
                //얘네도 다 Err로 바꿔줘야함.
                let decoded = User::decode(data).unwrap();
                Ok(Self::Login(decoded))
            },
            Some(Ok(0x35)) => {
                let decoded = Message::decode(data).unwrap();
                Ok(Self::Message(decoded))
            }
            _ => {
                Err(PacketError::ParseError)
            }
        }
    }
}