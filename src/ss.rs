use std::path::PathBuf;

use color_eyre::eyre::bail;
use color_eyre::eyre::Ok;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Config {
    server: String,

    port: String,

    password: String,

    timeout: i32,

    method: String,

    fast_open: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub enum KcpMode {
    Fast3,

    Fast2,

    #[default]
    Fast,

    Normal,

    Manual,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, Debug, Default)]
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

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct KcpConfig {
    listen: String,

    local: String,

    crypt: Crypt,

    key: String,

    send: i32,

    recv: i32,

    mtu: i32,

    mode: KcpMode,

    dscp: i32,

    data_shard: i32,

    parity_shard: i32,

    comp: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ShadowSocks {
    path: PathBuf,

    bin: PathBuf,

    kcp: Option<PathBuf>,

    err_log: PathBuf,

    out_log: PathBuf,

    kcp_log: PathBuf,

    cfg: Config,

    kcp_cfg: Option<KcpConfig>,
}

impl ShadowSocks {
    const BIN: &'static str = "server";

    const KCP: &'static str = "kcp";

    const CFG: &'static str = "config.json";

    const KCP_CFG: &'static str = "kcp.json";

    const ERR_LOG: &'static str = "stderr.log";

    const OUT_LOG: &'static str = "stdout.log";

    const KCP_LOG: &'static str = "kcp.log";

    pub fn load(path: PathBuf) -> color_eyre::Result<Self> {
        let bin_symlink = path.join(Self::BIN);
        let kcp_symlink = path.join(Self::KCP);
        let err_log = path.join(Self::ERR_LOG);
        let out_log = path.join(Self::OUT_LOG);
        let kcp_log = path.join(Self::KCP_LOG);
        let kcp_cfg = path.join(Self::KCP_CFG);
        let cfg = path.join(Self::CFG);
        let bin = if bin_symlink.exists() {
            if bin_symlink.is_symlink() {
                bin_symlink.read_link()?
            } else {
                bin_symlink
            }
        } else {
            bail!("Can not find `server` in `{:?}`", path);
        };

        Ok(Self {
            path,
            bin,
            kcp: if kcp_symlink.exists() {
                Some(if kcp_symlink.is_symlink() {
                    kcp_symlink.read_link()?
                } else {
                    kcp_symlink
                })
            } else {
                None
            },
            err_log,
            out_log,
            kcp_log,
            cfg: {
                let cfg = std::fs::read_to_string(&cfg)?;

                serde_json::from_str(&cfg)?
            },
            kcp_cfg: {
                let cfg = std::fs::read_to_string(&kcp_cfg)?;

                serde_json::from_str(&cfg)?
            },
        })
    }

    pub fn write_cfg(&self) -> color_eyre::Result<()> {
        let kcp_cfg = self.path.join(Self::KCP_CFG);
        let cfg = self.path.join(Self::CFG);
        let bin_symlink = self.path.join(Self::BIN);
        let kcp_symlink = self.path.join(Self::KCP);

        std::fs::write(cfg, serde_json::to_string(&self.cfg)?)?;
        std::fs::write(kcp_cfg, serde_json::to_string(&self.kcp_cfg)?)?;
        if self.bin.parent() != Some(&self.path) {
            symlink::symlink_file(&self.bin, bin_symlink)?;
        }
        if let Some(kcp) = &self.kcp {
            if kcp.parent() != Some(&self.path) {
                symlink::symlink_file(kcp, kcp_symlink)?;
            }
        }
        Ok(())
    }

    pub fn set_path(&mut self, path: PathBuf) -> &mut Self {
        self.path = path;
        self
    }

    pub fn set_bin(&mut self, path: PathBuf) -> &mut Self {
        self.bin = path;
        self
    }

    pub fn set_kcp(&mut self, kcp: PathBuf) -> &mut Self {
        self.kcp = Some(kcp);
        self
    }

    pub fn set_cfg(&mut self, cfg: Config) -> &mut Self {
        self.cfg = cfg;
        self
    }

    pub fn set_kcp_cfg(&mut self, kcp: KcpConfig) -> &mut Self {
        self.kcp_cfg = Some(kcp);
        self
    }
}
