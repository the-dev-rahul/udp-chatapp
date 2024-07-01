use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, interval};

const DISCONNECT_MESSAGE: &str = "!DISCONNECT";
const HEARTBEAT_MESSAGE: &str = "!HEARTBEAT";
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:8080").await?);
    let clients = Arc::new(Mutex::new(HashMap::new()));

    println!("Server listening on 127.0.0.1:8080");

    let clients_clone = Arc::clone(&clients);
    let socket_clone = Arc::clone(&socket);
    tokio::spawn(async move {
        let mut interval = interval(HEARTBEAT_INTERVAL);
        loop {
            interval.tick().await;
            check_inactive_clients(&clients_clone, &socket_clone).await;
        }
    });

    let mut buf = vec![0u8; 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let message = String::from_utf8_lossy(&buf[..len]);
        
        if message == DISCONNECT_MESSAGE {
            handle_disconnect(&clients, &addr, &socket).await?;
        } else {
            handle_message(&clients, &addr, &message, &socket).await?;
        }
    }
}

async fn handle_message(
    clients: &Arc<Mutex<HashMap<SocketAddr, (String, tokio::time::Instant)>>>,
    addr: &SocketAddr,
    message: &str,
    socket: &Arc<UdpSocket>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut clients_lock = clients.lock().await;
    
    if !clients_lock.contains_key(addr) {
        println!("New client connected: {} {}", addr, message);
        clients_lock.insert(*addr, (format!("{}{}{}",addr.ip(),":", addr.port()), tokio::time::Instant::now()));
    }
    
    let (sender_id, last_active) = clients_lock.get_mut(addr).unwrap();
    *last_active = tokio::time::Instant::now();
    let sender_id = sender_id.clone();
    drop(clients_lock);

    if message == HEARTBEAT_MESSAGE{
        return Ok(());
    }

    for (client_addr, _) in clients.lock().await.iter() {
        if client_addr != addr {
            let formatted_message = format!("{}: {}", sender_id, message);
            socket.send_to(formatted_message.as_bytes(), client_addr).await?;
        }
    }
    Ok(())
}

async fn handle_disconnect(
    clients: &Arc<Mutex<HashMap<SocketAddr, (String, tokio::time::Instant)>>>,
    addr: &SocketAddr,
    socket: &Arc<UdpSocket>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut clients_lock = clients.lock().await;
    if let Some((client_id, _)) = clients_lock.remove(addr) {
        println!("Client disconnected: {} ({})", client_id, addr);
        drop(clients_lock);
        
        for (client_addr, _) in clients.lock().await.iter() {
            if client_addr != addr {
                let disconnect_message = format!("Server: {} has disconnected", client_id);
                socket.send_to(disconnect_message.as_bytes(), client_addr).await?;
            }
        }
    }
    Ok(())
}

async fn check_inactive_clients(
    clients: &Arc<Mutex<HashMap<SocketAddr, (String, tokio::time::Instant)>>>,
    socket: &Arc<UdpSocket>,
) {
    let mut to_remove = Vec::new();
    let now = tokio::time::Instant::now();

    let clients_lock = clients.lock().await;
    for (&addr, &(ref client_id, last_active)) in clients_lock.iter() {
        println!("{:?}, {:?}", now.duration_since(last_active), CLIENT_TIMEOUT);
        if now.duration_since(last_active) > CLIENT_TIMEOUT {
            to_remove.push((addr, client_id.clone()));
        }
    }
    drop(clients_lock);

    for (addr, client_id) in to_remove {
        let mut clients_lock = clients.lock().await;
        clients_lock.remove(&addr);
        drop(clients_lock);

        println!("Client timed out: {} ({})", client_id, addr);

        for (client_addr, _) in clients.lock().await.iter() {
            if client_addr != &addr {
                let disconnect_message: String = format!("Server: {} has disconnected (timed out)", client_id);
                let _ = socket.send_to(disconnect_message.as_bytes(), client_addr).await;
            }
        }
    }
}