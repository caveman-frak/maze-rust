use num::Unsigned;

#[allow(dead_code)]
pub fn diff<T: Unsigned + PartialOrd>(left: T, right: T) -> T {
    if left > right {
        left - right
    } else {
        right - left
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_diff_u8() {
        assert_eq!(diff(0u8, 0u8), 0);
        assert_eq!(diff(3u8, 3u8), 0);
        assert_eq!(diff(1u8, 3u8), 2);
        assert_eq!(diff(6u8, 3u8), 3);
    }

    #[test]
    fn check_diff_u16() {
        assert_eq!(diff(1u16, 3u16), 2);
        assert_eq!(diff(6u16, 3u16), 3);
    }

    #[test]
    fn check_diff_u32() {
        assert_eq!(diff(1u32, 3u32), 2);
        assert_eq!(diff(6u32, 3u32), 3);
    }

    #[test]
    fn check_diff_u64() {
        assert_eq!(diff(1u64, 3u64), 2);
        assert_eq!(diff(6u64, 3u64), 3);
    }

    #[test]
    fn check_diff_usize() {
        assert_eq!(diff(1usize, 3usize), 2);
        assert_eq!(diff(6usize, 3usize), 3);
    }
}
