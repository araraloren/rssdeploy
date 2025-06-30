use cote::prelude::*;
use prettytable::{Row, Table};

use super::AppContext;

#[derive(Debug, Cote)]
#[cote(shellcomp, aborthelp, width = 50, overload, notexit)]
pub struct List {
    /// Instead, list the configuration
    #[arg(alias = "-l")]
    pub local: bool,
}

impl List {
    pub async fn invoke_cmd(&self, ac: &mut AppContext) -> color_eyre::Result<()> {
        if self.local {
            println!("-------------------CONFIG------------------------");
            for (index, cfg) in ac.cfgs.iter().enumerate() {
                println!("INDEX: {}", index);
                println!("{}", serde_json::to_string_pretty(cfg)?);
                println!("-----------------------------------------------");
            }
        } else {
            let mut table = Table::new();

            table.add_row(Row::from(["Config", "Shadowsock", "Kcptun"]));
            for inst in ac.insts.iter() {
                table.add_row(Row::from(vec![
                    inst.id.to_string(),
                    inst.ss.id().map(|v| v.to_string()).unwrap_or_default(),
                    format!("{:?}", inst.kcp.as_ref().map(|v| v.id())),
                ]));
            }
            table.printstd();
        }

        Ok(())
    }
}
