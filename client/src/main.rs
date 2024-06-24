use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::io::{self, BufReader};
use tokio::io::AsyncBufReadExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let server_addr = "127.0.0.1:8080".parse::<SocketAddr>()?;
        
    let stdin = io::stdin();
    
    let mut reader = BufReader::new(stdin);
    
    let mut buf = String::from("CONNECTING");
    socket.send_to(buf.as_bytes(), &server_addr).await?;
    buf.clear();
    let mut buf2 = vec![0u8; 1024];

    loop {
        tokio::select! {
            n = reader.read_line(&mut buf) => {
                match n {
                    Ok(0) => break,
                    Ok(_) => {
                        socket.send_to(buf.as_bytes(), &server_addr).await?;
                    },
                    Err(e) => return Err(e.into()),
                }
                buf.clear();
            },

            res = socket.recv_from(&mut buf2) => {
                let (n, _addr) = match res {
                    Ok((n, _addr)) => (n, _addr),
                    Err(e) => {
                        eprintln!("Error receiving message: {:?}", e);
                        continue;
                    }
                };
                if let Ok(msg) = std::str::from_utf8(&buf2[..n]) {
                    print!("- {}", msg);
                }
                buf.clear();
            },
        }
    }

    Ok(())
}
