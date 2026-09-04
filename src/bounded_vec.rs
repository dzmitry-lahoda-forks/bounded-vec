use crate::witnesses;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::slice::{Iter, IterMut};
use thiserror::Error;

/// Non-empty Vec bounded with minimal (L - lower bound) and maximal (U - upper bound) items quantity.
///
/// # Type Parameters
///
/// * `W` - witness type to prove vector ranges and shape of interface accordingly
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct BoundedVec<
    T,
    const L: usize,
    const U: usize,
    W = witnesses::NonEmpty<L, U>,
    #[cfg(feature = "nightly")] A: alloc::alloc::Allocator = alloc::alloc::Global,
> {
    #[cfg(feature = "nightly")]
    inner: Vec<T, A>,
    #[cfg(not(feature = "nightly"))]
    inner: Vec<T>,
    witness: W,
}

/// BoundedVec errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum BoundedVecOutOfBounds {
    /// Items quantity is less than L (lower bound)
    #[error("Lower bound violation: smaller by {got_smaller_by} than {lower_bound}")]
    LowerBoundError {
        /// L (lower bound)
        lower_bound: usize,
        /// Number of items below the lower bound
        got_smaller_by: usize,
    },
    /// Items quantity is more than U (upper bound)
    #[error("Upper bound violation: larger by {got_larger_by} than {upper_bound}")]
    UpperBoundError {
        /// U (upper bound)
        upper_bound: usize,
        /// Number of items above the upper bound
        got_larger_by: usize,
    },
}

fn normalized_range<R: core::ops::RangeBounds<usize>>(
    range: &R,
    len: usize,
) -> core::ops::Range<usize> {
    let start = match range.start_bound() {
        core::ops::Bound::Included(start) => *start,
        core::ops::Bound::Excluded(start) => match start.checked_add(1) {
            Some(start) => start,
            None => panic!("range start index overflow"),
        },
        core::ops::Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        core::ops::Bound::Included(end) => match end.checked_add(1) {
            Some(end) => end,
            None => panic!("range end index overflow"),
        },
        core::ops::Bound::Excluded(end) => *end,
        core::ops::Bound::Unbounded => len,
    };
    assert!(
        start <= end,
        "slice index starts at {start} but ends at {end}"
    );
    assert!(
        end <= len,
        "range end index {end} out of range for slice of length {len}"
    );
    start..end
}

fn larger_by_after_adding(len: usize, additional: usize, upper_bound: usize) -> usize {
    match len.checked_add(additional) {
        Some(new_len) => new_len.saturating_sub(upper_bound),
        None if len <= upper_bound => additional.saturating_sub(upper_bound - len),
        None => usize::MAX,
    }
}

fn smaller_by_after_removing(len: usize, removed: usize, lower_bound: usize) -> usize {
    match len.checked_sub(removed) {
        Some(new_len) => lower_bound.saturating_sub(new_len),
        None => lower_bound,
    }
}

impl<T, const U: usize> Default for BoundedVec<T, 0, U, witnesses::Empty<U>> {
    fn default() -> Self {
        BoundedVec {
            inner: Vec::new(),
            witness: witnesses::empty(),
        }
    }
}

impl<T, const U: usize> BoundedVec<T, 0, U, witnesses::Empty<U>> {
    /// Creates new BoundedVec or returns error if items count is out of bounds
    ///
    /// # Parameters
    ///
    /// * `items` - vector of items within bounds
    ///
    /// # Errors
    ///
    /// * `UpperBoundError` - if `items`` len is more than U (upper bound)
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// let data: BoundedVec<_, 0, 8, witnesses::Empty<8>> =
    ///     BoundedVec::<_, 0, 8, witnesses::Empty<8>>::from_vec(vec![1u8, 2]).unwrap();
    /// ```
    pub fn from_vec(items: Vec<T>) -> Result<Self, BoundedVecOutOfBounds> {
        let witness = witnesses::empty::<U>();
        let len = items.len();
        if len > U {
            Err(BoundedVecOutOfBounds::UpperBoundError {
                upper_bound: U,
                got_larger_by: len - U,
            })
        } else {
            Ok(BoundedVec {
                inner: items,
                witness,
            })
        }
    }

