use samp_query::{SampClient, query_info_batch};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInfo {
    pub target: Option<String>,
    pub hostname: String,
    pub players: u32,
    pub max_players: u32,
    pub gamemode: String,
    pub language: String,
    pub password: bool,
    pub ping_ms: u32,
    pub rules: Option<std::collections::HashMap<String, String>>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ClientResponse {
    pub id: u8,
    pub name: String,
    pub score: i32,
    pub ping: Option<u32>,
}

pub fn query_server(ip: &str, port: u16) -> Result<ServerInfo, String> {
    let target = format!("{}:{}", ip, port);
    let client = SampClient::new(Duration::from_secs(2)).map_err(|e| e.to_string())?;

    let info = client.get_info(&target).map_err(|e| e.to_string())?;
    let ping = client.get_ping(&target).unwrap_or(Duration::from_millis(0));

    let mut rules_map = std::collections::HashMap::new();
    if let Ok(rules) = client.get_rules(&target) {
        for rule in rules {
            rules_map.insert(rule.name, rule.value);
        }
    }

    Ok(ServerInfo {
        target: Some(target),
        hostname: info.hostname.to_string(),
        players: info.players as u32,
        max_players: info.max_players as u32,
        gamemode: info.gamemode.to_string(),
        language: info.mapname.to_string(),
        password: info.password,
        ping_ms: ping.as_millis() as u32,
        rules: Some(rules_map),
        error: None,
    })
}

pub fn query_batch(targets: Vec<String>) -> Result<Vec<ServerInfo>, String> {
    // 2s timeout, 1 retry, 5000 PPS, 4 DNS threads
    let results =
        query_info_batch(targets, Duration::from_secs(2), 1, 5000, 4).map_err(|e| e.to_string())?;

    let mut parsed = Vec::new();
    for res in results {
        match res.result {
            Ok(info) => parsed.push(ServerInfo {
                target: Some(res.original_input),
                hostname: info.hostname.to_string(),
                players: info.players as u32,
                max_players: info.max_players as u32,
                gamemode: info.gamemode.to_string(),
                language: info.mapname.to_string(),
                password: info.password,
                ping_ms: res.rtt.as_millis() as u32,
                rules: None,
                error: None,
            }),
            Err(e) => parsed.push(ServerInfo {
                target: Some(res.original_input),
                hostname: "".into(),
                players: 0,
                max_players: 0,
                gamemode: "".into(),
                language: "".into(),
                password: false,
                ping_ms: 0,
                rules: None,
                error: Some(e.to_string()),
            }),
        }
    }
    Ok(parsed)
}

pub fn query_clients(ip: &str, port: u16) -> Result<Vec<ClientResponse>, String> {
    let target = format!("{}:{}", ip, port);
    let client = SampClient::new(Duration::from_secs(2)).map_err(|e| e.to_string())?;

    match client.get_detailed_clients(&target) {
        Ok(detailed) => Ok(detailed
            .into_iter()
            .map(|c| ClientResponse {
                id: c.player_id,
                name: c.name,
                score: c.score,
                ping: if c.ping == 4294967295 || c.ping == 65535 {
                    None
                } else {
                    Some(c.ping)
                },
            })
            .collect()),
        Err(_) => {
            let basic = client.get_clients(&target).map_err(|e| e.to_string())?;
            Ok(basic
                .into_iter()
                .map(|c| ClientResponse {
                    id: 0,
                    name: c.name,
                    score: c.score,
                    ping: None,
                })
                .collect())
        }
    }
}
