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
}

pub fn fetch_server_list() -> Result<Vec<OpenMpServer>, String> {
    let url = "https://api.open.mp/servers/";
    
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().map_err(|e| e.to_string())?;
    let servers: Vec<OpenMpServer> = resp.json().map_err(|e| e.to_string())?;
    
    Ok(servers)
}
