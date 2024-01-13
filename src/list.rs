use std::path::PathBuf;

use cote::*;

use crate::ss_display_help;

#[derive(Debug, Cote)]
#[cote(width = 50, overload)]
pub struct List {
    /// Display help message
    #[arg(alias = "-h")]
    help: bool,

    /// Set the path of local configurations
    #[arg(alias = "-p", value = "/home/root/configs", valid = valid!(|v: &PathBuf| v.exists()))]
    path: PathBuf,
}

pub fn process_list<'a>(args: &[&str]) -> color_eyre::Result<()> {
    let args = Args::from(args.iter().map(|v| *v));
    let CoteRes {
        mut policy,
        ret,
        mut parser,
    } = List::parse_args(args)?;

    if ret.status() {
        if ss_display_help!(parser, policy, ListInternalApp) {
            return Ok(());
        }
    }
    let list = List::try_extract(parser.optset_mut())?;

    

    Ok(())
}
