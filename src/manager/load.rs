use cote::prelude::*;
use tokio::fs::read_to_string;

use super::AppContext;

#[derive(Debug, Cote)]
#[cote(aborthelp, width = 50, overload, notexit)]
pub struct Load {
    /// Set the path of configuration
    #[arg(alias = "-c", value = "~/shadowsocks.json")]
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
