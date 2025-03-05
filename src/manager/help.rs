use cote::prelude::*;

use super::{kill::Kill, list::List, load::Load, start::Start, AppContext};

#[derive(Debug, Cote)]
#[cote(aborthelp, width = 50, overload, notexit)]
pub struct Help {
    /// Show help message of given command
    #[pos()]
    name: String,
}

impl Help {
    pub async fn invoke_cmd(&self, _ac: &mut AppContext) -> color_eyre::Result<()> {
        let cmds = [
            ("kill", Kill::into_parser()?, Kill::new_help_context()),
            ("list", List::into_parser()?, List::new_help_context()),
            ("load", Load::into_parser()?, Load::new_help_context()),
            ("start", Start::into_parser()?, Start::new_help_context()),
        ];

        for (name, parser, help_ctx) in &cmds {
            if &self.name == name {
                parser.display_help_ctx(help_ctx.clone())?;
                return Ok(());
            }
        }

        Err(color_eyre::Report::msg(format!(
            "Available commands are: {}",
            cmds.iter().map(|v| v.0).collect::<Vec<_>>().join(", ")
        )))
    }
}
