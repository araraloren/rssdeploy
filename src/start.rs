use std::path::PathBuf;

use crate::manager::{KcpMode, Method};
use cote::*;

#[derive(Debug, Cote)]
#[cote(width = 50, overload)]
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
    #[arg(alias = "-t")]
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
    #[arg(alias = "-sw", value = 2048u32)]
    pub send_wnd: Option<u32>,

    /// Set receive windows size
    #[arg(alias = "-rw", value = 2048u32)]
    pub recv_wnd: Option<u32>,

    /// Set mtu value
    #[arg(value = 1400u32)]
    pub mtu: Option<u32>,

    /// Set dscp value
    #[arg(value = 46u32)]
    pub dscp: Option<u32>,

    /// Set datashard value
    #[arg(alias = "-ds", value = 30u32)]
    pub data_shard: Option<u32>,

    /// Set parityshard value
    #[arg(alias = "-ps", value = 15u32)]
    pub parity_shard: Option<u32>,

    /// Set kcptun mode
    pub mode: Option<KcpMode>,

    /// Enable compress mode
    pub compress: bool,

    /// Set the log file path of kcp
    pub kcp_log: Option<PathBuf>,

    /// The index of configuration
    #[pos()]
    pub index: usize,
}
