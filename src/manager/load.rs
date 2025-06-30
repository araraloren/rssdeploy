use std::env::current_dir;
use std::ffi::OsString;

use cote::prelude::*;
use cote::shell::value::repeat_values;
use cote::shell::value::Values;
use cote::Error;
use tokio::fs::read_to_string;

use super::AppContext;

pub const DEFAULT_CONFIG: &str = "~/shadowsocks.json";

#[derive(Debug, Cote)]
#[cote(shellcomp, aborthelp, width = 50, overload, notexit)]
pub struct Load {
    /// Set the path of configuration
    #[arg(alias = "-c", value = DEFAULT_CONFIG, scvalues = search_json_cfg())]
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

pub fn search_json_cfg<O>() -> impl Values<O, Err = Error> {
    repeat_values(|_| {
        let mut vals = vec![OsString::from(DEFAULT_CONFIG)];

        // search .json in current working directory
        if let Ok(read_dir) = current_dir().and_then(std::fs::read_dir) {
            for entry in read_dir {
                if let Ok(path) = entry.map(|v| v.path()) {
                    if path.extension().and_then(|v| v.to_str()) == Some("json") {
                        if let Some(filename) = path.file_name() {
                            let filename = filename.to_os_string();

                            if !vals.contains(&filename) {
                                vals.push(filename);
                            }
                        }
                    }
                }
            }
        }

        Ok(vals)
    })
}
