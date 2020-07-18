use crate::style::Attribute;
use std::ops::{BitAnd, BitOr, BitXor};

/// a bitset for all possible attributes
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Attributes {
    bytes: u32,
    idx: u32,
}

impl Iterator for Attributes {
    type Item = Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        let len = 32 - self.bytes.leading_zeros();
        if self.idx >= len {
            return None;
        }
        let mut has_it = self.bytes & (1u32 << (self.idx + 1));
        while has_it == 0 {
            self.idx += 1;
            if self.idx >= len {
                return None;
            }
            has_it = self.bytes & (1u32 << (self.idx + 1));
        }

        let res = Attribute::iterator().nth(self.idx as usize).unwrap();
        self.idx += 1;
        Some(res)
    }
}

impl From<Attribute> for Attributes {
    fn from(attribute: Attribute) -> Self {
        Self {
            bytes: attribute.bytes(),
            idx: 0,
        }
    }
}

impl From<&[Attribute]> for Attributes {
    fn from(arr: &[Attribute]) -> Self {
        let mut attributes = Attributes::default();
        for &attr in arr {
            attributes.set(attr);
        }
        attributes
    }
}

impl BitAnd<Attribute> for Attributes {
    type Output = Self;
    fn bitand(self, rhs: Attribute) -> Self {
        Self {
            bytes: self.bytes & rhs.bytes(),
            idx: 0,
        }
    }
}
impl BitAnd for Attributes {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self {
            bytes: self.bytes & rhs.bytes,
            idx: 0,
        }
    }
}

impl BitOr<Attribute> for Attributes {
    type Output = Self;
    fn bitor(self, rhs: Attribute) -> Self {
        Self {
            bytes: self.bytes | rhs.bytes(),
            idx: 0,
        }
    }
}
impl BitOr for Attributes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self {
            bytes: self.bytes | rhs.bytes,
            idx: 0,
        }
    }
}

impl BitXor<Attribute> for Attributes {
    type Output = Self;
    fn bitxor(self, rhs: Attribute) -> Self {
        Self {
            bytes: self.bytes ^ rhs.bytes(),
            idx: 0,
        }
    }
}
impl BitXor for Attributes {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self {
            bytes: self.bytes ^ rhs.bytes,
            idx: 0,
        }
    }
}

impl Attributes {
    /// Sets the attribute.
    /// If it's already set, this does nothing.
    #[inline(always)]
    pub fn set(&mut self, attribute: Attribute) {
        self.bytes |= attribute.bytes();
    }

    /// Unsets the attribute.
    /// If it's not set, this changes nothing.
    #[inline(always)]
    pub fn unset(&mut self, attribute: Attribute) {
        self.bytes &= !attribute.bytes();
    }

    /// Sets the attribute if it's unset, unset it
    /// if it is set.
    #[inline(always)]
    pub fn toggle(&mut self, attribute: Attribute) {
        self.bytes ^= attribute.bytes();
    }

    /// Returns whether the attribute is set.
    #[inline(always)]
    pub fn has(self, attribute: Attribute) -> bool {
        self.bytes & attribute.bytes() != 0
    }

    /// Sets all the passed attributes. Removes none.
    #[inline(always)]
    pub fn extend(&mut self, attributes: Attributes) {
        self.bytes |= attributes.bytes;
    }

    /// Returns whether there is no attribute set.
    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.bytes == 0
    }
}

#[cfg(test)]
mod tests {
    use super::{Attribute, Attributes};

    #[test]
    fn test_attributes() {
        let mut attributes: Attributes = Attribute::Bold.into();
        assert!(attributes.has(Attribute::Bold));
        attributes.set(Attribute::Italic);
        assert!(attributes.has(Attribute::Italic));
        attributes.unset(Attribute::Italic);
        assert!(!attributes.has(Attribute::Italic));
        attributes.toggle(Attribute::Bold);
        assert!(attributes.is_empty());
    }
}
