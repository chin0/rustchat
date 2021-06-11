use std::convert::TryInto;
use std::io::Bytes;
use std::io::Read;

use chrono::prelude::*;

use crate::protocol::Framing;
use crate::protocol::PacketError;

pub struct Message {
    username: String,
    time: i64,
    data: String,
}


//TODO: Unwrap 부분 전부 Err 리턴으로 바꾸기.
impl Message {
    pub fn new(username: &str, message: &str) -> Self {
        let username = String::from(username.trim());
        let data = String::from(message);
        let time = Utc::now().timestamp();
        Message {
            username,
            time,
            data
        }
    }
    pub fn to_str(&self) -> String{
        let naive = NaiveDateTime::from_timestamp(self.time, 0);
        let datetime:DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
        format!("[{}][{}]{}", newdate, self.username, self.data)
    }

    pub fn get_sender(&self) -> &str {
        &self.username
    }
}


// 이건 length encoding이 필수다.
// 첫 8바이트를 data length로 하고, username은 null terminate로 간주해서 파싱하자.
impl Framing for Message {
    fn encode_data(&self) -> Vec<u8> {
        let datalength: u64 = self.data.len() as u64;
        let mut encoded = Vec::new();

        encoded.extend_from_slice(datalength.as_ne_bytes());
        encoded.extend_from_slice(self.time.as_ne_bytes());
        encoded.extend_from_slice(self.username.as_bytes().try_into().unwrap());
        encoded.push(0);
        encoded.extend_from_slice(self.data.as_bytes().try_into().unwrap());
        encoded
    }

    fn decode<T: Read>(data: &mut Bytes<T>) -> Result<Self, PacketError> {
        let datalength = u64::from_ne_bytes(
            data.take(8)
            .collect::<Result<Vec<u8>, _>>()
            .unwrap()
            .try_into()
            .unwrap());
        let time = i64::from_ne_bytes(
            data.take(8)
            .collect::<Result<Vec<u8>, _>>()
            .unwrap()
            .try_into()
            .unwrap());
        let username = data.take_while(|x| !x.contains(&0))
            .collect();
        let username = match username {
            Ok(result) => String::from_utf8(result).unwrap(),
            Err(_) => return Err(PacketError::ParseError)
        };
        let data = data.take(datalength as usize)
            .collect();
        let data = match data {
            Ok(result) => String::from_utf8(result).unwrap(),
            Err(_) => return Err(PacketError::ParseError)
        };
        Ok(Message {
            username,
            time,
            data
        })
    }
}

#[test]
fn test_encode_decode() {
    let msg = Message::new("fuck", "fuckyou");
    let Message { username, time, data} = Message::decode(&mut msg.encode_data().bytes()).unwrap();
    assert_eq!(username ,msg.username);
    assert_eq!(time ,msg.time);
    assert_eq!(data ,msg.data);
}