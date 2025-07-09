use color_eyre::eyre::eyre;
use cote::prelude::*;

use super::AppContext;

#[derive(Debug, Cote)]
#[cote(shellcomp, aborthelp, width = 50, overload, notexit)]
pub struct Kill {
    /// Kill all shadowsocks instance
    pub all: bool,

    /// Kill given shadowsocks instance
    #[arg(alias = "-i", value = 0usize)]
    pub id: Option<usize>,
}

impl Kill {
    pub async fn invoke_cmd(&self, ctx: &mut AppContext) -> color_eyre::Result<()> {
        for inst in ctx
            .insts
            .iter_mut()
            .filter(|v| self.all || Some(v.id) == self.id)
        {
            inst.ss.kill().await?;
            if let Some(kcp) = inst.kcp.as_mut() {
                kcp.kill().await?;
            }
        }
        if self.all {
            ctx.insts.clear();
        } else if let Some(index) = ctx.insts.iter().position(|v| Some(v.id) == self.id) {
            ctx.insts.remove(index);
        } else if let Some(index) = self.id {
            return Err(eyre!("Invalid id `{index}`, no instance found"));
        }

        Ok(())
    }
}
