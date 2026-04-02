use crate::config::ServerConfig;
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

pub struct SshClient {
    session: Session,
}

impl SshClient {
    pub fn connect(config: &ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        if let Some(key) = &config.key_path {
            let key_path = Path::new(key);
            let expanded_path = if key.starts_with("~/") {
                if let Some(base_dirs) = directories::BaseDirs::new() {
                    base_dirs.home_dir().join(key.strip_prefix("~/").unwrap())
                } else {
                    key_path.to_path_buf()
                }
            } else {
                key_path.to_path_buf()
            };
            session.userauth_pubkey_file(
                &config.username,
                None,
                &expanded_path,
                config.password.as_deref(),
            )?;
        } else if let Some(password) = &config.password {
            session.userauth_password(&config.username, password)?;
        } else {
            session.userauth_agent(&config.username)?;
        }
        if !session.authenticated() {
            return Err("SSH authentication failed".into());
        }
        Ok(Self { session })
    }

    pub fn execute(&self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut channel = self.session.channel_session()?;
        channel.exec(command)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        channel.wait_close()?;
        Ok(s)
    }
}
