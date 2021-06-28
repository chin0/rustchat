use crate::user::User;

struct Chatroom {
    id: u64,
    name: String,
    member: Vec<User>,
}