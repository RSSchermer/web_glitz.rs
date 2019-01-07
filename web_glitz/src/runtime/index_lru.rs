pub(crate) struct IndexLRU {
    linkage: Vec<(usize, usize)>,
    lru_index: usize,
    mru_index: usize,
}

impl IndexLRU {
    pub(crate) fn new(max_index: usize) -> Self {
        let mut linkage = Vec::with_capacity(max_index);
        let texture_units = max_index as i32;

        for i in 0..texture_units {
            linkage.push((
                ((i - 1) % texture_units) as usize,
                ((i + 1) % texture_units) as usize,
            ));
        }

        IndexLRU {
            linkage,
            lru_index: 0,
            mru_index: (texture_units - 1) as usize,
        }
    }

    pub(crate) fn use_index(&mut self, index: usize) {
        if index != self.mru_index {
            if index == self.lru_index {
                self.use_lru_index();
            } else {
                let (previous, next) = self.linkage[index];

                self.linkage[previous].1 = next;
                self.linkage[next].0 = previous;
                self.linkage[self.lru_index].0 = index;
                self.linkage[self.mru_index].1 = index;
                self.linkage[index].0 = self.mru_index;
                self.linkage[index].1 = self.lru_index;
                self.mru_index = index;
            }
        }
    }

    pub(crate) fn use_lru_index(&mut self) -> usize {
        let old_lru_index = self.lru_index;

        self.lru_index = self.linkage[old_lru_index].1;
        self.mru_index = old_lru_index;

        old_lru_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_lru() {
        let mut lru = IndexLRU::new(4);

        assert_eq!(lru.use_lru_index(), 0);
        assert_eq!(lru.use_lru_index(), 1);
        assert_eq!(lru.use_lru_index(), 2);
        assert_eq!(lru.use_lru_index(), 3);
        assert_eq!(lru.use_lru_index(), 0);

        lru.use_index(0);

        assert_eq!(lru.use_lru_index(), 1);

        lru.use_index(3);

        assert_eq!(lru.use_lru_index(), 2);
        assert_eq!(lru.use_lru_index(), 0);
        assert_eq!(lru.use_lru_index(), 1);
        assert_eq!(lru.use_lru_index(), 3);
        assert_eq!(lru.use_lru_index(), 2);
        assert_eq!(lru.use_lru_index(), 0);
        assert_eq!(lru.use_lru_index(), 1);
    }
}
