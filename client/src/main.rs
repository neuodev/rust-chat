use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn sleep(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn user_input() -> String {
    let mut buff = String::new();
    io::stdin().read_line(&mut buff).expect("Failed to read from the stdin");
    buff.trim().to_string()
}

fn display_msg(msg: String) {
    let parts = msg.split(":").collect::<Vec<&str>>();
    if parts.len() == 1 {
        println!("{msg}");
    } else {
        let user = parts[0];
        let msg = parts[1];

        println!("{user}:{msg}");
    }
}

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Failed to connect to the server");
    client.set_nonblocking(true).expect("Failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf-8");
                display_msg(msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Server disconnected");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.write_all(&buff).expect("Write to the socket failed");
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        sleep(100);
    });

    println!("What is yout name?");
    let username = user_input();
    tx.send(String::from(format!("{username} joined our chat!"))).expect("Failed to send the reciver message");
    
    println!("Write a message: ");
    loop {
        let msg = user_input();
        let msg = username.clone() + ": ".into() + &msg;
        tx.send(msg).expect("Failed to send message to the receiver.");
    }
}
