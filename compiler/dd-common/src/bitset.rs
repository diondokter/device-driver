#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BitSet {
    words: Vec<usize>,
    len: usize,
}

impl BitSet {
    pub const fn new() -> Self {
        Self {
            words: Vec::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, value: bool) {
        let (next_word, next_bit) = Self::index_to_word_index(self.len);

        let word = if next_word == self.words.len() {
            self.words.push_mut(0)
        } else {
            &mut self.words[next_word]
        };

        *word |= (value as usize) << next_bit;

        self.len += 1;
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.len {
            return None;
        }

        let (word, bit) = Self::index_to_word_index(index);

        Some((self.words[word] & (1 << bit)) > 0)
    }

    #[track_caller]
    pub fn set(&mut self, index: usize, value: bool) {
        if index >= self.len {
            panic!(
                "index out of range, tried to get element {index} from a set with len {}",
                self.len
            );
        }

        let (word, bit) = Self::index_to_word_index(index);

        if value {
            self.words[word] |= 1 << bit;
        } else {
            self.words[word] &= !(1 << bit);
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn index_to_word_index(index: usize) -> (usize, usize) {
        let word = index / usize::BITS as usize;
        let bits = index % usize::BITS as usize;
        (word, bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitset() {
        let mut set = BitSet::new();
        set.push(true);
        set.push(false);
        set.push(true);
        set.push(false);

        assert_eq!(set.len(), 4);

        assert_eq!(set.get(0), Some(true));
        assert_eq!(set.get(1), Some(false));
        assert_eq!(set.get(2), Some(true));
        assert_eq!(set.get(3), Some(false));
        assert_eq!(set.get(4), None);
    }

    #[test]
    fn bitset_long() {
        const SIZE: usize = 1000;

        let mut set = BitSet::new();

        for i in 0..SIZE {
            set.push(i.is_power_of_two());
        }

        for i in 0..SIZE {
            assert_eq!(set.get(i), Some(i.is_power_of_two()));
        }

        for i in 0..SIZE {
            set.set(i, !i.is_power_of_two());
        }

        for i in 0..SIZE {
            assert_eq!(set.get(i), Some(!i.is_power_of_two()));
        }

        assert_eq!(set.get(SIZE), None);
    }

    #[test]
    #[should_panic = "index out of range, tried to get element 1 from a set with len 1"]
    fn bitset_set_panics() {
        let mut set = BitSet::new();
        set.push(true);

        set.set(1, false);
    }
}
