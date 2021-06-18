use std::collections::{HashMap, hash_map};
use std::io::{BufReader, Read, Write};
use std::process::exit;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread::spawn;

use librustchat::protocol::{Command, Framing};
use librustchat::protocol::PacketError;
use librustchat::user::User;

struct Session {
    connected_peers: HashMap<SocketAddr, TcpStream>,
    connected_users: HashMap<SocketAddr, User>
}

//TODO: 서버측에서는 모니터링 스레드를 만들어서 현재 무슨 세션이 연결되어 있는지 확인할수 있게 할것.
impl Session {
    fn new() -> Self {
        let connected_peers = HashMap::new();
        let connected_users = HashMap::new();
        Session {
            connected_peers,
            connected_users
        }
    }

    fn peer_iter(&self) -> hash_map::Iter<'_,SocketAddr, TcpStream> {
        self.connected_peers.iter()
    }

    fn peer_iter_mut(&mut self) -> hash_map::IterMut<'_, SocketAddr, TcpStream> {
        self.connected_peers.iter_mut()
    }

    fn user_iter(&self) -> hash_map::Iter<'_,SocketAddr,User> {
        self.connected_users.iter()
    }

    //이렇게 남발해도 효율이 괜찮게 나올진 모르겠네..
    fn find_user(&self, peer: &SocketAddr) -> Option<&User> {
        self.connected_users.get(peer)
    }

    fn find_peer_stream(&self, peer: &SocketAddr) -> Option<TcpStream> {
        self.connected_peers.get(peer).and_then(|s| Some(
            s.try_clone().unwrap()
        ))
    }

    fn add_peer(&mut self, stream: TcpStream) -> std::io::Result<()>{
        let peer = stream.peer_addr()?;
        self.connected_peers.insert(peer, stream);
        Ok(())
    }

    fn add_user_session(&mut self, peer: SocketAddr, user: User) {
        self.connected_users.insert(peer, user);
    }

    fn delete_peer(&mut self, peer: &SocketAddr) {
        let user = self.connected_users.remove(&peer); 
        if let Some(u) = user {
            println!("logout: {}[peer: {}]", u.get_username(), peer);
        }
        let s = self.connected_peers.remove(&peer);
        if let Some(s) = s{
                s.shutdown(std::net::Shutdown::Both);
        }
    }
}
enum HandlerMessage {
    NewConnection(TcpStream),
    CloseConnection(SocketAddr),
    Msg(Command, SocketAddr)
}

fn process_message(com: Command, session: &mut Session, peer: SocketAddr) {
    match com {
        Command::Login(u) => {
            session.add_user_session(peer, u);
        },
        Command::Message(data) => {
            let user = session.find_user(&peer)
                .and_then(|x| Some(x.get_username()));
            //나중에 서버 에러타입 추가해서 로그인 안된경우도 Result로 리턴하게끔..
            if let Some(username) = user {
                if username == data.get_sender() {
                    println!("{}", data.to_str());
                    for (_,v) in session.peer_iter_mut() {
                        v.write(&data.encode_data()).unwrap();
                    }
                }
            }
        },
        _ => {
            
        }
    }
}
//각 클라이언트의 데이터를 받아 처리하는 스레드.
fn message_handler(rx: mpsc::Receiver<HandlerMessage>) {
    let mut session = Session::new();
    println!("handler start..");
    for received in rx {
        match received {
            HandlerMessage::Msg(msg, addr) => {
                process_message(msg, &mut session, addr)
            },
            HandlerMessage::NewConnection(stream) => {
                let addr = stream.peer_addr().unwrap();
                println!("new_connection!: {}", addr);
                session.add_peer(stream).unwrap();
            },
            HandlerMessage::CloseConnection(ref addr) => {
                session.delete_peer(addr);
            }
        }
    }
    println!("message handler out.");
}

fn client_handler(stream: TcpStream, tx: mpsc::Sender<HandlerMessage>) {
    let addr = stream.peer_addr().unwrap();
    let mut reader = stream.try_clone().unwrap().bytes();
    tx.send(HandlerMessage::NewConnection(stream.try_clone().unwrap())).unwrap();

    loop {
        let message = Command::decode(&mut reader);
        match message {
            Ok(message) =>  tx.send(HandlerMessage::Msg(message, addr.clone())).unwrap(),
            Err(PacketError::DisconnectError) => {
                tx.send(HandlerMessage::CloseConnection(addr)).unwrap();
                break;
            }
            Err(PacketError::ParseError) => {
                eprintln!("cannot parse!");
            }
        }
    }
}

fn main(){
    let arguments : Vec<String> = std::env::args().collect();

    if arguments.len() < 3 {
        eprintln!("usage: {} ip port", arguments[0]);
        exit(1);
    }

    let ip : Ipv4Addr = arguments[1].parse()
        .expect("not a ip address!");

    let port: u16 = arguments[2].parse()
        .expect("not a port number!");

    let listener = TcpListener::bind(SocketAddr::from((ip, port))).unwrap();
    let (client_tx, server_rx) =  mpsc::channel();

    println!("listening: {}:{}", ip, port);
    let msg_handler = spawn(move || {
        message_handler(server_rx);
    });
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let tx_clone = client_tx.clone();
        spawn(|| {
            client_handler(stream, tx_clone);
        });
    }
    msg_handler.join().unwrap();
}