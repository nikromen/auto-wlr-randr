use anyhow::{Context, Result, anyhow};
use libc;
use mio::net::UnixListener;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Reload,
    Status,
    Switch(String),
}

pub struct SocketListener {
    pub listener: UnixListener,
    path: PathBuf,
}

impl SocketListener {
    pub fn bind<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        ensure_socket_dir_exists()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let listener = UnixListener::bind(&path)
            .with_context(|| format!("Failed to bind to socket at {}", path.display()))?;
        Ok(Self { listener, path })
    }
}

impl Drop for SocketListener {
    fn drop(&mut self) {
        log::debug!("Cleaning up socket file at {}", self.path.display());
        if let Err(e) = std::fs::remove_file(&self.path)
            && self.path.exists()
        {
            log::error!("Failed to remove socket file: {e}");
        }
    }
}

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(format!("{runtime_dir}/auto-wlr-randr/auto-wlr-randr.sock"));
    }

    let uid = unsafe { libc::getuid() };
    PathBuf::from(format!(
        "/run/user/{uid}/auto-wlr-randr/auto-wlr-randr.sock"
    ))
}

pub fn ensure_socket_dir_exists() -> std::io::Result<()> {
    if let Some(dir) = get_socket_path().parent()
        && !dir.exists()
    {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub fn handle_client_request<F>(listener: &SocketListener, command_handler: F) -> Result<String>
where
    F: FnOnce(Command) -> Result<String>,
{
    match listener.listener.accept() {
        Ok((mut stream, _)) => {
            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer)?;
            let command: Command = serde_json::from_slice(&buffer)
                .map_err(|e| anyhow!("Failed to deserialize command: {}", e))?;
            log::info!("Received command: {command:?}");

            let result = command_handler(command);

            let response_json = match &result {
                Ok(msg) => serde_json::to_string(&Ok::<_, String>(msg.clone()))?,
                Err(e) => serde_json::to_string(&Err::<String, _>(e.to_string()))?,
            };

            if let Err(e) = stream.write_all(response_json.as_bytes()) {
                log::error!("Failed to send response: {e}");
            }

            result
        }
        Err(e) => {
            // Only log the error if it's not EAGAIN/EWOULDBLOCK (Resource temporarily unavailable)
            if e.kind() != std::io::ErrorKind::WouldBlock {
                log::error!("Failed to accept IPC connection: {e}");
            }
            Err(anyhow!("Failed to accept IPC connection: {}", e))
        }
    }
}
