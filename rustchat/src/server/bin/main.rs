use std::io::{BufRead, BufReader, BufWriter};
use std::io::Write;
use std::process::exit;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::thread::spawn;

//각 클라이언트의 데이터를 받아 처리하는 스레드.
fn message_handler() {

}
fn client_handler(stream: TcpStream) {
    let addr = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    println!("addr: {}", addr);
    loop {
        let mut s = String::new();
        reader.read_line(&mut s).unwrap();
        if s.eq("end\n") {
            break;
        }
        writer.write(s.as_bytes()).unwrap();
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

    println!("listening: {}:{}")
    //let mut userlist: Vec<User> = Vec::new();
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        spawn(|| {
            client_handler(stream);
        });
    }
}