    /// Returns the first element of the vector, or `None` if it is empty
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<u8, 0, 8, witnesses::Empty<8>> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.first(), Some(&1u8));
    /// ```
    pub fn first(&self) -> Option<&T> {
        self.inner.first()
    }

    /// Constructs a new, empty `BoundedVec`.
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::EmptyBoundedVec;
    /// let v: EmptyBoundedVec<i32, 10> = EmptyBoundedVec::new();
    /// assert!(v.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            witness: witnesses::empty(),
        }
    }

    /// Constructs a new, empty `BoundedVec` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            witness: witnesses::empty(),
        }
    }

    /// Returns the last element of the vector, or `None` if it is empty
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<u8, 0, 8, witnesses::Empty<8>> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.last(), Some(&2u8));
    /// ```
    pub fn last(&self) -> Option<&T> {
        self.inner.last()
    }

    /// Returns a mutable reference to the first element of the vector, or `None` if it is empty
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.inner.first_mut()
    }

    /// Returns a mutable reference to the last element of the vector, or `None` if it is empty
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.inner.last_mut()
    }

    /// Removes the last element from the vector and returns it, or `None` if it is empty.
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::EmptyBoundedVec;
    /// let mut v: EmptyBoundedVec<i32, 4> = vec![1, 2].try_into().unwrap();
    /// assert_eq!(v.pop(), Some(2));
    /// assert_eq!(v.pop(), Some(1));
    /// assert_eq!(v.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> T {
        self.inner.remove(index)
    }

    /// Removes the subslice indicated by the given range and returns a
    /// double-ended iterator over the removed elements.
    ///
    /// If the iterator is dropped before being fully consumed, it drops the
    /// remaining removed elements. The returned iterator keeps a mutable borrow
    /// on the vector to optimize its implementation.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if the
    /// end point is greater than the vector length.
    ///
    /// # Leaking
    ///
    /// If the iterator is not dropped (for example, through [`core::mem::forget`]),
    /// the vector may have lost and leaked elements arbitrarily, including
    /// elements outside the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_bounded_collections::EmptyBoundedVec;
    /// let mut v: EmptyBoundedVec<i32, 4> = vec![1, 2, 3].try_into().unwrap();
    /// let u: Vec<_> = v.drain(1..).collect();
    /// assert_eq!(v.as_slice(), &[1]);
    /// assert_eq!(u, &[2, 3]);
    ///
    /// // A full range clears the vector, like `clear()` does.
    /// v.drain(..);
    /// assert!(v.is_empty());
    /// ```
    pub fn drain<R>(&mut self, range: R) -> alloc::vec::Drain<'_, T>
    where
        R: core::ops::RangeBounds<usize>,
    {
        self.inner.drain(range)
    }

    /// Clears the vector, removing all values.
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::EmptyBoundedVec;
    /// let mut v: EmptyBoundedVec<i32, 4> = vec![1, 2].try_into().unwrap();
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f);
    }

    /// Shortens the vector, keeping the first `len` elements and dropping the rest.
    pub fn truncate(&mut self, len: usize) {
        self.inner.truncate(len);
    }

    /// Removes consecutive duplicate elements in the vector.
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.inner.dedup();
    }

    /// Removes consecutive elements that satisfy the given predicate.
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.inner.dedup_by(same_bucket);
    }

    /// Removes consecutive duplicate elements according to a key.
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.inner.dedup_by_key(key);
    }
}

/// Methods which works for all witnesses
impl<T, const L: usize, const U: usize, W> BoundedVec<T, L, U, W> {
    /// Returns the number of elements in the vector.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 4> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::EmptyBoundedVec;
    /// let data: EmptyBoundedVec<u8, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert!(!data.is_empty());
    /// assert!(EmptyBoundedVec::<u8, 8>::new().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Appends an item to the vector.
    ///
    /// # Panics
    ///
    /// Panics if will be greater than `U`.
    #[cfg(feature = "panic")]
    pub fn push(&mut self, item: T) {
        let len = self.inner.len();
        // NOTE: need to thing if split unbounded `usize::MAX`(panic) and bounded(return error)
        if len >= U {
            panic!(
                "Cannot push item to BoundedVec: length {} is already at upper bound {}",
                len, U
            );
        }
        self.inner.push(item);
    }

