use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter};
use std::io::Write;
use std::process::exit;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread::spawn;


struct Message {
    user: String,
    msg: String
}

impl Message {
    fn new(user: &str, message: &str) -> Self {
        let user = String::from(user);
        let msg = String::from(message);
        Message {
            user,
            msg 
        }
    }
}

enum HandlerMessage {
    NewConnection(TcpStream),
    CloseConnection(SocketAddr),
    Msg(Message)
}

impl HandlerMessage {
    fn new_message(user: &str, msg: &str) -> Self{
        Self::Msg(Message::new(user, msg))
    }
}

//각 클라이언트의 데이터를 받아 처리하는 스레드.
fn message_handler(rx: mpsc::Receiver<HandlerMessage>) {
    let mut connections: HashMap<SocketAddr, TcpStream> = HashMap::new();
    println!("reading message..");
    for received in rx {
        match received {
            HandlerMessage::Msg(Message { user, msg }) => {
                println!("{}:{}", user, msg);
                for (_,v) in connections.iter_mut() {
                    writeln!(v, "{}:{}", user, msg).unwrap();
                    v.flush().unwrap();
                }
            },
            HandlerMessage::NewConnection(stream) => {
                let addr = stream.peer_addr().unwrap();
                println!("new_connection!: {}", addr);
                connections.insert(addr, stream);
            },
            HandlerMessage::CloseConnection(ref addr) => {
               let stream = connections.remove(addr).unwrap();
               stream.shutdown(std::net::Shutdown::Both).unwrap();
            }
        }
    }
    println!("message handler out.");
}

fn client_handler(stream: TcpStream, tx: mpsc::Sender<HandlerMessage>) {
    let mut user = String::new();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    tx.send(HandlerMessage::NewConnection(stream.try_clone().unwrap())).unwrap();

    reader.read_line(&mut user).unwrap();

    loop {
        let mut message = String::new();
        reader.read_line(&mut message).unwrap();
        tx.send(HandlerMessage::new_message(user.trim(), message.trim())).unwrap();
        if message.eq("end\n") || message.len() == 0{
            tx.send(HandlerMessage::CloseConnection(stream.peer_addr().unwrap())).unwrap();
            break;
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
    //let mut userlist: Vec<User> = Vec::new();
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let tx_clone = client_tx.clone();
        spawn(|| {
            client_handler(stream, tx_clone);
        });
    }
    msg_handler.join().unwrap();
}