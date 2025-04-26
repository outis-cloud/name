use maxminddb::{Reader, geoip2};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use trust_dns_proto::op::{Message, ResponseCode};
use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};
use tokio::net::UdpSocket;
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

// ساختار برای مدیریت ایندکس Round-Robin
struct ServerSelector {
    index: usize,
}

impl ServerSelector {
    fn new() -> Self {
        ServerSelector { index: 0 }
    }

    fn next_server<'a>(&mut self, servers: &'a [&'a str]) -> &'a str {
        let server = servers[self.index % servers.len()];
        self.index = (self.index + 1) % servers.len();
        server
    }
}

#[tokio::main]
async fn main() {
    // بارگذاری دیتابیس GeoIP
    let geo_db = Reader::open_readfile("GeoLite2-Country.mmdb").unwrap_or_else(|err| {
        eprintln!("Failed to open GeoLite2 database: {}", err);
        std::process::exit(1);
    });

    // تنظیم لیست سرورهای DNS برای هر کشور
    let dns_servers: HashMap<String, Vec<&str>> = vec![
        ("US".to_string(), vec!["8.8.8.8:53", "8.8.4.4:53"]),
        ("DE".to_string(), vec!["1.1.1.1:53", "1.0.0.1:53"]),
        ("DEFAULT".to_string(), vec!["0.0.0.0:53"]),
    ].into_iter().collect();

    // اشتراک‌گذاری geo_db و dns_servers بین تردها
    let geo_db = Arc::new(geo_db);
    let dns_servers = Arc::new(dns_servers);
    let server_selector = Arc::new(Mutex::new(ServerSelector::new()));

    // راه‌اندازی UDP socket برای دریافت درخواست‌ها
    let socket = UdpSocket::bind("0.0.0.0:53").await.unwrap_or_else(|err| {
        eprintln!("Failed to bind socket: {}", err);
        std::process::exit(1);
    });
    let socket = Arc::new(socket);

    let mut buf = [0; 512]; // بافر برای داده‌های دریافت Spirituals
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await.unwrap_or_else(|err| {
            eprintln!("Failed to receive data: {}", err);
            std::process::exit(1);
        });

        let request_data = buf[..len].to_vec();
        let socket = Arc::clone(&socket);
        let geo_db = Arc::clone(&geo_db);
        let dns_servers = Arc::clone(&dns_servers);
        let server_selector = Arc::clone(&server_selector);

        // پردازش درخواست به صورت غیرهمزمان
        tokio::spawn(async move {
            handle_client_request(
                &request_data,
                addr,
                &socket,
                &geo_db,
                &dns_servers,
                &server_selector,
            ).await;
        });
    }
}

fn get_country_from_ip<T>(geo_db: &Reader<Vec<u8>>, ip: IpAddr) -> Option<String> {
    if let Ok(geo_data) = geo_db.lookup(ip) {
        if let Some(country) = geo_data.as_ref().and_then(|g: &T| g.country()) {
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
    dns_servers: &Arc<HashMap<String, Vec<&str>>>,
    server_selector: &Arc<Mutex<ServerSelector>>,
) {
    let ip = client_addr.ip();
    let country = get_country_from_ip(geo_db, ip).unwrap_or_else(|| "DEFAULT".to_string());
    println!("Request from IP: {}, Country: {}", ip, country);

    // پارس درخواست DNS
    let request = match Message::from_vec(request_data) {
        Ok(msg) => msg,
        Err(err) => {
            eprintln!("Failed to parse DNS request: {}", err);
            send_error_response(socket, client_addr, request_data, ResponseCode::FormErr).await;
            return;
        }
    };

    // انتخاب سرور DNS با استفاده از Round-Robin
    let dns_server = {
        let mut selector = server_selector.lock().await;
        let servers = dns_servers.get(&country).unwrap_or_else(|| dns_servers.get("DEFAULT").unwrap());
        selector.next_server(servers)
    };

    // ارسال درخواست به سرور DNS و دریافت پاسخ
    match forward_dns_request(&request, dns_server).await {
        Ok(response) => {
            // ارسال پاسخ به کلاینت
            let response_data = response.to_vec().unwrap_or_default();
            socket
                .send_to(&response_data, client_addr)
                .await
                .unwrap_or_else(|err| {
                    eprintln!("Failed to send response to client: {}", err);
                    0
                });
        }
        Err(err) => {
            eprintln!("DNS forwarding failed: {}", err);
            send_error_response(socket, client_addr, request_data, ResponseCode::ServFail).await;
        }
    }
}

async fn forward_dns_request(request: &Message, server: &str) -> io::Result<Message> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(server).await?;

    // ارسال درخواست
    let request_data = request.to_vec()?;
    socket.send(&request_data).await?;

    // دریافت پاسخ
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
    if let Ok(mut request) = Message::from_vec(request_data) {
        let mut response = Message::new();
        response.set_id(request.id());
        response.set_response_code(code);
        response.set_message_type(trust_dns_proto::op::MessageType::Response);
        if let Ok(response_data) = response.to_vec() {
            socket
                .send_to(&response_data, client_addr)
                .await
                .unwrap_or_else(|err| {
                    eprintln!("Failed to send error response: {}", err);
                    0
                });
        }
    }
}