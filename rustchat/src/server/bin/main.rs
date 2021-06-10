use std::io::{BufRead, BufReader, BufWriter};
use std::io::Write;
use std::process::exit;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread::spawn;


//temp

#[derive(Clone)]
struct User {
    username: String
}

struct Message {
    user: User,
    message: String
}

//각 클라이언트의 데이터를 받아 처리하는 스레드.
fn message_handler(rx: mpsc::Receiver<Message>) {
    println!("reading message..");
    for received in rx {
        println!("{}: {}", received.user.username, received.message.trim());
    }
    println!("message handler out.");
}

fn client_handler(stream: TcpStream, tx: mpsc::Sender<Message>) {
    let addr = stream.peer_addr().unwrap();
    let mut user = User { username: String::new() };
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = BufWriter::new(stream.try_clone().unwrap());

    write!(writer, "username: ").unwrap();
    writer.flush().unwrap();
    reader.read_line(&mut user.username).unwrap();
    user.username = String::from(user.username.trim());

    println!("addr: {}", addr);
    loop {
        let mut message = String::new();
        reader.read_line(&mut message).unwrap();
        tx.send(Message{user: user.clone(), 
                        message: message.clone()}).unwrap();

        if message.eq("end\n") {
            break;
        }
        writer.write(message.as_bytes()).unwrap();
        writer.flush().unwrap();
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