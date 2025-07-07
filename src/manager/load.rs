use cote::prelude::*;
use tokio::fs::read_to_string;

use super::AppContext;

pub const DEFAULT_CONFIG: &str = "~/shadowsocks.json";

#[derive(Debug, Cote)]
#[cote(shellcomp, aborthelp, width = 50, overload, notexit)]
pub struct Load {
    /// Set the path of configuration
    #[arg(alias = "-c", value = DEFAULT_CONFIG)]
    pub config: Option<String>,
}

impl Load {
    pub async fn invoke_cmd(&self, ac: &mut AppContext) -> color_eyre::Result<()> {
        let path = self.config.as_ref().unwrap();
        let path = shellexpand::full(&path)?;

        ac.cfgs = serde_json::from_str(&read_to_string(&*path).await?)?;

        Ok(())
    }
}
