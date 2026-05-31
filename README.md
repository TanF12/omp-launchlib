# open.mp Core & Injector

A backend library and Windows DLL injector created to abstract away DLL injection and server querying for SA-MP and open.mp launchers. 

This repository consists of two components:
1. **`omp-core`**: A cross-platform Rust library (`cdylib`) that exposes a clean JSON-over-FFI C-ABI for querying servers, fetching the masterlist, and managing game launches.
2. **`omp-injector`**: A lightweight, native Windows executable designed to safely suspend `gta_sa.exe`, inject required multiplayer DLLs (`samp.dll`, `omp-client.dll`) via `CreateRemoteThread`, and resume execution.

## Features
* **JSON FFI Boundary:** integrates with any frontend (Go, C#, Node, Python, etc.) without complex C-struct memory mapping.
* **FFI:** Wraps all exported C-functions in `catch_unwind` boundaries to guarantee frontend stability even on network or parsing panics.
* **Blazing Fast Queries:** Utilizes my [samp-query](https://github.com/TanF12/samp-query) library for high-throughput, multi-threaded UDP batch querying.
* **Native Windows Injection:** Uses Win32 APIs to boot the game in a suspended state.

## Building from Source

**Prerequisites:** 
* Rust toolchain (1.80+)
* *Note: The injector must be built as a 32-bit binary because `gta_sa.exe` is a stricly 32-bit app.*

```bash
# Add the 32-bit Windows target (Required for injector)
rustup target add i686-pc-windows-msvc

# Build the DLL Injector (Windows only)
cargo build --package omp-injector --release --target i686-pc-windows-msvc

# Build the Core Shared Library (Cross-platform)
cargo build --package omp-core --release

## API Usage
The `omp-core` library exposes the following standard C-ABI functions:
* `char* omp_core_fetch_servers()`
* `char* omp_core_query_server(const char* ip, uint16_t port)`
* `char* omp_core_query_batch(const char* json_targets)`
* `char* omp_core_launch(const char* config_json)`
* `void omp_core_free_string(char* s)`

**Memory Rule:** Any string returned by `omp_core_*` **must** be freed by the caller using `omp_core_free_string` to avoid memory leaks.
```