    /// Returns a reference to underlying `Vec``
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.as_vec(), &vec![1u8,2]);
    /// ```
    pub fn as_vec(&self) -> &Vec<T> {
        &self.inner
    }

    /// Returns an underlying `Vec``
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.to_vec(), vec![1u8,2]);
    /// ```
    pub fn to_vec(self) -> Vec<T> {
        self.inner
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(data.as_slice(), &[1u8,2]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    /// Returns a reference for an element at index or `None` if out of bounds
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let elem = *data.get(1).unwrap();
    /// assert_eq!(elem, 2);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    /// Returns a mutable reference for an element at index or `None` if out of bounds
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let mut data: BoundedVec<u8, 2, 8> = [1u8,2].into();
    /// let elem = *data.get_mut(1).unwrap();
    /// assert_eq!(elem, 2);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }

    /// Returns an iterator
    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }

    /// Returns an iterator that allows to modify each value
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.inner.iter_mut()
    }

    /// Appends an element to the back of the vector, returning an error if
    /// the vector length is already at upper bound `U`.
    #[cfg_attr(all(test, not(debug_assertions)), no_panic::no_panic)]
    pub fn try_push(&mut self, item: T) -> Result<(), (T, BoundedVecOutOfBounds)> {
        let len = self.inner.len();
        if len >= U {
            return Err((
                item,
                BoundedVecOutOfBounds::UpperBoundError {
                    upper_bound: U,
                    got_larger_by: larger_by_after_adding(len, 1, U),
                },
            ));
        }
        self.inner.push(item);
        Ok(())
    }

    /// Extends the vector, returning an error without changing it if the final
    /// length would exceed upper bound `U`.
    ///
    /// Buffers at most the remaining capacity and stops at the first excess item.
    /// `got_larger_by` is a lower bound on the excess, using the iterator's size
    /// hint when available; it is exact for exact-size iterators.
    #[cfg_attr(all(test, not(debug_assertions)), no_panic::no_panic)]
    pub fn try_extend<I>(&mut self, iter: I) -> Result<(), BoundedVecOutOfBounds>
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        let len = self.inner.len();
        let additional = iter.size_hint().0;
        let got_larger_by = larger_by_after_adding(len, additional, U);
        if got_larger_by != 0 {
            return Err(BoundedVecOutOfBounds::UpperBoundError {
                upper_bound: U,
                got_larger_by,
            });
        }
        let remaining = U.saturating_sub(len);
        let mut items: Vec<T> = iter.by_ref().take(remaining).collect();
        if items.len() == remaining && iter.next().is_some() {
            return Err(BoundedVecOutOfBounds::UpperBoundError {
                upper_bound: U,
                got_larger_by: 1,
            });
        }
        self.inner.append(&mut items);
        Ok(())
    }

    /// Inserts an element at position `index` within the vector, shifting all
    /// elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    /// Panics if the vector length is already at upper bound `U`.
    #[cfg(feature = "panic")]
    pub fn insert(&mut self, index: usize, element: T) {
        let len = self.inner.len();
        if len >= U {
            panic!(
                "Cannot insert item into BoundedVec: length {} is already at upper bound {}",
                len, U
            );
        }
        self.inner.insert(index, element);
    }

    /// Inserts an element at position `index` within the vector, returning an error
    /// if the vector length is already at upper bound `U`.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[cfg_attr(all(test, not(debug_assertions)), no_panic::no_panic)]
    pub fn try_insert(
        &mut self,
        index: usize,
        element: T,
    ) -> Result<(), (T, BoundedVecOutOfBounds)> {
        let len = self.inner.len();
        if len >= U {
            return Err((
                element,
                BoundedVecOutOfBounds::UpperBoundError {
                    upper_bound: U,
                    got_larger_by: larger_by_after_adding(len, 1, U),
                },
            ));
        }
        self.inner.insert(index, element);
        Ok(())
    }

    /// Extracts a mutable slice containing the entire vector.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.inner.as_mut_slice()
    }

    /// Returns the total number of elements the vector can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted in the given `BoundedVec`.
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Reserves the minimum capacity for at least `additional` more elements to be inserted in the given `BoundedVec`.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    /// Shrinks the capacity of the vector as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    /// Creates a new `BoundedVec` by consuming `self` and mapping each element.
    ///
    /// The result preserves the length, bounds `L` and `U`, and witness `W`,
    /// even though the original vector is consumed and turned into an iterator.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    /// let data = data.mapped(|x| x * 2);
    /// assert_eq!(data, [2u8, 4].into());
    /// ```
    pub fn mapped<F, N>(self, map_fn: F) -> BoundedVec<N, L, U, W>
    where
        F: FnMut(T) -> N,
    {
        BoundedVec {
            inner: self.inner.into_iter().map(map_fn).collect(),
            witness: self.witness,
        }
    }

    /// Creates a new `BoundedVec` by mapping references to every element.
    ///
    /// The result preserves the length and bounds `L` and `U`, and clones the
    /// witness `W`. The original vector remains available.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    /// let mapped = data.mapped_ref(|x| x * 2);
    /// assert_eq!(mapped, [2u8, 4].into());
    /// assert_eq!(data.as_slice(), &[1, 2]);
    /// ```
    pub fn mapped_ref<F, N>(&self, map_fn: F) -> BoundedVec<N, L, U, W>
    where
        F: FnMut(&T) -> N,
        W: Clone,
    {
        BoundedVec {
            inner: self.inner.iter().map(map_fn).collect(),
            witness: self.witness.clone(),
        }
    }

    /// Creates a new `BoundedVec` by consuming `self` and fallibly mapping each element.
    ///
    /// On success, the result preserves the length, bounds `L` and `U`, and
    /// witness `W`. This behaves like chaining `into_iter()`, `map`, and
    /// `collect::<Result<Vec<N>, E>>()`, then wrapping the result in a `BoundedVec`.
    ///
    /// Since this method consumes `self`, an error drops the remaining input
    /// elements and any output elements already produced.
    ///
    /// # Errors
    ///
    /// Returns the first error from `map_fn` immediately, without mapping any
    /// remaining elements.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    /// let data: Result<BoundedVec<u8, 2, 8>, _> = data.try_mapped(|_| Err("failed"));
    /// assert_eq!(data, Err("failed"));
    /// ```
    pub fn try_mapped<F, N, E>(self, map_fn: F) -> Result<BoundedVec<N, L, U, W>, E>
    where
        F: FnMut(T) -> Result<N, E>,
    {
        let witness = self.witness;
        let inner = self
            .inner
            .into_iter()
            .map(map_fn)
            .collect::<Result<_, _>>()?;
        Ok(BoundedVec { inner, witness })
    }

    /// Creates a new `BoundedVec` by fallibly mapping references to every element.
    ///
    /// On success, the result preserves the length and bounds `L` and `U`, and
    /// clones the witness `W`. The original vector is borrowed and remains
    /// available even if mapping fails.
    ///
    /// # Errors
    ///
    /// Returns the first error from `map_fn` immediately, without mapping any
    /// remaining elements. Any output elements already produced are dropped.
    ///
    /// # Example
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let data: BoundedVec<u8, 2, 8> = [1u8, 2].into();
    /// let mapped: Result<BoundedVec<u8, 2, 8>, _> = data.try_mapped_ref(|_| Err("failed"));
    /// assert_eq!(mapped, Err("failed"));
    /// assert_eq!(data.as_slice(), &[1, 2]);
    /// ```
    pub fn try_mapped_ref<F, N, E>(&self, map_fn: F) -> Result<BoundedVec<N, L, U, W>, E>
    where
        F: FnMut(&T) -> Result<N, E>,
        W: Clone,
    {
        let inner = self.inner.iter().map(map_fn).collect::<Result<_, _>>()?;
        Ok(BoundedVec {
            inner,
            witness: self.witness.clone(),
        })
    }

    /// Returns a new `BoundedVec` with indices included.
    pub fn enumerated(self) -> BoundedVec<(usize, T), L, U, W> {
        BoundedVec {
            inner: self.inner.into_iter().enumerate().collect(),
            witness: self.witness,
        }
    }
}

