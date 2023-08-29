use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::process::{Command, Stdio};
use std::{path::PathBuf, process::Child};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    server: String,
    port: String,
    password: String,
    timeout: i32,
    method: String,
    fast_open: bool,
}

#[derive(Debug, Default)]
pub struct Shadowsocks {
    binary: PathBuf,
    config: Config,
    proc: Option<Child>,
}

impl Shadowsocks {
    pub fn new(binary: PathBuf, config: Config) -> Self {
        Self {
            binary,
            config,
            proc: None,
        }
    }

    pub fn binary(&self) -> &PathBuf {
        &self.binary
    }

    pub fn proc(&self) -> Option<&Child> {
        self.proc.as_ref()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn id(&self) -> u32 {
        self.proc.as_ref().map(|v| v.id()).unwrap_or(0)
    }

    pub fn with_server(mut self, server: String) -> Self {
        self.config.server = server;
        self
    }

    pub fn with_port(mut self, port: String) -> Self {
        self.config.port = port;
        self
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.config.password = password;
        self
    }

    pub fn with_timeout(mut self, timeout: i32) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn with_method(mut self, method: String) -> Self {
        self.config.method = method;
        self
    }

    pub fn with_fast_open(mut self, fast_open: bool) -> Self {
        self.config.fast_open = fast_open;
        self
    }

    pub fn start(&mut self, out: Option<Stdio>, err: Option<Stdio>) -> std::io::Result<&mut Self> {
        let mut command = Command::new(&self.binary);

        command
            .arg("-s")
            .arg(format!("{}:{}", &self.config.server, &self.config.port))
            .arg("-k")
            .arg(&self.config.password)
            .arg("-m")
            .arg(&self.config.method)
            .arg("--timeout")
            .arg(format!("{}", self.config.timeout));
        if self.config.fast_open {
            command.arg("--tcp-fast-open");
        }
        if let Some(out) = out {
            command.stdout(out);
        }
        if let Some(err) = err {
            command.stderr(err);
        }
        self.proc = Some(command.spawn()?);
        Ok(self)
    }

    pub fn kill(&mut self) -> std::io::Result<()> {
        self.proc
            .as_mut()
            .ok_or_else(|| std::io::Error::from(ErrorKind::Other))?
            .kill()
    }
}

#[derive(Debug, Default)]
pub struct Kcptun {
    binary: PathBuf,
    dscp: u32,
    datashard: u32,
    parityshard: u32,
    sndwnd: u32,
    rcvwnd: u32,
    mode: String,
    mtu: u32,
    server: String,
    port: String,
    crypt: String,
    listen: String,
    password: String,
    proc: Option<Child>,
}

impl Kcptun {
    pub fn binary(&self) -> &PathBuf {
        &self.binary
    }

    pub fn dscp(&self) -> u32 {
        self.dscp
    }

    pub fn datashard(&self) -> u32 {
        self.datashard
    }

    pub fn parityshard(&self) -> u32 {
        self.parityshard
    }

    pub fn sndwnd(&self) -> u32 {
        self.sndwnd
    }

    pub fn rcvwnd(&self) -> u32 {
        self.rcvwnd
    }

    pub fn mode(&self) -> &String {
        &self.mode
    }

    pub fn mtu(&self) -> u32 {
        self.mtu
    }

    pub fn server(&self) -> &String {
        &self.server
    }

    pub fn port(&self) -> &String {
        &self.port
    }

    pub fn crypt(&self) -> &String {
        &self.crypt
    }

    pub fn listen(&self) -> &String {
        &self.listen
    }

    pub fn password(&self) -> &String {
        &self.password
    }

    pub fn with_binary(mut self, binary: PathBuf) -> Self {
        self.binary = binary;
        self
    }

    pub fn with_dscp(mut self, dscp: u32) -> Self {
        self.dscp = dscp;
        self
    }

    pub fn with_datashard(mut self, datashard: u32) -> Self {
        self.datashard = datashard;
        self
    }

    pub fn with_parityshard(mut self, parityshard: u32) -> Self {
        self.parityshard = parityshard;
        self
    }

    pub fn with_sndwnd(mut self, sndwnd: u32) -> Self {
        self.sndwnd = sndwnd;
        self
    }

    pub fn with_rcvwnd(mut self, rcvwnd: u32) -> Self {
        self.rcvwnd = rcvwnd;
        self
    }

    pub fn with_mode(mut self, mode: String) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = mtu;
        self
    }

    pub fn with_server(mut self, server: String) -> Self {
        self.server = server;
        self
    }

    pub fn with_port(mut self, port: String) -> Self {
        self.port = port;
        self
    }

    pub fn with_crypt(mut self, crypt: String) -> Self {
        self.crypt = crypt;
        self
    }

    pub fn with_listen(mut self, listen: String) -> Self {
        self.listen = listen;
        self
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.password = password;
        self
    }

    pub fn start(&mut self, out: Option<Stdio>, err: Option<Stdio>) -> std::io::Result<&mut Self> {
        let mut command = Command::new(&self.binary);

        command
            .arg("-l")
            .arg(format!(":{}", &self.listen))
            .arg("-t")
            .arg(format!("{}:{}", &self.server, &self.port))
            .arg("-crypt")
            .arg(&self.crypt)
            .arg("-key")
            .arg(&self.password)
            .arg("-nocomp")
            .arg("-dscp")
            .arg(format!("{}", &self.dscp))
            .arg("-datashard")
            .arg(format!("{}", &self.datashard))
            .arg("-parityshard")
            .arg(format!("{}", &self.parityshard))
            .arg("-sndwnd")
            .arg(format!("{}", &self.sndwnd))
            .arg("-rcvwnd")
            .arg(format!("{}", &self.rcvwnd))
            .arg("-mode")
            .arg(&self.mode)
            .arg("-mtu")
            .arg(format!("{}", &self.mtu));
        if let Some(out) = out {
            command.stdout(out);
        }
        if let Some(err) = err {
            command.stderr(err);
        }
        self.proc = Some(command.spawn()?);
        Ok(self)
    }
}
