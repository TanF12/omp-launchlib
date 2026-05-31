use crate::{api, launcher, query};
use serde_json::json;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;

fn to_c_char(s: String) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

macro_rules! safe_ffi {
    ($closure:expr) => {
        match panic::catch_unwind($closure) {
            Ok(result) => result,
            Err(_) => to_c_char(json!({ "error": "Rust panic caught at FFI boundary" }).to_string()),
        }
    };
}

/// Fetches the global server list from the open.mp API.
/// Returns a JSON string containing the server list.
#[unsafe(no_mangle)]
pub extern "C" fn omp_core_fetch_servers() -> *mut c_char {
    safe_ffi!(|| {
        match api::fetch_server_list() {
            Ok(servers) => to_c_char(serde_json::to_string(&servers).unwrap()),
            Err(e) => to_c_char(json!({ "error": e }).to_string()),
        }
    })
}

/// Queries a specific server for its live information.
///
/// # Safety
/// * `ip` must be a valid, null-terminated C string (or null).
/// * The returned pointer must be freed by the caller using `omp_core_free_string` to avoid memory leaks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omp_core_query_server(ip: *const c_char, port: u16) -> *mut c_char {
    safe_ffi!(|| {
        let ip_str = unsafe {
            if ip.is_null() {
                return to_c_char(json!({ "error": "IP is null" }).to_string());
            }
            CStr::from_ptr(ip).to_str().unwrap_or("")
        };
        match query::query_server(ip_str, port) {
            Ok(info) => to_c_char(serde_json::to_string(&info).unwrap()),
            Err(e) => to_c_char(json!({ "error": e }).to_string()),
        }
    })
}

/// Performs a high-performance batch query on multiple servers.
///
/// # Safety
/// * `json_targets` must be a valid, null-terminated C string containing a JSON string array of IP targets.
/// * The returned pointer must be freed by the caller using `omp_core_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omp_core_query_batch(json_targets: *const c_char) -> *mut c_char {
    safe_ffi!(|| {
        let json_str = unsafe {
            if json_targets.is_null() {
                return to_c_char(json!({ "error": "Targets null" }).to_string());
            }
            CStr::from_ptr(json_targets).to_str().unwrap_or("")
        };
        let targets: Vec<String> = match serde_json::from_str(json_str) {
            Ok(t) => t,
            Err(e) => return to_c_char(json!({ "error": e.to_string() }).to_string()),
        };
        match query::query_batch(targets) {
            Ok(results) => to_c_char(serde_json::to_string(&results).unwrap()),
            Err(e) => to_c_char(json!({ "error": e }).to_string()),
        }
    })
}

/// Launches the game (either natively or through Wine) with the injector.
///
/// # Safety
/// * `config_json` must be a valid, null-terminated C string containing a `LaunchConfig` JSON payload.
/// * The returned pointer must be freed by the caller using `omp_core_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omp_core_launch(config_json: *const c_char) -> *mut c_char {
    safe_ffi!(|| {
        let json_str = unsafe {
            if config_json.is_null() {
                return to_c_char(
                    json!({ "success": false, "error": "Config is null" }).to_string(),
                );
            }
            CStr::from_ptr(config_json).to_str().unwrap_or("")
        };
        let config: launcher::LaunchConfig = match serde_json::from_str(json_str) {
            Ok(c) => c,
            Err(e) => {
                return to_c_char(json!({ "success": false, "error": e.to_string() }).to_string());
            }
        };
        let result = launcher::launch_game(config);
        to_c_char(serde_json::to_string(&result).unwrap())
    })
}

/// Queries the player list for a specific server.
/// Returns a JSON string array of ClientResponse.
///
/// # Safety
/// * `ip` must be a valid, null-terminated C string (or null).
/// * The returned pointer must be freed by the caller using `omp_core_free_string` to avoid memory leaks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omp_core_query_clients(ip: *const c_char, port: u16) -> *mut c_char {
    safe_ffi!(|| {
        let ip_str = unsafe {
            if ip.is_null() {
                return to_c_char(json!({ "error": "IP is null" }).to_string());
            }
            CStr::from_ptr(ip).to_str().unwrap_or("")
        };
        
        match query::query_clients(ip_str, port) {
            Ok(clients) => to_c_char(serde_json::to_string(&clients).unwrap()),
            Err(e) => to_c_char(json!({ "error": e }).to_string()),
        }
    })
}

/// Safely deallocates a C-string previously allocated and returned by this library.
///
/// # Safety
/// * `s` must be a pointer returned by one of the `omp_core_*` functions.
/// * `s` must not be freed more than once (no double-frees).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn omp_core_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        } // Re-takes ownership and drops it
    }
}
