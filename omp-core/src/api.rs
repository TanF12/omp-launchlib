use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenMpServer {
    pub ip: String,
    pub hn: String,
    pub pc: u32,
    pub pm: u32,
    pub gm: String,
    pub la: String,
    pub pa: bool,
    pub vn: String,
    pub omp: bool,
    pub pr: bool,
    pub ru: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize)]
struct ApiOpenMpServer {
    pub ip: Option<String>,
    pub hn: Option<String>,
    pub pc: Option<u32>,
    pub pm: Option<u32>,
    pub gm: Option<String>,
    pub la: Option<String>,
    pub pa: Option<bool>,
    pub vn: Option<String>,
    pub omp: Option<bool>,
    pub pr: Option<bool>,
    pub ru: Option<std::collections::HashMap<String, String>>,
}

pub fn fetch_server_list(url: &str) -> Result<Vec<OpenMpServer>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().map_err(|e| e.to_string())?;

    let api_servers: Vec<ApiOpenMpServer> = resp.json().map_err(|e| e.to_string())?;

    let servers = api_servers
        .into_iter()
        .filter_map(|s| {
            let ip = s.ip?;
            Some(OpenMpServer {
                ip,
                hn: s.hn.unwrap_or_default(),
                pc: s.pc.unwrap_or(0),
                pm: s.pm.unwrap_or(0),
                gm: s.gm.unwrap_or_default(),
                la: s.la.unwrap_or_default(),
                pa: s.pa.unwrap_or(false),
                vn: s.vn.unwrap_or_default(),
                omp: s.omp.unwrap_or(false),
                pr: s.pr.unwrap_or(false),
                ru: s.ru,
            })
        })
        .collect();

    Ok(servers)
}
