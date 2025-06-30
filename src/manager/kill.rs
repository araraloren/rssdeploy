use cote::prelude::*;

use super::AppContext;

#[derive(Debug, Cote)]
#[cote(shellcomp, aborthelp, width = 50, overload, notexit)]
pub struct Kill {
    /// Kill all shadowsocks instance
    pub all: bool,

    /// Kill given shadowsocks instance
    #[arg(alias = "-i", value = 0usize)]
    pub index: Option<usize>,
}

impl Kill {
    pub async fn invoke_cmd(&self, ac: &mut AppContext) -> color_eyre::Result<()> {
        if self.all {
            for inst in ac.insts.iter_mut() {
                inst.ss.kill().await?;
                if let Some(kcp) = inst.kcp.as_mut() {
                    kcp.kill().await?;
                }
            }
            ac.insts.clear();
        } else {
            let index = self.index.unwrap();
            let inst = ac
                .insts
                .get_mut(index)
                .ok_or_else(|| color_eyre::Report::msg("Index out of bound, no instance found"))?;

            inst.ss.kill().await?;
            if let Some(kcp) = inst.kcp.as_mut() {
                kcp.kill().await?;
            }
            ac.insts.remove(index);
        }

        Ok(())
    }
}
