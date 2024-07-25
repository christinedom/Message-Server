use std::io::{self, BufReader, BufWriter, Read, Write, ErrorKind};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() -> io::Result<()> {
    let client = TcpStream::connect(LOCAL)?;
    client.set_nonblocking(true)?;

    let (tx, rx) = mpsc::channel::<String>();
    let mut reader = BufReader::new(client.try_clone()?);
    let mut writer = BufWriter::new(client);

    thread::spawn(move || {
        loop {
            let mut buff = vec![0; MSG_SIZE];
            match reader.read_exact(&mut buff) {
                Ok(_) => {
                    let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    if let Ok(msg_str) = String::from_utf8(msg) {
                        println!("Received: {:?}", msg_str);
                    } else {
                        eprintln!("Received invalid UTF-8 message");
                    }
                },
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                },
                Err(_) => {
                    println!("Connection with server was severed");
                    break;
                }
            }

            match rx.try_recv() {
                Ok(msg) => {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    writer.write_all(&buff).expect("Failed to write to socket");
                    println!("Sent: {:?}", msg);
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }
        }
    });

    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff)?;
        let msg = buff.trim().to_string();
        if msg == "quit" || tx.send(msg).is_err() { break; }
    }

    println!("Goodbye!");
    Ok(())
}
