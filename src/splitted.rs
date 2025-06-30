enum State {
    Space,
    Normal,
}

enum Spec {
    Escape,
    SingleQuote,
    DoubleQuote,
}

#[derive(Debug)]
pub struct Slice<'a> {
    pub beg: usize,
    pub dat: &'a str,
}

pub struct Splitted<'a> {
    slices: Vec<Slice<'a>>,
}

pub struct Args<'a> {
    pub cword: Option<usize>,

    pub args: Vec<&'a str>,
}

impl<'a> Splitted<'a> {
    pub fn new(line: &'a str) -> Self {
        Self {
            slices: Self::inner_split(line),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Slice<'a>> {
        self.slices.iter()
    }

    pub fn len(&self) -> usize {
        self.slices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn split_args(&self, pos: Option<usize>) -> Args<'a> {
        let mut args = vec![];
        let mut cword = None;

        for (index, slice) in self.iter().enumerate() {
            let dat = slice.dat.trim();

            if Some(index) == pos {
                cword = Some(args.len());
                args.push(dat);
            } else if !dat.is_empty() {
                args.push(dat);
            }
        }

        Args { cword, args }
    }

    fn inner_split(line: &'a str) -> Vec<Slice<'a>> {
        let mut beg = 0;
        let mut state = State::Space;
        let mut spec = vec![];
        let mut words = vec![];

        for (pos, char) in line
            .char_indices()
            .map(|(p, c)| (p, Some(c)))
            .chain(std::iter::once((line.len(), None)))
        {
            if let Some(char) = char {
                if let Some(top) = spec.last() {
                    if matches!(top, Spec::Escape) {
                        spec.pop();
                        continue;
                    }
                }
                match state {
                    State::Space if !char.is_whitespace() => {
                        if pos - beg > 0 {
                            words.push(Slice {
                                beg,
                                dat: &line[beg..pos],
                            });
                        }
                        state = State::Normal;
                        beg = pos;
                    }
                    State::Normal if char.is_whitespace() => {
                        if spec.last().is_none() {
                            if pos - beg > 0 {
                                words.push(Slice {
                                    beg,
                                    dat: &line[beg..pos],
                                });
                            }
                            state = State::Space;
                            beg = pos;
                        }
                    }
                    State::Normal => {}
                    _ => {}
                }
                // check if we have new special char
                match char {
                    '\'' => {
                        if let Some(pos) = spec.iter().position(|v| matches!(v, Spec::SingleQuote))
                        {
                            for _ in 0..spec.len() - pos {
                                spec.pop();
                            }
                        } else {
                            spec.push(Spec::SingleQuote);
                        }
                    }
                    '"' => {
                        if let Some(pos) = spec.iter().position(|v| matches!(v, Spec::DoubleQuote))
                        {
                            for _ in 0..spec.len() - pos {
                                spec.pop();
                            }
                        } else {
                            spec.push(Spec::DoubleQuote);
                        }
                    }
                    '\\' => {
                        spec.push(Spec::Escape);
                    }
                    _ => {}
                }
            } else if pos - beg >= 1 {
                words.push(Slice {
                    beg,
                    dat: &line[beg..],
                });
            }
        }

        words
    }
}

impl<'a> IntoIterator for Splitted<'a> {
    type Item = Slice<'a>;

    type IntoIter = std::vec::IntoIter<Slice<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.slices.into_iter()
    }
}
