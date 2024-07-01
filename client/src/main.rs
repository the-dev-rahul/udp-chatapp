use tokio::net::UdpSocket;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::Arc;
use tokio::time::{Duration, interval};

const DISCONNECT_MESSAGE: &str = "!DISCONNECT";
const HEARTBEAT_MESSAGE: &str = "!HEARTBEAT";
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:0").await?);
    socket.connect("127.0.0.1:8080").await?;

    println!("Connected to server. Start typing messages (type '!DISCONNECT' to quit):");

    let socket_clone = Arc::clone(&socket);
    let receive_handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 1024];
        loop {
            match socket_clone.recv(&mut buf).await {
                Ok(len) => {
                    let message = String::from_utf8_lossy(&buf[..len]);
                        println!("{}", message);
                }
                Err(e) => eprintln!("Failed to receive message: {}", e),
            }
        }
    });

    let socket_clone = Arc::clone(&socket);
    let heartbeat_handle = tokio::spawn(async move {
        let mut interval = interval(HEARTBEAT_INTERVAL);
        loop {
            interval.tick().await;
            let _ = socket_clone.send(HEARTBEAT_MESSAGE.as_bytes()).await;
        }
    });

    let mut stdin: tokio::io::Lines<BufReader<tokio::io::Stdin>> = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = stdin.next_line().await? {
        if line == DISCONNECT_MESSAGE {
            socket.send(DISCONNECT_MESSAGE.as_bytes()).await?;
            break;
        }
        socket.send(line.as_bytes()).await?;
    }

    // Clean up
    receive_handle.abort();
    heartbeat_handle.abort();
    println!("Disconnected from server.");

    Ok(())
}