impl<T, const L: usize, const U: usize> BoundedVec<T, L, U, witnesses::NonEmpty<L, U>> {
    /// Creates new BoundedVec or returns error if items count is out of bounds
    ///
    /// # Parameters
    ///
    /// * `items` - vector of items within bounds
    ///
    /// # Errors
    ///
    /// * `LowerBoundError` - if `items`` len is less than L (lower bound)
    /// * `UpperBoundError` - if `items`` len is more than U (upper bound)
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::witnesses;
    /// let data: BoundedVec<_, 2, 8, witnesses::NonEmpty<2, 8>> =
    ///     BoundedVec::<_, 2, 8, witnesses::NonEmpty<2, 8>>::from_vec(vec![1u8, 2]).unwrap();
    /// ```
    pub fn from_vec(items: Vec<T>) -> Result<Self, BoundedVecOutOfBounds> {
        let witness = witnesses::non_empty::<L, U>();
        let len = items.len();
        if len < L {
            Err(BoundedVecOutOfBounds::LowerBoundError {
                lower_bound: L,
                got_smaller_by: L - len,
            })
        } else if len > U {
            Err(BoundedVecOutOfBounds::UpperBoundError {
                upper_bound: U,
                got_larger_by: len - U,
            })
        } else {
            Ok(BoundedVec {
                inner: items,
                witness,
            })
        }
    }

    /// Returns the first element of non-empty Vec
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(*data.first(), 1);
    /// ```
    pub fn first(&self) -> &T {
        #[allow(clippy::unwrap_used)]
        self.inner.first().unwrap()
    }

    /// Returns the last element of non-empty Vec
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use std::convert::TryInto;
    ///
    /// let data: BoundedVec<_, 2, 8> = vec![1u8, 2].try_into().unwrap();
    /// assert_eq!(*data.last(), 2);
    /// ```
    pub fn last(&self) -> &T {
        #[allow(clippy::unwrap_used)]
        self.inner.last().unwrap()
    }

    /// Returns the last and all the rest of the elements
    pub fn split_last(&self) -> (&T, &[T]) {
        #[allow(clippy::unwrap_used)]
        self.inner.split_last().unwrap()
    }

