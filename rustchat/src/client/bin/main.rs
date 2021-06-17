use std::io::Bytes;
use std::io::Read;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::process::exit;
use std::thread::spawn;

use librustchat::ui::events::{Event, Events};
use librustchat::data::Message;
use librustchat::protocol::Framing;
use librustchat::protocol::Command;
use librustchat::user::User;

fn display_thread(mut stream: Bytes<TcpStream>) {
    loop {
        let data = Message::decode(&mut stream).unwrap();
        println!("{}", data.to_str());
    }
}
fn main() -> std::io::Result<()>{
    let stdin = stdin();
    let arguments : Vec<String> = std::env::args().collect();

    if arguments.len() < 3 {
        eprintln!("usage: {} ip port", arguments[0]);
        exit(1);
    }

    let ip : Ipv4Addr = arguments[1].parse()
        .expect("not a ip address!");

    let port: u16 = arguments[2].parse()
        .expect("not a port number!");

    
    let mut conn = TcpStream::connect(SocketAddr::from((ip, port)))?;
    
    let mut username = String::new();
    let mut password = String::new();
    print!("username: ");
    stdout().flush()?;
    stdin.read_line(&mut username)?;
    print!("password: ");
    stdout().flush()?;
    stdin.read_line(&mut password)?;

    let request = Command::Login(User::new(&username, &password));
    conn.write(&request.encode_data())?;

    let reader = conn.try_clone().unwrap().bytes();
    let handle = spawn(|| {
        display_thread(reader);
    });

    loop {
        let mut buf = String::new();

        stdin.read_line(&mut buf)?;
        let msg = Message::new(&username, &buf);
        conn.write(&Command::Message(msg).encode_data())?;
        conn.flush()?;
    }
    //Ok(())
}