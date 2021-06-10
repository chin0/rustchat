use std::convert::TryInto;

use chrono::prelude::*;
use crate::protocol::Result;

use crate::protocol::Framing;
pub struct Message {
    username: String,
    time: i64,
    data: String,
}


//TODO: Unwrap 부분 전부 Err 리턴으로 바꾸기.
impl Message {
    pub fn new(username: &str, message: &str) -> Self {
        let username = String::from(username);
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
        let newdate = datetime.format("%Y-%m-%d %H:%M;%S");
        format!("[{}][{}]{}", newdate, self.username, self.data)
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

    fn decode(data: &[u8]) -> Result<Self> {
        let datalength = u64::from_ne_bytes(data[0..8].try_into().unwrap());
        let time = i64::from_ne_bytes(data[8..16].try_into().unwrap());
        let first_null_position = 16 + data.iter().skip(16)
            .position(|x| *x == 0)
            .expect("wrong username");
        let username = String::from_utf8(data[16..first_null_position].to_vec()).unwrap();
        let data = String::from_utf8(
            data[first_null_position+1..first_null_position+1+datalength as usize]
            .to_vec()).unwrap();
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
    let Message { username, time, data} = Message::decode(&msg.encode_data()).unwrap();
    assert_eq!(username ,msg.username);
    assert_eq!(time ,msg.time);
    assert_eq!(data ,msg.data);
}