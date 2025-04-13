use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt; 
use maxminddb::geoip2;

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


async fn handle_request(mut stream: tokio::net::TcpStream, geo_routing: &GeoRouting) {
    let mut buffer = [0; 1024];
    let _ = stream.peek(&mut buffer).await;

    let country = "IR"; 

    
    let reader = maxminddb::Reader::open_readfile("./GeoCountry.mmdb").unwrap();

    if let Some(server) = geo_routing.route_request(country) {
        let message = format!("Redirecting to: {}", server);
        let _ = stream.write_all(message.as_bytes())
            .await
            .map_err(|ـ| "Failed to respond");
    } else {
        let _ = stream.write_all("No route available".as_bytes()).await;
    }
}


#[tokio::main]
async fn main() {
    let geo_routing = GeoRouting::new();
    let listener = TcpListener::bind("0.0.0.0:8182")
        .await
        .expect("Failed to bind");

    println!("Listening on 0.0.0.0:8182");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let geo_routing = geo_routing.clone();

        tokio::spawn(async move {
            handle_request(stream, &geo_routing).await;
        });
    }
}
