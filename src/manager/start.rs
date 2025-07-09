use std::env::temp_dir;
use std::path::PathBuf;

use cote::prelude::*;
use tokio::fs::{create_dir_all, read_to_string, write};
use tokio::process::Command;

use crate::config::{KcpMode, Method, SsConfig};

use super::AppContext;

#[derive(Debug, Cote)]
#[cote(shellcomp, aborthelp, width = 50, overload, notexit)]
pub struct Start {
    /// Set the path of ssserver
    #[arg(valid = valid!(|v: &PathBuf| v.exists()))]
    pub bin: Option<PathBuf>,

    /// Set the path of shadowsocks configuration
    #[arg(alias = "-c")]
    pub config: Option<PathBuf>,

    /// Set the listen port of ssserver
    #[arg(alias = "-p")]
    pub port: Option<u32>,

    /// Set the password of ssserver
    #[arg(alias = "-p")]
    pub password: Option<String>,

    /// Set the timeout of ssserver
    #[arg(alias = "-t")]
    pub timeout: Option<u32>,

    /// Set the timeout of ssserver
    #[arg(alias = "-t", scvalues = Method::values())]
    pub method: Option<Method>,

    /// Enable fast open for ssserver
    pub fast_open: bool,

    /// Set the log file path of ssserver
    pub out_log: Option<PathBuf>,

    /// Set the error log file path of ssserver
    pub err_log: Option<PathBuf>,

    /// Set the path of kcptun
    #[arg(valid = valid!(|v: &PathBuf| v.exists()))]
    pub kcp: Option<PathBuf>,

    /// Start a kcptun server at 127.0.0.1:{port + 1}
    #[arg(alias = "-k")]
    pub enable_kcp: bool,

    /// Set the listen port of kcptun, default is {port + 1}
    #[arg(alias = "-l")]
    pub listen: Option<u32>,

    /// Set send windows size
    #[arg(alias = "-sw", value = 2048u32, scvalues = ["2048"])]
    pub send_wnd: Option<u32>,

    /// Set receive windows size
    #[arg(alias = "-rw", value = 2048u32, scvalues = ["2048"])]
    pub recv_wnd: Option<u32>,

    /// Set mtu value
    #[arg(value = 1400u32, scvalues = ["1400"])]
    pub mtu: Option<u32>,

    /// Set dscp value
    #[arg(value = 46u32, scvalues = ["46"])]
    pub dscp: Option<u32>,

    /// Set datashard value
    #[arg(alias = "-ds", value = 30u32, scvalues = ["30"])]
    pub data_shard: Option<u32>,

    /// Set parityshard value
    #[arg(alias = "-ps", value = 15u32, scvalues = ["15"])]
    pub parity_shard: Option<u32>,

    /// Set kcptun mode
    #[arg(scvalues = KcpMode::values())]
    pub mode: Option<KcpMode>,

    /// Enable compress mode
    pub compress: bool,

    /// Set the log file path of kcp
    pub kcp_log: Option<PathBuf>,

    /// The index of configuration
    #[pos()]
    pub index: usize,
}

impl Start {
    pub async fn invoke_cmd(&self, ac: &mut AppContext) -> color_eyre::Result<()> {
        let deploy_cfg = ac.cfgs.get(self.index).ok_or_else(|| {
            color_eyre::Report::msg(
                "Index out of bound, load the configurations using command `load`",
            )
        })?;

        let bin = self.bin.as_ref().unwrap_or(&deploy_cfg.bin);
        let bin = shellexpand::path::full(bin.as_path())?;
        let mut cmd = Command::new(&*bin);
        let server_port;

        if let Some(config) = self.config.as_ref() {
            let config = shellexpand::path::full(config.as_path())?;
            let path = &*config;

            cmd.arg("-c").arg(path.to_str().unwrap_or_default());

            // read port
            let ss_config: SsConfig = serde_json::from_str(&read_to_string(path).await?)?;

            server_port = ss_config.server_port;
        } else {
            let ss_cfg = &deploy_cfg.ss_cfg;

            let server = &ss_cfg.server;
            let password = self.password.as_ref().unwrap_or(&ss_cfg.password);
            let timeout = self.timeout.unwrap_or(ss_cfg.timeout);
            let method = self.method.unwrap_or(ss_cfg.method);
            let fast_open = self.fast_open || ss_cfg.fast_open;
            let temp_dir = temp_dir();
            let temp_file = temp_dir.join(format!("ss_config_{}.json", self.index));

            server_port = self.port.unwrap_or(ss_cfg.server_port);

            write(
                &temp_file,
                serde_json::to_string_pretty(&SsConfig {
                    server: server.clone(),
                    server_port,
                    password: password.to_string(),
                    timeout,
                    method,
                    fast_open,
                })?,
            )
            .await?;

            cmd.arg("-c").arg(temp_file);
        }
        let out_log = self.out_log.as_ref().or(self.out_log.as_ref());
        let out_log = out_log.and_then(|v| shellexpand::path::full(v).ok());
        let err_log = self.err_log.as_ref().or(self.err_log.as_ref());
        let err_log = err_log.and_then(|v| shellexpand::path::full(v).ok());

        if let Some(out_log) = out_log {
            if let Some(out_log) = out_log.parent() {
                create_dir_all(out_log).await?
            }
            cmd.stdout(
                std::fs::File::options()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(out_log)?,
            );
        }
        if let Some(err_log) = err_log {
            if let Some(err_log) = err_log.parent() {
                create_dir_all(err_log).await?
            }

            cmd.stderr(
                std::fs::File::options()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(err_log)?,
            );
        }

        println!("start cmd => {cmd:?}");

        let ss = cmd.spawn()?;
        let mut kcp = None;

        if self.enable_kcp {
            if let Some(cfg) = &deploy_cfg.kcp_cfg {
                let bin = self.kcp.as_ref().unwrap_or(&deploy_cfg.kcp);
                let bin = shellexpand::path::full(bin.as_path())?;
                let mut cmd = Command::new(&*bin);

                // listen to ss server and port
                let kcp_server = format!("{}:{}", cfg.server, server_port);
                let kcp_port = self.listen.unwrap_or(server_port + 1);
                let send_wnd = self.send_wnd.unwrap_or(cfg.send_wnd);
                let recv_wnd = self.recv_wnd.unwrap_or(cfg.recv_wnd);
                let dscp = self.dscp.unwrap_or(cfg.dscp);
                let ds = self.data_shard.unwrap_or(cfg.data_shard);
                let ps = self.parity_shard.unwrap_or(cfg.parity_shard);
                let mode = self.mode.unwrap_or(cfg.mode);
                let mtu = self.mtu.unwrap_or(cfg.mtu);
                let no_comp = !self.compress || cfg.comp;
                let kcp_log = self.kcp_log.as_ref().or(self.kcp_log.as_ref());
                let kcp_log = kcp_log.and_then(|v| shellexpand::path::full(v).ok());

                cmd.arg("-l")
                    .arg(format!(":{kcp_port}"))
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
                if let Some(kcp_log) = kcp_log {
                    if let Some(kcp_log) = kcp_log.parent() {
                        create_dir_all(kcp_log).await?
                    }
                    cmd.stderr(
                        std::fs::File::options()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(kcp_log)?,
                    );
                }
                kcp = Some(cmd.spawn()?);
            }
        }

        ac.insts.push(crate::manager::SsInstance {
            id: self.index,
            ss,
            kcp,
        });

        Ok(())
    }
}