    /// Return a Some(BoundedVec) or None if `v` is empty
    /// # Example
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// use const_bounded_collections::OptBoundedVecToVec;
    ///
    /// let opt_bv_none = BoundedVec::<u8, 2, 8>::opt_empty_vec(vec![]).unwrap();
    /// assert!(opt_bv_none.is_none());
    /// assert_eq!(opt_bv_none.to_vec(), Vec::<u8>::new());
    /// let opt_bv_some = BoundedVec::<u8, 2, 8>::opt_empty_vec(vec![0u8, 2]).unwrap();
    /// assert!(opt_bv_some.is_some());
    /// assert_eq!(opt_bv_some.to_vec(), vec![0u8, 2]);
    /// ```
    pub fn opt_empty_vec(
        v: Vec<T>,
    ) -> Result<Option<BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>>, BoundedVecOutOfBounds> {
        if v.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self::from_vec(v)?))
        }
    }

    /// Returns a mutable reference to the first element of non-empty Vec
    pub fn first_mut(&mut self) -> &mut T {
        #[allow(clippy::unwrap_used)]
        self.inner.first_mut().unwrap()
    }

    /// Returns a mutable reference to the last element of non-empty Vec
    pub fn last_mut(&mut self) -> &mut T {
        #[allow(clippy::unwrap_used)]
        self.inner.last_mut().unwrap()
    }

    /// Returns the first and all the rest of the elements.
    pub fn split_first(&self) -> (&T, &[T]) {
        #[allow(clippy::unwrap_used)]
        self.inner.split_first().unwrap()
    }

    /// Returns a mutable reference to the first element and a mutable slice of all the rest.
    pub fn split_first_mut(&mut self) -> (&mut T, &mut [T]) {
        #[allow(clippy::unwrap_used)]
        self.inner.split_first_mut().unwrap()
    }

    /// Returns a mutable reference to the last element and a mutable slice of all the rest.
    pub fn split_last_mut(&mut self) -> (&mut T, &mut [T]) {
        #[allow(clippy::unwrap_used)]
        self.inner.split_last_mut().unwrap()
    }

    /// Removes elements in `range`, but never more than `len - L`, and returns
    /// an owning, double-ended iterator over the removed elements.
    ///
    /// If the requested range is larger than the removable amount, its end is
    /// shortened so that exactly `L` elements remain. Removal is completed
    /// before this method returns, so forgetting the iterator cannot violate
    /// the lower bound.
    ///
    /// The returned iterator does not borrow the vector. Dropping it before it
    /// is fully consumed drops the remaining removed elements.
    ///
    /// # Leaking
    ///
    /// Forgetting the iterator with [`core::mem::forget`] leaks its remaining
    /// removed elements, but does not remove or leak any further vector elements.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if the
    /// end point is greater than the vector length.
    ///
    /// # Examples
    ///
    /// ```
    /// use const_bounded_collections::BoundedVec;
    /// let mut v: BoundedVec<i32, 2, 5> = vec![1, 2, 3, 4, 5].try_into().unwrap();
    /// let removed: Vec<_> = v.drain(1..).collect();
    /// assert_eq!(removed, &[2, 3, 4]);
    /// assert_eq!(v.as_slice(), &[1, 5]);
    ///
    /// // A full range preserves at least the lower bound of two elements.
    /// v.drain(..);
    /// assert_eq!(v.as_slice(), &[1, 5]);
    /// ```
    pub fn drain<R>(&mut self, range: R) -> vec::IntoIter<T>
    where
        R: core::ops::RangeBounds<usize>,
    {
        let range = normalized_range(&range, self.inner.len());
        let removable = self.inner.len().saturating_sub(L);
        let drain_len = (range.end - range.start).min(removable);
        let end = range.start + drain_len;
        self.inner
            .drain(range.start..end)
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Removes the last element from the vector and returns it, or returns
    /// `LowerBoundError` if the vector length is already at lower bound `L`.
    pub fn try_pop(&mut self) -> Result<T, BoundedVecOutOfBounds> {
        let len = self.inner.len();
        if len <= L {
            return Err(BoundedVecOutOfBounds::LowerBoundError {
                lower_bound: L,
                got_smaller_by: smaller_by_after_removing(len, 1, L),
            });
        }
        #[allow(clippy::unwrap_used)]
        Ok(self.inner.pop().unwrap())
    }

    /// Removes the element at position `index`, returning it, or returns
    /// `LowerBoundError` if removing an element would violate lower bound `L`.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    pub fn try_remove(&mut self, index: usize) -> Result<T, BoundedVecOutOfBounds> {
        let len = self.inner.len();
        if len <= L {
            return Err(BoundedVecOutOfBounds::LowerBoundError {
                lower_bound: L,
                got_smaller_by: smaller_by_after_removing(len, 1, L),
            });
        }
        if index >= len {
            panic!("removal index (is {index}) should be < len (is {len})");
        }
        Ok(self.inner.remove(index))
    }

    /// Shortens the vector to `len`, or returns `LowerBoundError` if `len < L`.
    pub fn try_truncate(&mut self, len: usize) -> Result<(), BoundedVecOutOfBounds> {
        if len < L {
            return Err(BoundedVecOutOfBounds::LowerBoundError {
                lower_bound: L,
                got_smaller_by: L - len,
            });
        }
        self.inner.truncate(len);
        Ok(())
    }

    /// Removes consecutive duplicate elements in the vector, or returns
    /// `LowerBoundError` if deduplication causes the length to drop below `L`.
    pub fn try_dedup(&mut self) -> Result<(), BoundedVecOutOfBounds>
    where
        T: PartialEq + Clone,
    {
        let mut v = self.inner.clone();
        v.dedup();
        if v.len() < L {
            return Err(BoundedVecOutOfBounds::LowerBoundError {
                lower_bound: L,
                got_smaller_by: L - v.len(),
            });
        }
        self.inner = v;
        Ok(())
    }
}

impl<T, const U: usize> BoundedVec<T, 1, U, witnesses::NonEmpty<1, U>> {
    /// Creates a new non-empty `BoundedVec` containing a single element.
    ///
    /// # Example
    /// ```
    /// use const_bounded_collections::NonEmptyVec;
    /// let v = NonEmptyVec::new(42);
    /// assert_eq!(*v.first(), 42);
    /// ```
    pub fn new(first: T) -> Self {
        Self {
            inner: vec![first],
            witness: witnesses::non_empty(),
        }
    }

