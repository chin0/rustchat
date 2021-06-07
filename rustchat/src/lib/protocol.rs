use crate::user::User;

enum Command {
    Register = 0x30,  //회원가입
    Login,
    JoinChatRoom, //채팅방에 접속
    CreateChatRoom,
    Message, //메시지 전송
}