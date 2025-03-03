use std::fmt::Display;
use std::path::PathBuf;

use cote::aopt::raise_error;
use cote::prelude::CoteOpt;
use cote::prelude::CoteVal;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SsConfig {
    pub server: String,

    pub server_port: u32,

    pub password: String,

    pub timeout: u32,

    pub method: Method,

    pub fast_open: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct KcpConfig {
    pub server: String,

    pub crypt: Crypt,

    pub key: String,

    pub send_wnd: u32,

    pub recv_wnd: u32,

    pub mtu: u32,

    pub mode: KcpMode,

    pub dscp: u32,

    pub data_shard: u32,

    pub parity_shard: u32,

    pub comp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeployConfig {
    pub bin: PathBuf,

    pub kcp: PathBuf,

    pub err_log: Option<PathBuf>,

    pub out_log: Option<PathBuf>,

    pub kcp_log: Option<PathBuf>,

    pub ss_cfg: SsConfig,

    pub kcp_cfg: Option<KcpConfig>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, Serialize, Deserialize, CoteVal, CoteOpt)]
#[serde(try_from = "&str")]
#[coteval(mapstr = TryFrom::try_from)]
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

impl<'a> TryFrom<&'a str> for Method {
    type Error = cote::Error;

    fn try_from(val: &'a str) -> Result<Self, Self::Error> {
        match val {
            "2022-blake3-chacha20-poly1305" | "Blake3ChaCha20Poly1305_2022" => {
                Ok(Method::Blake3ChaCha20Poly1305_2022)
            }
            "aes-128-gcm" | "Aes128" => Ok(Self::Aes128),
            "aes-256-gcm" | "Aes256" => Ok(Self::Aes256),
            "chacha20-ietf-poly1305" | "ChaCha20IetfFPoly1305" => Ok(Self::ChaCha20IetfFPoly1305),
            "2022-blake3-aes-128-gcm" | "Blake3Aes128_2022" => Ok(Self::Blake3Aes128_2022),
            "2022-blake3-aes-256-gcm" | "Blake3Aes256_2022" => Ok(Self::Blake3Aes256_2022),
            "plain" | "Plain" => Ok(Self::Plain),
            _ => Err(raise_error!("Unknown crypt method: {}", val)),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Blake3ChaCha20Poly1305_2022 => "2022-blake3-chacha20-poly1305",
                Method::Aes128 => "aes-128-gcm",
                Method::Aes256 => "aes-256-gcm",
                Method::ChaCha20IetfFPoly1305 => "chacha20-ietf-poly1305",
                Method::Blake3Aes128_2022 => "2022-blake3-aes-128-gcm",
                Method::Blake3Aes256_2022 => "2022-blake3-aes-256-gcm",
                Method::Plain => "plain",
                Method::None => "none",
            }
        )
    }
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

impl Display for KcpMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                KcpMode::Fast3 => "fast3",
                KcpMode::Fast2 => "fast2",
                KcpMode::Fast => "fast",
                KcpMode::Normal => "normal",
                KcpMode::Manual => "manual",
            }
        )
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

impl Display for Crypt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
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
            }
        )
    }
}
