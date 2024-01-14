use cote::*;

#[derive(Debug, Cote)]
#[cote(width = 50, overload)]
pub struct Loader {
    /// Set the path of configuration
    #[arg(alias = "-c", value = "~/shadowsocks.json")]
    pub config: Option<String>,
}
