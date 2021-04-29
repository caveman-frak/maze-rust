use rand::{Error, RngCore};
use rand_core::impls;
use std::iter::IntoIterator;

pub struct SeriesRng<T, I> {
    iterator: I,
    iterable: T,
}

#[allow(dead_code)]
impl<T, I> SeriesRng<T, I>
where
    I: Iterator<Item = u64>,
    T: IntoIterator<Item = u64, IntoIter = I> + Clone,
{
    pub fn new(iterable: T) -> Self {
        SeriesRng {
            iterator: iterable.clone().into_iter(),
            iterable,
        }
    }
}

impl<T, I> RngCore for SeriesRng<T, I>
where
    I: Iterator<Item = u64>,
    T: IntoIterator<Item = u64, IntoIter = I> + Clone,
{
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        match self.iterator.next() {
            Some(result) => result,
            None => {
                self.iterator = self.iterable.clone().into_iter();
                self.next_u64()
            }
        }
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SeriesRng;
    use rand::RngCore;
    use std::array::IntoIter;

    #[test]
    fn test_series_with_vec() {
        let mut some_rng = SeriesRng::new(vec![0, 1]);

        assert_eq!(some_rng.next_u32(), 0);
        assert_eq!(some_rng.next_u32(), 1);
        assert_eq!(some_rng.next_u32(), 0);
        assert_eq!(some_rng.next_u32(), 1);
    }

    #[test]
    fn test_series_with_range() {
        let mut some_rng = SeriesRng::new(0..2);

        assert_eq!(some_rng.next_u32(), 0);
        assert_eq!(some_rng.next_u32(), 1);
        assert_eq!(some_rng.next_u32(), 0);
        assert_eq!(some_rng.next_u32(), 1);
    }

    #[test]
    fn test_series_with_inclusive_range() {
        let mut some_rng = SeriesRng::new(0..=2);

        assert_eq!(some_rng.next_u32(), 0);
        assert_eq!(some_rng.next_u32(), 1);
        assert_eq!(some_rng.next_u32(), 2);
        assert_eq!(some_rng.next_u32(), 0);
    }

    #[test]
    fn test_series_with_endless_range() {
        let mut some_rng = SeriesRng::new(0..);

        assert_eq!(some_rng.next_u32(), 0);
        assert_eq!(some_rng.next_u32(), 1);
        assert_eq!(some_rng.next_u32(), 2);
        assert_eq!(some_rng.next_u32(), 3);
    }

    #[test]
    fn test_series_with_array() {
        let mut some_rng = SeriesRng::new(IntoIter::new([0, 1, 2]));

        assert_eq!(some_rng.next_u32(), 0);
        assert_eq!(some_rng.next_u32(), 1);
        assert_eq!(some_rng.next_u32(), 2);
        assert_eq!(some_rng.next_u32(), 0);
    }
}
