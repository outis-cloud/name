use maxminddb::{geoip2, Reader};
use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::process::exit;
use std::sync::Arc;
use tokio::net::UdpSocket;
use trust_dns_proto::op::{Message, ResponseCode};

#[tokio::main]
async fn main() {
    let geo_db = Reader::open_readfile("GeoLite2-Country.mmdb").unwrap_or_else(|err| {
        println!("Failed to open GeoLite2 database: {}", err);
        exit(1);
    });

    let dns_servers: HashMap<String, &str> = vec![
        ("US".to_string(), "192.168.1.10:53"),
        ("DE".to_string(), "192.168.1.11:53"),
        ("DEFAULT".to_string(), "0.0.0.0:53"),
    ]
    .into_iter()
    .collect();

    let geo_db = Arc::new(geo_db);
    let dns_servers = Arc::new(dns_servers);

    let socket = UdpSocket::bind("0.0.0.0:53").await.unwrap_or_else(|err| {
        println!("Failed to bind socket: {}", err);
        exit(1);
    });
    let socket = Arc::new(socket);

    let mut buf = [0; 512];
    loop {
        let result = socket.recv_from(&mut buf).await;
        match result {
            Ok((len, addr)) => {
                let request_data = buf[..len].to_vec();
                let socket = Arc::clone(&socket);
                let geo_db = Arc::clone(&geo_db);
                let dns_servers = Arc::clone(&dns_servers);

                tokio::spawn(async move {
                    handle_client_request(&request_data, addr, &socket, &geo_db, &dns_servers)
                        .await;
                });
            }
            Err(err) => {
                println!("Failed to receive data: {}", err);
                continue;
            }
        }
    }
}

fn get_country_from_ip(geo_db: &Reader<Vec<u8>>, ip: IpAddr) -> Option<String> {
    if let Ok(geo_data) = geo_db.lookup::<geoip2::Country>(ip) {
        if let Some(country) = geo_data.unwrap().country {
            return Some(country.iso_code.unwrap_or_default().to_string());
        }
    }
    None
}

async fn handle_client_request(
    request_data: &[u8],
    client_addr: SocketAddr,
    socket: &Arc<UdpSocket>,
    geo_db: &Arc<Reader<Vec<u8>>>,
    dns_servers: &Arc<HashMap<String, &str>>,
) {
    let ip = client_addr.ip();
    let country = get_country_from_ip(geo_db, ip).unwrap_or_else(|| "DEFAULT".to_string());
    println!("Request from IP: {}, Country: {}", ip, country);

    let request = match Message::from_vec(request_data) {
        Ok(msg) => msg,
        Err(err) => {
            println!("Failed to parse DNS request: {}", err);
            send_error_response(socket, client_addr, request_data, ResponseCode::FormErr).await;
            return;
        }
    };

    let dns_server = dns_servers
        .get(&country)
        .unwrap_or_else(|| dns_servers.get("DEFAULT").unwrap());

    match forward_dns_request(&request, dns_server).await {
        Ok(response) => {
            let response_data = response.to_vec().unwrap_or_default();
            socket
                .send_to(&response_data, client_addr)
                .await
                .unwrap_or_else(|err| {
                    println!("Failed to send response to client: {}", err);
                    0
                });
        }
        Err(err) => {
            println!("DNS forwarding failed: {}", err);
            send_error_response(socket, client_addr, request_data, ResponseCode::ServFail).await;
        }
    }
}

async fn forward_dns_request(request: &Message, server: &str) -> io::Result<Message> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(server).await?;

    let request_data = request.to_vec()?;
    socket.send(&request_data).await?;

    let mut buf = [0; 512];
    let len = socket.recv(&mut buf).await?;
    Message::from_vec(&buf[..len]).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

async fn send_error_response(
    socket: &Arc<UdpSocket>,
    client_addr: SocketAddr,
    request_data: &[u8],
    code: ResponseCode,
) {
    if let Ok(request) = Message::from_vec(request_data) {
        let mut response = Message::new();
        response.set_id(request.id());
        response.set_response_code(code);
        response.set_message_type(trust_dns_proto::op::MessageType::Response);
        if let Ok(response_data) = response.to_vec() {
            socket
                .send_to(&response_data, client_addr)
                .await
                .unwrap_or_else(|err| {
                    println!("Failed to send error response: {}", err);
                    0
                });
        }
    }
}