    /// Creates a new non-empty `BoundedVec` with the given capacity, containing a single initial element.
    pub fn with_capacity(capacity: usize, first: T) -> Self {
        let mut inner = Vec::with_capacity(capacity);
        inner.push(first);
        Self {
            inner,
            witness: witnesses::non_empty(),
        }
    }

    /// Creates a non-empty `BoundedVec` from an initial element and a tail `Vec`.
    #[cfg_attr(all(test, not(debug_assertions)), no_panic::no_panic)]
    pub fn from_head_tail(first: T, mut rest: Vec<T>) -> Result<Self, BoundedVecOutOfBounds> {
        let rest_len = rest.len();
        let len = match rest_len.checked_add(1) {
            Some(len) if len <= U => len,
            Some(len) => {
                return Err(BoundedVecOutOfBounds::UpperBoundError {
                    upper_bound: U,
                    got_larger_by: len - U,
                });
            }
            None => {
                return Err(BoundedVecOutOfBounds::UpperBoundError {
                    upper_bound: U,
                    got_larger_by: larger_by_after_adding(rest_len, 1, U),
                });
            }
        };
        let mut inner = Vec::with_capacity(len);
        inner.push(first);
        inner.append(&mut rest);
        Ok(Self {
            inner,
            witness: witnesses::non_empty(),
        })
    }

    /// Removes consecutive duplicate elements in the vector.
    ///
    /// For a vector with lower bound `1`, deduplication is guaranteed to leave
    /// at least 1 element, preserving the non-empty invariant.
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.inner.dedup();
    }

    /// Removes consecutive elements that satisfy the given predicate.
    ///
    /// Guaranteed to preserve at least 1 element for lower bound `1`.
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.inner.dedup_by(same_bucket);
    }

    /// Removes consecutive duplicate elements according to a key.
    ///
    /// Guaranteed to preserve at least 1 element for lower bound `1`.
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.inner.dedup_by_key(key);
    }
}

/// A non-empty Vec with no effective upper-bound on its length.
/// Lenght is bounded to `u32::MAX``
#[cfg(any(feature = "schemars", feature = "borsh", feature = "borsh_schema"))]
pub type NonEmptyVec<T> =
    BoundedVec<T, 1, { u32::MAX as usize }, witnesses::NonEmpty<1, { u32::MAX as usize }>>;

/// A non-empty Vec with no effective upper-bound on its length
#[cfg(not(any(feature = "schemars", feature = "borsh", feature = "borsh_schema")))]
pub type NonEmptyVec<T> = BoundedVec<T, 1, { usize::MAX }, witnesses::NonEmpty<1, { usize::MAX }>>;

/// Possibly empty Vec with upper-bound on its length
pub type EmptyBoundedVec<T, const U: usize> = BoundedVec<T, 0, U, witnesses::Empty<U>>;

/// Non-empty Vec with bounded length
pub type NonEmptyBoundedVec<T, const L: usize, const U: usize> =
    BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>;

impl<T, const L: usize, const U: usize> TryFrom<Vec<T>>
    for BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>
{
    type Error = BoundedVecOutOfBounds;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::from_vec(value)
    }
}

impl<T, const U: usize> TryFrom<Vec<T>> for BoundedVec<T, 0, U, witnesses::Empty<U>> {
    type Error = BoundedVecOutOfBounds;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::from_vec(value)
    }
}

// when feature(const_evaluatable_checked) is stable cover all array sizes (L..=U)
impl<T, const L: usize, const U: usize> From<[T; L]>
    for BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>
{
    fn from(arr: [T; L]) -> Self {
        BoundedVec {
            inner: arr.into(),
            witness: witnesses::non_empty(),
        }
    }
}

impl<T, const L: usize, const U: usize> From<BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>>
    for Vec<T>
{
    fn from(v: BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>) -> Self {
        v.inner
    }
}

impl<T, const L: usize, const U: usize, W> IntoIterator for BoundedVec<T, L, U, W> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T, const L: usize, const U: usize, W> IntoIterator for &'a BoundedVec<T, L, U, W> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, T, const L: usize, const U: usize, W> IntoIterator for &'a mut BoundedVec<T, L, U, W> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl<T, const L: usize, const U: usize, W> AsRef<Vec<T>> for BoundedVec<T, L, U, W> {
    fn as_ref(&self) -> &Vec<T> {
        &self.inner
    }
}

impl<T, const L: usize, const U: usize, W> AsRef<[T]> for BoundedVec<T, L, U, W> {
    fn as_ref(&self) -> &[T] {
        self.inner.as_ref()
    }
}

#[cfg(feature = "panic")]
impl<T, const L: usize, const U: usize, W> AsMut<Vec<T>> for BoundedVec<T, L, U, W> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        self.inner.as_mut()
    }
}

impl<T, const L: usize, const U: usize, W> AsMut<[T]> for BoundedVec<T, L, U, W> {
    fn as_mut(&mut self) -> &mut [T] {
        self.inner.as_mut()
    }
}

