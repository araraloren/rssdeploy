use cote::*;

#[derive(Debug, Cote)]
#[cote(width = 50, overload)]
pub struct Kill {
    /// Kill the given shadowsocks instance
    #[pos()]
    pub index: usize,
}
