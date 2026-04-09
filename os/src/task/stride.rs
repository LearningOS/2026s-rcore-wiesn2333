pub struct Stride(pub usize);

pub const BIG_STRIDE: usize = usize::MAX >> 10;

impl Ord for Stride {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let this = self.0;
        let other = other.0;
        {
            let min = this.min(other);
            let max = this.max(other);
            if max - min > BIG_STRIDE << 1 {
                if this > other {
                    return core::cmp::Ordering::Less;
                } else {
                    return core::cmp::Ordering::Greater;
                }
            }
            if this > other {
                core::cmp::Ordering::Greater
            } else {
                core::cmp::Ordering::Less
            }
        }
    }
}

impl Eq for Stride {}

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Stride {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
