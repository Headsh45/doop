use rand::{rngs::OsRng, RngCore};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub struct PortableState {
    child: Mutex<Option<Child>>,
}

impl Default for PortableState {
    fn default() -> Self {
        Self {
            child: Mutex::new(None),
        }
    }
}

fn portable_root() -> io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "portable executable has no parent directory"))
}

fn find_port() -> io::Result<u16> {
    for port in 4400..4450 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AddrNotAvailable,
        "no free local port found between 4400 and 4449",
    ))
}

fn load_or_create_secret(config_dir: &Path) -> io::Result<String> {
    fs::create_dir_all(config_dir)?;
    let path = config_dir.join("auth.secret");
    if path.exists() {
        let mut value = String::new();
        fs::File::open(&path)?.read_to_string(&mut value)?;
        let value = value.trim().to_string();
        if !value.is_empty() {
            return Ok(value);
        }
    }

    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let secret = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let mut file = fs::File::create(&path)?;
    file.write_all(secret.as_bytes())?;
    Ok(secret)
}

pub fn start(state: &PortableState) -> io::Result<String> {
    let root = portable_root()?;
    let app_dir = root.join("app");
    let runtime_dir = root.join("runtime");
    let data_dir = root.join("data");
    let config_dir = root.join("config");
    let logs_dir = root.join("logs");

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&logs_dir)?;

    let bun = runtime_dir.join(if cfg!(target_os = "windows") { "bun.exe" } else { "bun" });
    let server_entry = app_dir.join("server").join("index.ts");

    if !bun.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("portable Bun runtime not found: {}", bun.display()),
        ));
    }
    if !server_entry.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("portable Doop server not found: {}", server_entry.display()),
        ));
    }

    let port = find_port()?;
    let base_url = format!("http://127.0.0.1:{port}");
    let secret = load_or_create_secret(&config_dir)?;

    let stdout = OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs_dir.join("server.log"))?;
    let stderr = stdout.try_clone()?;

    let mut command = Command::new(&bun);
    command
        .arg(&server_entry)
        .current_dir(&app_dir)
        .env("NODE_ENV", "production")
        .env("HOST", "127.0.0.1")
        .env("PORT", port.to_string())
        .env("BETTER_AUTH_URL", &base_url)
        .env("BETTER_AUTH_SECRET", secret)
        .env("DOOP_PORTABLE", "1")
        .env("DOOP_DATA_DIR", &data_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW

    let mut child = command.spawn()?;
    let deadline = Instant::now() + Duration::from_secs(20);

    loop {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        if let Some(status) = child.try_wait()? {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("portable Doop server exited during startup with {status}; see logs/server.log"),
            ));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "portable Doop server did not become ready within 20 seconds; see logs/server.log",
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }

    *state.child.lock().expect("portable child mutex poisoned") = Some(child);
    Ok(base_url)
}

pub fn shutdown(state: &PortableState) {
    let Ok(mut guard) = state.child.lock() else {
        return;
    };
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}