impl<T, const L: usize, const U: usize, W> core::ops::Deref for BoundedVec<T, L, U, W> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const L: usize, const U: usize, W> core::ops::DerefMut for BoundedVec<T, L, U, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const L: usize, const U: usize, W> core::borrow::Borrow<[T]> for BoundedVec<T, L, U, W> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const L: usize, const U: usize, W> core::borrow::BorrowMut<[T]> for BoundedVec<T, L, U, W> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const L: usize, const U: usize, W, I: core::slice::SliceIndex<[T]>> core::ops::Index<I>
    for BoundedVec<T, L, U, W>
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T, const L: usize, const U: usize, W, I: core::slice::SliceIndex<[T]>> core::ops::IndexMut<I>
    for BoundedVec<T, L, U, W>
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

#[cfg(feature = "panic")]
impl<T, const L: usize, const U: usize, W> Extend<T> for BoundedVec<T, L, U, W> {
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        for item in iter {
            self.push(item);
        }
    }
}

#[cfg(feature = "panic")]
impl<'a, T: Clone + 'a, const L: usize, const U: usize, W> Extend<&'a T>
    for BoundedVec<T, L, U, W>
{
    fn extend<Iter: IntoIterator<Item = &'a T>>(&mut self, iter: Iter) {
        for item in iter {
            self.push(item.clone());
        }
    }
}

/// `Option<BoundedVec<T, _, _>>` to `Vec<T>`
pub trait OptBoundedVecToVec<T> {
    /// `Option<BoundedVec<T, _, _>>` to `Vec<T>`
    fn to_vec(self) -> Vec<T>;
}

impl<T, const L: usize, const U: usize> OptBoundedVecToVec<T>
    for Option<BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>>
{
    fn to_vec(self) -> Vec<T> {
        self.map(|bv| bv.into()).unwrap_or_default()
    }
}

impl<T, const U: usize> From<BoundedVec<T, 0, U>> for EmptyBoundedVec<T, U> {
    fn from(this: BoundedVec<T, 0, U>) -> Self {
        Self {
            inner: this.inner,
            witness: witnesses::empty(),
        }
    }
}

