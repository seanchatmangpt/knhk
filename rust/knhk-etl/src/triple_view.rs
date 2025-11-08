// rust/knhk-etl/src/triple_view.rs
// Zero-copy triple access patterns
// Based on simdjson: use views instead of owned strings for zero-copy access

use crate::load::SoAArrays;

/// Zero-copy view into a single triple in SoAArrays
///
/// Pattern from simdjson: use views instead of copies for zero-copy access.
/// The view borrows from SoAArrays and provides zero-copy access to triple components.
///
/// # Lifetime
/// The view is tied to the lifetime of the SoAArrays it references.
/// Views become invalid if SoAArrays is dropped or reused.
///
/// # Performance Benefits
/// - Zero-copy access (no allocation, no copying)
/// - Cache-friendly (references existing data)
/// - Type-safe (prevents use-after-free)
pub struct TripleView<'a> {
    soa: &'a SoAArrays,
    index: usize,
}

impl<'a> TripleView<'a> {
    /// Create new triple view
    ///
    /// # Arguments
    /// * `soa` - Reference to SoAArrays
    /// * `index` - Index of triple (must be < 8)
    ///
    /// # Safety
    /// Caller must ensure index < 8 and SoAArrays outlives the view.
    #[inline(always)]
    pub fn new(soa: &'a SoAArrays, index: usize) -> Self {
        Self { soa, index }
    }

    /// Get subject (zero-copy)
    #[inline(always)]
    pub fn subject(&self) -> u64 {
        self.soa.s[self.index]
    }

    /// Get predicate (zero-copy)
    #[inline(always)]
    pub fn predicate(&self) -> u64 {
        self.soa.p[self.index]
    }

    /// Get object (zero-copy)
    #[inline(always)]
    pub fn object(&self) -> u64 {
        self.soa.o[self.index]
    }

    /// Get all components as tuple (zero-copy)
    #[inline(always)]
    pub fn as_tuple(&self) -> (u64, u64, u64) {
        (
            self.soa.s[self.index],
            self.soa.p[self.index],
            self.soa.o[self.index],
        )
    }

    /// Get index
    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }
}

/// Zero-copy iterator over triples in SoAArrays
///
/// Forward-only iterator (pattern from simdjson On-Demand API).
/// Iterates over triples in SoAArrays without copying data.
pub struct TripleIterator<'a> {
    soa: &'a SoAArrays,
    index: usize,
    len: usize,
}

impl<'a> TripleIterator<'a> {
    /// Create new iterator
    ///
    /// # Arguments
    /// * `soa` - Reference to SoAArrays
    /// * `len` - Number of triples to iterate (must be ≤ 8)
    #[inline(always)]
    pub fn new(soa: &'a SoAArrays, len: usize) -> Self {
        Self {
            soa,
            index: 0,
            len: len.min(8), // Safety: clamp to max capacity
        }
    }
}

impl<'a> Iterator for TripleIterator<'a> {
    type Item = TripleView<'a>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let view = TripleView::new(self.soa, self.index);
            self.index += 1;
            Some(view)
        } else {
            None
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for TripleIterator<'a> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len.saturating_sub(self.index)
    }
}

/// Extension trait for SoAArrays to provide zero-copy iteration
pub trait SoAArraysExt {
    /// Get zero-copy view of triple at index
    fn view_triple(&self, index: usize) -> TripleView<'_>;

    /// Iterate over triples (zero-copy, forward-only)
    fn iter_triples(&self, len: usize) -> TripleIterator<'_>;
}

impl SoAArraysExt for SoAArrays {
    #[inline(always)]
    fn view_triple(&self, index: usize) -> TripleView<'_> {
        TripleView::new(self, index)
    }

    #[inline(always)]
    fn iter_triples(&self, len: usize) -> TripleIterator<'_> {
        TripleIterator::new(self, len)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_triple_view() {
        let mut soa = SoAArrays::new();
        soa.s[0] = 1;
        soa.p[0] = 100;
        soa.o[0] = 1000;

        let view = TripleView::new(&soa, 0);
        assert_eq!(view.subject(), 1);
        assert_eq!(view.predicate(), 100);
        assert_eq!(view.object(), 1000);
        assert_eq!(view.as_tuple(), (1, 100, 1000));
    }

    #[test]
    fn test_triple_iterator() {
        let mut soa = SoAArrays::new();
        for i in 0..5 {
            soa.s[i] = (i + 1) as u64;
            soa.p[i] = 100;
            soa.o[i] = (i + 100) as u64;
        }

        let mut iter = TripleIterator::new(&soa, 5);
        assert_eq!(iter.len(), 5);

        let view1 = iter.next().unwrap();
        assert_eq!(view1.subject(), 1);
        assert_eq!(iter.len(), 4);

        let views: Vec<_> = iter.collect();
        assert_eq!(views.len(), 4);
        assert_eq!(views[0].subject(), 2);
    }

    #[test]
    fn test_soa_arrays_ext() {
        let mut soa = SoAArrays::new();
        soa.s[0] = 1;
        soa.p[0] = 100;
        soa.o[0] = 1000;

        let view = soa.view_triple(0);
        assert_eq!(view.subject(), 1);

        let mut iter = soa.iter_triples(1);
        let view = iter.next().unwrap();
        assert_eq!(view.subject(), 1);
    }
}
