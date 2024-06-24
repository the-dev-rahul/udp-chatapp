use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let socket = UdpSocket::bind(addr).await?;

    let mut hashmap: HashMap<String, SocketAddr> = HashMap::new();
    

    println!("Listening on: {}", addr);

    let mut buf = vec![0u8; 1024];

    loop {
        let (len, peer) = socket.recv_from(&mut buf).await?;

        let peer_str = format!("{}{}{}", peer.ip().to_string(), ":", peer.port().to_string());
        
        if ! hashmap.contains_key(&peer_str){
            hashmap.insert(peer_str.clone(), peer);
            continue;
        }


        for (key, value) in hashmap.iter(){
            if peer_str !=  *key{
                socket.send_to(&buf[..len], &value).await?;
            }
        }
    }
}