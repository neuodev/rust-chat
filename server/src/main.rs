use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::mpsc;
use std::thread;
use std::env;


const MSG_SIZE: usize = 32;

struct Client {
    socket: TcpStream,
    addr: SocketAddr,
}
struct  Message {
    from: SocketAddr, 
    msg: String,
}

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

// Welcome message 
// Send to all user expect the one who sent it!
fn main() {
    let args = env::args().collect::<Vec<String>>();
    let ip_addr = args[1].clone();
    let server = TcpListener::bind(ip_addr.clone()).expect("Listener failed to bind");
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");
    println!("Server is running on {ip_addr}");
    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<Message>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);
            let tx = tx.clone();
            let s = socket.try_clone().expect("Failed to clone client");
            clients.push(Client{socket: s, addr});

            thread::spawn(move || loop {
                let mut buf = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buf) {
                    Ok(_) => {
                        let msg = buf.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf-8 message");
                        println!("{} {:?}", addr, msg);
                        tx.send(Message { from: addr, msg }).expect("Failed to send msg to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (), // continue  
                    Err(_) => {
                        println!("Closing Connection with {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                if client.addr != msg.from {
                    return client.socket.write_all(&buff).map(|_| client).ok();
                };

                Some(client)
            }).collect::<Vec<_>>();
        }

        sleep()
    }
}