/// Supports encoding and decoding with [borsh](https://crates.io/crates/borsh).
/// BorshSchema is supported behind the `borsh_schema` feature.
///
/// By default Borsh uses u32 as length prefix for sequences.
/// For bounded we used u8, u16 or u32 depending on the U.
/// Increase or decreasing U may not always be backward compatible.
#[cfg(feature = "borsh")]
mod borsh_impl {
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};

    impl<T: BorshSerialize, const L: usize, const U: usize, W> BorshSerialize
        for BoundedVec<T, L, U, W>
    {
        fn serialize<Writer: borsh::io::Write>(
            &self,
            writer: &mut Writer,
        ) -> borsh::io::Result<()> {
            let len = self.inner.len();
            if U <= usize::from(u8::MAX) {
                #[expect(clippy::expect_used)]
                let len: u8 = len.try_into().expect("proved by design");
                len.serialize(writer)?;
            } else if U <= usize::from(u16::MAX) {
                #[expect(clippy::expect_used)]
                let len: u16 = len.try_into().expect("proved by design");
                len.serialize(writer)?;
            } else {
                #[expect(clippy::expect_used)]
                let len: u32 = len.try_into().expect("proved by design");
                len.serialize(writer)?;
            };

            // adapted from internals of borsh-rs
            let data = self.as_slice();
            if let Some(u8_slice) = T::u8_slice(data) {
                writer.write_all(u8_slice)?;
            } else {
                for item in data {
                    item.serialize(writer)?;
                }
            }
            Ok(())
        }
    }

    fn read_items<T: BorshDeserialize, const U: usize, R: borsh::io::Read>(
        reader: &mut R,
        min_len: usize,
    ) -> borsh::io::Result<Vec<T>> {
        let len = if U <= usize::from(u8::MAX) {
            usize::from(u8::deserialize_reader(reader)?)
        } else if U <= usize::from(u16::MAX) {
            usize::from(u16::deserialize_reader(reader)?)
        } else {
            let len = u32::deserialize_reader(reader)?;
            usize::try_from(len).map_err(|_| {
                borsh::io::Error::new(
                    borsh::io::ErrorKind::Other,
                    alloc::format!("Length overflow: got {len}"),
                )
            })?
        };
        if len < min_len {
            return Err(borsh::io::Error::new(
                borsh::io::ErrorKind::Other,
                alloc::format!("Lower bound violation: got {len} (expected >= {min_len})"),
            ));
        } else if len > U {
            return Err(borsh::io::Error::new(
                borsh::io::ErrorKind::Other,
                alloc::format!("Upper bound violation: got {len} (expected <= {U})"),
            ));
        }
        // adapted from internals for borsh-rs
        let data = if len == 0 {
            Vec::new()
        } else if let Some(vec_bytes) = T::vec_from_reader(len as u32, reader)? {
            vec_bytes
        } else {
            let el_size = core::mem::size_of::<T>() as u32;
            let cautious = core::cmp::max(core::cmp::min(len as u32, 4096 / el_size), 1) as usize;

            // TODO(16): return capacity allocation when we can safely do that.
            let mut result = Vec::with_capacity(cautious);
            for _ in 0..len {
                result.push(T::deserialize_reader(reader)?);
            }
            result
        };

        Ok(data)
    }

    impl<T: BorshDeserialize, const U: usize> BorshDeserialize for EmptyBoundedVec<T, U> {
        fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
            let data = read_items::<T, U, R>(reader, 0)?;
            Ok(Self {
                inner: data,
                witness: witnesses::empty(),
            })
        }
    }

    impl<T: BorshDeserialize, const L: usize, const U: usize> BorshDeserialize for BoundedVec<T, L, U> {
        fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
            let data = read_items::<T, U, R>(reader, L)?;
            Ok(Self {
                inner: data,
                witness: witnesses::non_empty(),
            })
        }
    }

    #[cfg(feature = "borsh_schema")]
    mod schema {
        use super::*;
        use alloc::collections::btree_map::{BTreeMap, Entry};
        use borsh::BorshSchema;

        impl<T: BorshSchema, const L: usize, const U: usize, W> BorshSchema for BoundedVec<T, L, U, W> {
            fn add_definitions_recursively(
                definitions: &mut BTreeMap<borsh::schema::Declaration, borsh::schema::Definition>,
            ) {
                let len_width = if U <= usize::from(u8::MAX) {
                    1
                } else if U <= usize::from(u16::MAX) {
                    2
                } else {
                    4 // proven by design
                };

                let definition = borsh::schema::Definition::Sequence {
                    length_width: len_width,
                    #[expect(clippy::expect_used)]
                    length_range: core::ops::RangeInclusive::<u64>::new(
                        u64::try_from(L).expect("proved by design"),
                        u64::try_from(U).expect("proved by design"),
                    ),
                    elements: T::declaration(),
                };
                match definitions.entry(Self::declaration()) {
                    Entry::Occupied(occ) => {
                        let existing_def = occ.get();
                        assert_eq!(
                            existing_def,
                            &definition,
                            "Redefining type schema for {}. Types with the same names are not supported.",
                            occ.key()
                        );
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(definition);
                    }
                }
                T::add_definitions_recursively(definitions);
            }

            fn declaration() -> borsh::schema::Declaration {
                alloc::format!("BoundedVec<{}, {}, {}>", T::declaration(), L, U)
            }
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
mod arbitrary {

    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::Arbitrary;
    use proptest::prelude::*;
    use proptest::strategy::BoxedStrategy;

    impl<T: Arbitrary, const L: usize, const U: usize> Arbitrary
        for BoundedVec<T, L, U, witnesses::NonEmpty<L, U>>
    where
        T::Strategy: 'static,
    {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any::<T>(), L..=U)
                .prop_map(|items| Self::from_vec(items).unwrap())
                .boxed()
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Serialize};

    // direct impl to unify serde in one place instead of doing attribute on declaration and deserialize here
    impl<T: Serialize, const L: usize, const U: usize, W> Serialize for BoundedVec<T, L, U, W> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.inner.serialize(serializer)
        }
    }

    impl<'de, T: Deserialize<'de>, const U: usize> Deserialize<'de> for EmptyBoundedVec<T, U> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let inner = Vec::<T>::deserialize(deserializer)?;
            Self::from_vec(inner).map_err(serde::de::Error::custom)
        }
    }

    impl<'de, T: Deserialize<'de>, const L: usize, const U: usize> Deserialize<'de>
        for BoundedVec<T, L, U>
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let inner = Vec::<T>::deserialize(deserializer)?;
            Self::from_vec(inner).map_err(serde::de::Error::custom)
        }
    }

    #[cfg(feature = "schemars")]
    mod schema {
        use super::*;
        use alloc::borrow::Cow;

        // we cannot use `serde` attributes, because these do not work with `const`, only numeric literals supported
        impl<T: schemars::JsonSchema, const L: usize, const U: usize, W> schemars::JsonSchema
            for BoundedVec<T, L, U, W>
        {
            fn schema_name() -> Cow<'static, str> {
                alloc::format!("BoundedVec{}Min{}Max{}", T::schema_name(), L, U).into()
            }

            fn json_schema(r#gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "type": "array",
                    "items": T::json_schema(r#gen),
                    "minItems": L as u32,
                    "maxItems": U as u32
                })
            }
        }
    }
}

#[cfg(test)]
mod no_panic_tests {
    use super::*;

    #[test]
    fn fallible_growth_does_not_panic() {
        let mut vector = EmptyBoundedVec::<u8, 4>::with_capacity(4);
        assert!(vector.try_push(1).is_ok());
        assert!(vector.try_extend([2, 3]).is_ok());
        assert!(vector.try_extend([4, 5]).is_err());
        assert!(vector.try_insert(1, 4).is_ok());
        assert!(vector.try_insert(0, 5).is_err());
    }
}
