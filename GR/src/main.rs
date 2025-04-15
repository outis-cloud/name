use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use maxminddb::{geoip2, Reader};

use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
struct IpGeoData {
    ip: String,
    country: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeoRouting {
    geo_data: HashMap<String, String>,
}

impl GeoRouting {
    fn new() -> Self {
        let mut geo_data = HashMap::new();
        geo_data.insert("IR".to_string(), "8.8.8.8".to_string());
        GeoRouting { geo_data }
    }

    fn route_request(&self, country: &str) -> Option<String> {
        self.geo_data.get(country).cloned()
    }
}

async fn handle_request(
    mut stream: tokio::net::TcpStream,
    geo_routing: Arc<GeoRouting>,
    geoip_reader: Arc<Reader<Vec<u8>>>,
) {
    let mut buffer = [0; 1024];
    let _ = stream.peek(&mut buffer).await;

    let ip = match stream.peer_addr() {
        Ok(addr) => addr.ip(),
        Err(_) => {
            let _ = stream.write_all("Failed to get IP".as_bytes()).await;
            return;
        }
    };

    let country_code = geoip_reader
        .lookup::<geoip2::Country>(ip)
        .ok()
        .and_then(|country| {
            country
                .and_then(|c| c.country)
                .and_then(|c| c.iso_code.map(|s| s.to_string()))
        });

        match country_code {
            Some(code) => {
                println!("User from country: {}", code);
        
                if code == "IR" {
                    if let Some(server) = geo_routing.route_request(&code) {
                        let message = format!("Redirecting to: {}", server);
                        let _ = stream.write_all(message.as_bytes()).await;
                        return;
                    }
                }
        
                let _ = stream.write_all("Welcome! You're connected to the default server.".as_bytes()).await;
            }
            None => {
                let _ = stream.write_all("Could not determine country".as_bytes()).await;
            }
        }
        
}

#[tokio::main]
async fn main() {
    let geo_routing = Arc::new(GeoRouting::new());
    let geoip_reader = Arc::new(
        Reader::open_readfile("./GeoLite2-Country.mmdb").expect("Could not open GeoIP DB"),
    );

    let listener = TcpListener::bind("0.0.0.0:8182")
        .await
        .expect("Failed to bind");

    println!("Listening on 0.0.0.0:8182");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let geo_routing = Arc::clone(&geo_routing);
        let geoip_reader = Arc::clone(&geoip_reader);

        tokio::spawn(async move {
            handle_request(stream, geo_routing, geoip_reader).await;
        });
    }
}
