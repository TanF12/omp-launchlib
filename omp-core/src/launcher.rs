use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

#[derive(Deserialize)]
pub struct LaunchConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub password: Option<String>,
    pub game_path: String,
    pub dll_path: String,
    pub omp_dll_path: Option<String>,
    pub is_wine: bool,
    pub wine_prefix: Option<String>,
    pub injector_exe_path: String,
}

#[derive(Serialize)]
pub struct LaunchResult {
    pub success: bool,
    pub error: Option<String>,
}

pub fn launch_game(config: LaunchConfig) -> LaunchResult {
    let mut dlls = vec![config.dll_path];
    if let Some(ref omp) = config.omp_dll_path
        && !omp.is_empty()
    {
        dlls.push(omp.clone());
    }

    let mut args = vec![
        format!("{}/gta_sa.exe", config.game_path),
        dlls.len().to_string(),
    ];
    args.extend(dlls);

    args.extend(vec![
        "-c".into(),
        "-n".into(),
        config.name,
        "-h".into(),
        config.host,
        "-p".into(),
        config.port.to_string(),
    ]);

    if let Some(pwd) = config.password
        && !pwd.is_empty()
    {
        args.extend(vec!["-z".into(), pwd]);
    }

    let mut cmd = if config.is_wine {
        let mut c = Command::new("wine");
        c.arg(&config.injector_exe_path).args(&args);
        c.env("WINEDLLOVERRIDES", "dinput8,samp,omp-client=n,b");
        if let Some(prefix) = config.wine_prefix
            && !prefix.is_empty()
        {
            c.env("WINEPREFIX", prefix);
        }
        c
    } else {
        let mut c = Command::new(&config.injector_exe_path);
        c.args(&args);
        c
    };

    cmd.current_dir(&config.game_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    match cmd.spawn() {
        Ok(_) => LaunchResult {
            success: true,
            error: None,
        },
        Err(e) => LaunchResult {
            success: false,
            error: Some(e.to_string()),
        },
    }
}
