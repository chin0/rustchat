use std::process::exit;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};

use librustchat::user::User;

fn handler(stream: TcpStream) {

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
    let mut userlist: Vec<User> = Vec::new();
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let addr = stream.peer_addr().unwrap();
        println!("connected: {}", addr);
        handler(stream);
    }
}