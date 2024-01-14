use std::fs::create_dir_all;
use std::fs::File;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;

use color_eyre::eyre::Report;
use cote::*;

use crate::kill::Kill;
use crate::kill::KillInternalApp;
use crate::list::List;
use crate::list::ListInternalApp;
use crate::load::Loader;
use crate::load::LoaderInternalApp;
use crate::ss_display_help;
use crate::start::Start;
use crate::start::StartInternalApp;
use prettytable::Row;
use prettytable::Table;
use serde::Deserialize;
use serde::Serialize;
use serde_json::to_string_pretty;

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug, Default, CoteVal, CoteOpt)]
pub enum Method {
    #[default]
    Blake3ChaCha20Poly1305_2022,

    Aes128,

    Aes256,

    ChaCha20IetfFPoly1305,

    Blake3Aes128_2022,

    Blake3Aes256_2022,

    Plain,

    None,
}

impl ToString for Method {
    fn to_string(&self) -> String {
        String::from(match self {
            Method::Blake3ChaCha20Poly1305_2022 => "2022-blake3-chacha20-poly1305",
            Method::Aes128 => "aes-128-gcm",
            Method::Aes256 => "aes-256-gcm",
            Method::ChaCha20IetfFPoly1305 => "chacha20-ietf-poly1305",
            Method::Blake3Aes128_2022 => "2022-blake3-aes-128-gcm",
            Method::Blake3Aes256_2022 => "2022-blake3-aes-256-gcm",
            Method::Plain => "plain",
            Method::None => "none",
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SsConfig {
    server: String,

    #[serde(rename = "server_port")]
    port: u32,

    password: String,

    timeout: u32,

    method: Method,

    fast_open: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug, Default, CoteVal, CoteOpt)]
pub enum KcpMode {
    Fast3,

    Fast2,

    #[default]
    Fast,

    Normal,

    Manual,
}

impl ToString for KcpMode {
    fn to_string(&self) -> String {
        String::from(match self {
            KcpMode::Fast3 => "Fast3",
            KcpMode::Fast2 => "Fast2",
            KcpMode::Fast => "Fast",
            KcpMode::Normal => "Normal",
            KcpMode::Manual => "Manual",
        })
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Debug, Default, CoteVal, CoteOpt)]
pub enum Crypt {
    #[default]
    Aes,

    Aes128,

    Aes192,

    Salsa20,

    BlowFish,

    TwoFish,

    Cast5,

    Des3,

    Tea,

    XTea,

    Xor,

    Sm4,

    None,
}

impl ToString for Crypt {
    fn to_string(&self) -> String {
        String::from(match self {
            Crypt::Aes => "Aes",
            Crypt::Aes128 => "Aes128",
            Crypt::Aes192 => "Aes192",
            Crypt::Salsa20 => "Salsa20",
            Crypt::BlowFish => "BlowFish",
            Crypt::TwoFish => "TwoFish",
            Crypt::Cast5 => "Cast5",
            Crypt::Des3 => "3Des",
            Crypt::Tea => "Tea",
            Crypt::XTea => "XTea",
            Crypt::Xor => "Xor",
            Crypt::Sm4 => "Sm4",
            Crypt::None => "None",
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct KcpConfig {
    server: String,

    crypt: Crypt,

    key: String,

    send_wnd: u32,

    recv_wnd: u32,

    mtu: u32,

    mode: KcpMode,

    dscp: u32,

    data_shard: u32,

    parity_shard: u32,

    comp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    bin: PathBuf,

    kcp: PathBuf,

    err_log: PathBuf,

    out_log: PathBuf,

    kcp_log: PathBuf,

    cfg: SsConfig,

    kcp_cfg: Option<KcpConfig>,
}

impl Config {
    pub fn start_server(&self, start: &Start) -> color_eyre::Result<Instance> {
        let cfg = &self.cfg;
        let bin = start.bin.as_ref().unwrap_or(&self.bin);
        let bin = shellexpand::path::full(bin.as_path())?;
        let mut cmd = Command::new(&*bin);

        let ss_port = start.port.unwrap_or(cfg.port);
        let server = format!("{}:{}", &cfg.server, ss_port);
        let password = start.password.as_ref().unwrap_or(&cfg.password);
        let timeout = start.timeout.unwrap_or(cfg.timeout);
        let method = start.method.unwrap_or(cfg.method);
        let fast_open = start.fast_open || cfg.fast_open;
        let out_log = start.out_log.as_ref().unwrap_or(&self.out_log);
        let out_log = shellexpand::path::full(out_log.as_path())?;
        let err_log = start.err_log.as_ref().unwrap_or(&self.err_log);
        let err_log = shellexpand::path::full(err_log.as_path())?;

        cmd.arg("-s")
            .arg(server)
            .arg("-k")
            .arg(password)
            .arg("-m")
            .arg(method.to_string())
            .arg("--timeout")
            .arg(timeout.to_string());
        if fast_open {
            cmd.arg("--tcp-fast-open");
        }
        if let Some(out_log) = out_log.parent() {
            create_dir_all(out_log)?
        }
        if let Some(err_log) = err_log.parent() {
            create_dir_all(err_log)?
        }
        cmd.stdout(
            File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(out_log)?,
        );
        cmd.stderr(
            File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(err_log)?,
        );

        dbg!(&cmd);

        let ss = cmd.spawn()?;
        let mut kcp = None;

        if start.enable_kcp {
            if let Some(cfg) = &self.kcp_cfg {
                let bin = &self.kcp;
                let bin = shellexpand::path::full(bin.as_path())?;
                let mut cmd = Command::new(&*bin);

                // listen to ss server and port
                let kcp_server = format!("{}:{}", cfg.server, ss_port);
                let kcp_port = start.listen.unwrap_or(ss_port + 1);
                let send_wnd = start.send_wnd.unwrap_or(cfg.send_wnd);
                let recv_wnd = start.recv_wnd.unwrap_or(cfg.recv_wnd);
                let dscp = start.dscp.unwrap_or(cfg.dscp);
                let ds = start.data_shard.unwrap_or(cfg.data_shard);
                let ps = start.parity_shard.unwrap_or(cfg.parity_shard);
                let mode = start.mode.unwrap_or(cfg.mode);
                let mtu = start.mtu.unwrap_or(cfg.mtu);
                let no_comp = !start.compress || cfg.comp;
                let kcp_log = start.kcp_log.as_ref().unwrap_or(&self.kcp_log);
                let kcp_log = shellexpand::path::full(kcp_log.as_path())?;

                cmd.arg("-l")
                    .arg(format!(":{}", kcp_port))
                    .arg("-t")
                    .arg(kcp_server)
                    .arg("-crypt")
                    .arg(cfg.crypt.to_string())
                    .arg("-key")
                    .arg(&cfg.key)
                    .arg("-sndwnd")
                    .arg(send_wnd.to_string())
                    .arg("-rcvwnd")
                    .arg(recv_wnd.to_string())
                    .arg("-dscp")
                    .arg(dscp.to_string())
                    .arg("-datashard")
                    .arg(ds.to_string())
                    .arg("-parityshard")
                    .arg(ps.to_string())
                    .arg("-mode")
                    .arg(mode.to_string())
                    .arg("-mtu")
                    .arg(mtu.to_string());
                if no_comp {
                    cmd.arg("-nocomp");
                }
                if let Some(kcp_log) = kcp_log.parent() {
                    create_dir_all(kcp_log)?
                }
                cmd.stderr(
                    File::options()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(kcp_log)?,
                );
                kcp = Some(cmd.spawn()?);
            }
        }

        Ok(Instance {
            index: start.index,
            ss,
            kcp,
        })
    }
}

#[derive(Debug)]
pub struct Instance {
    pub index: usize,

    pub ss: Child,

    pub kcp: Option<Child>,
}

#[derive(Debug, Default)]
pub struct Manager {
    configs: Vec<Config>,

    instances: Vec<Instance>,
}

impl Manager {
    const LOAD: &'static str = "load";
    const START: &'static str = "start";
    const HELP: &'static str = "help";
    const LIST: &'static str = "list";
    const KILL: &'static str = "kill";
    const START_ST: &'static str = "s";
    const HELP_ST: &'static str = "h";
    const LIST_ST: &'static str = "ls";

    pub fn set_configs(&mut self, configs: Vec<Config>) -> &mut Self {
        self.configs = configs;
        self
    }

    pub fn invoke_cmd(&mut self, args: &[&str]) -> color_eyre::Result<()> {
        let cmd = args
            .first()
            .cloned()
            .ok_or_else(|| color_eyre::Report::msg(format!("Command line too short: {args:?}")))?;

        match cmd {
            Self::LOAD => self.invoke_config_cmd(args),
            Self::START | Self::START_ST => self.invoke_start_cmd(args),
            Self::HELP | Self::HELP_ST => self.invoke_help_cmd(args),
            Self::LIST | Self::LIST_ST => self.invoke_list_cmd(args),
            Self::KILL => self.invoke_kill_cmd(args),
            _ => {
                eprintln!("Unkonwn command line `{:?}`", args);
                Ok(())
            }
        }
    }

    pub fn invoke_config_cmd(&mut self, args: &[&str]) -> color_eyre::Result<()> {
        let args = Args::from(args.iter().copied());
        let loader = Loader::parse(args)?;
        let path = loader.config.unwrap();
        let path = shellexpand::full(&path)?;
        let cont = std::fs::read_to_string(&*path)?;

        self.set_configs(serde_json::from_str(&cont)?);

        Ok(())
    }

    pub fn invoke_start_cmd(&mut self, args: &[&str]) -> color_eyre::Result<()> {
        let args = Args::from(args.iter().copied());
        let start = Start::parse(args)?;
        let config = self.configs.get(start.index).ok_or_else(|| {
            Report::msg("Index out of bound, load the configurations using command `config`")
        })?;

        self.instances.push(config.start_server(&start)?);

        Ok(())
    }

    pub fn invoke_help_cmd(&mut self, args: &[&str]) -> color_eyre::Result<()> {
        #[derive(Debug, Cote)]
        #[cote(width = 50, overload)]
        pub struct Help {
            /// Show help message of given command
            #[pos()]
            command: String,
        }

        let args = Args::from(args.iter().copied());
        let res = Help::parse(args)?;

        match res.command.as_str() {
            Self::START | Self::START_ST => {
                ss_display_help!(Start, StartInternalApp);
            }
            Self::LOAD => {
                ss_display_help!(Loader, LoaderInternalApp);
            }
            Self::HELP | Self::HELP_ST => {
                ss_display_help!(Help, HelpInternalApp);
            }
            Self::KILL => {
                ss_display_help!(Kill, KillInternalApp);
            }
            Self::LIST | Self::LIST_ST => {
                ss_display_help!(List, ListInternalApp);
            }
            _ => {
                println!("Available commands: start, load, list, kill, help. Try help <Command>")
            }
        }

        Ok(())
    }

    pub fn invoke_list_cmd(&mut self, args: &[&str]) -> color_eyre::Result<()> {
        let args = Args::from(args.iter().copied());
        let list = List::parse(args)?;

        if list.local {
            println!("-------------------CONFIG------------------------");
            for (index, cfg) in self.configs.iter().enumerate() {
                println!("INDEX: {}", index);
                println!("{}", to_string_pretty(cfg)?);
                println!("-----------------------------------------------");
            }
        } else {
            let mut table = Table::new();

            table.add_row(Row::from(["Config", "Shadowsock", "Kcptun"]));
            for inst in self.instances.iter() {
                table.add_row(Row::from(vec![
                    inst.index.to_string(),
                    inst.ss.id().to_string(),
                    format!("{:?}", inst.kcp.as_ref().map(|v| v.id())),
                ]));
            }
            table.printstd();
        }

        Ok(())
    }

    pub fn invoke_kill_cmd(&mut self, args: &[&str]) -> color_eyre::Result<()> {
        let args = Args::from(args.iter().copied());
        let kill = Kill::parse(args)?;
        let inst = self
            .instances
            .get_mut(kill.index)
            .ok_or_else(|| Report::msg("Index out of bound, no instance found"))?;

        inst.ss.kill()?;
        inst.kcp.as_mut().map(|v| v.kill()).transpose()?;

        Ok(())
    }
}
