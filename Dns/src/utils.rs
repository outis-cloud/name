use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use tokio::sync::RwLock;
use tokio::time;

/// Health-check data structure: tracks upstream health
pub type Upstreams = Arc<RwLock<HashMap<SocketAddr, bool>>>;

/// Spawns a background task to periodically test each upstream
/// and update its health status.
pub fn spawn_health_checker(upstreams: Upstreams) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let mut map = upstreams.write().await;
            for (&addr, status) in map.iter_mut() {
                let healthy = tokio::time::timeout(
                    Duration::from_secs(2),
                    test_dns_query(&addr),
                )
                .await
                .map(|r| r.is_ok())
                .unwrap_or(false);
                *status = healthy;
            }
        }
    });
}

/// Performs a simple DNS query (e.g., NS query for example.com) to test health.
async fn test_dns_query(upstream: &SocketAddr) -> io::Result<()> {
    // Build a minimal DNS request for NS 'example.com'.
    use trust_dns_proto::op::{Message, Query, OpCode, MessageType};
    use trust_dns_proto::rr::{Name, RecordType};

    let mut msg = Message::new();
    msg.set_id(0);
    msg.set_message_type(MessageType::Query);
    msg.set_op_code(OpCode::Query);
    let name = Name::from_ascii("example.com.").unwrap();
    let query = Query::query(name, RecordType::NS);
    msg.add_query(query);
    let bytes = msg.to_vec().unwrap();

    super::forward_dns_request_raw(&bytes, upstream).await.map(|_| ())
}

/// Cache key: (raw_request, country_code)
pub type DnsCache = Arc<DashMap<(Vec<u8>, String), CacheEntry>>;

/// A single cache entry holding the raw response and expiration time
pub struct CacheEntry {
    pub response: Vec<u8>,
    pub expires_at: Instant,
}

/// Forwards a request with in-memory caching. Respects record TTL.
pub async fn cached_forward(
    request_data: &[u8],
    country: &str,
    upstream: SocketAddr,
    cache: DnsCache,
) -> io::Result<Vec<u8>> {
    let key = (request_data.to_vec(), country.to_string());
    if let Some(entry) = cache.get(&key) {
        if Instant::now() < entry.expires_at {
            return Ok(entry.response.clone());
        }
        cache.remove(&key);
    }

    let response = super::forward_dns_request_raw(request_data, &upstream).await?;
    let ttl = extract_min_ttl(&response).unwrap_or(60);
    cache.insert(key, CacheEntry {
        response: response.clone(),
        expires_at: Instant::now() + Duration::from_secs(ttl as u64),
    });
    Ok(response)
}

/// Extracts the minimum TTL from DNS message bytes using trust-dns-proto.
fn extract_min_ttl(response: &[u8]) -> Option<u32> {
    use trust_dns_proto::op::Message;
    use trust_dns_proto::rr::Record;
    if let Ok(msg) = Message::from_vec(response) {
        msg.answers()
            .iter()
            .map(|r: &Record| r.ttl())
            .min()
    } else {
        None
    }
}
