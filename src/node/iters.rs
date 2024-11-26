use super::ReadNode;

pub struct KeyIter<N> {
    node: N,
    c: u8,
}

impl<N> KeyIter<N> {
    pub fn new(node: N) -> Self {
        Self { node, c: 0 }
    }
}

impl<N: ReadNode> Iterator for KeyIter<N> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.node.next_c(self.c)?;
        self.c = c + 1;
        Some(c)
    }
}

pub struct ChildIter<N> {
    node: N,
    c: u8,
}

impl<N> ChildIter<N> {
    pub fn new(node: N) -> Self {
        Self { node, c: 0 }
    }
}

impl<N: ReadNode> Iterator for ChildIter<N> {
    type Item = N::Idx;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.node.next_c(self.c)?;
        let idx = self.node.get(c);
        self.c = c + 1;
        Some(idx)
    }
}

pub struct PairIter<N> {
    node: N,
    c: u8,
}

impl<N> PairIter<N> {
    pub fn new(node: N) -> Self {
        Self { node, c: 0 }
    }
}

impl<N: ReadNode> Iterator for PairIter<N> {
    type Item = (u8, N::Idx);
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.node.next_c(self.c)?;
        let idx = self.node.get(c);
        self.c = c + 1;
        Some((c, idx))
    }
}
