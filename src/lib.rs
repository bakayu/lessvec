//! A minimal `Vec`-like collection implemented with manual allocation.
//!
//! This crate provides `MiniVec<T>`, a small, educational reimplementation of
//! `std::vec::Vec<T>` following patterns from the Rustonomicon. It's intended
//! for learning and small use-cases, not as a drop-in replacement for `Vec`.
//!
//! # Examples
//! ```
//! use minivec::MiniVec;
//!
//! let mut v = MiniVec::new();
//! v.push(1);
//! v.push(2);
//! assert_eq!(&*v, &[1, 2]);
//! assert_eq!(v.pop(), Some(2));
//! ```
//!
//! See individual method docs for more examples.
use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            0
        };

        // `NonNull::dangling` doubles as "unallocated" and "zero-sized allocation"
        RawVec {
            ptr: NonNull::dangling(),
            cap: cap,
        }
    }

    fn grow(&mut self) {
        // since we set the capacity to usize::MAX when T has size 0, getting
        // to here means Vec is overfull.
        assert!(mem::size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            // `new_cap` will not overflow because `self.cap <= isize::MAX`.
            let new_cap = 2 * self.cap;

            // `Layout::array` checks that the number of bytes is <= usize::MAX,
            // but this is redundant since old_layout.size() <= isize::MAX,
            // `unwrap` should never fail.
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't overflow; <= isize::MAX bytes.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // new_ptr becomes null if allocation fails, abort.
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };

        self.cap = new_cap;
    }

    fn grow_to(&mut self, new_cap: usize) {
        if mem::size_of::<T>() == 0 {
            return;
        }
        if new_cap <= self.cap {
            return;
        }

        let new_layout = Layout::array::<T>(new_cap).unwrap();
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_tr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_tr, old_layout, new_layout.size() as usize) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };

        self.cap = new_cap;
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 && std::mem::size_of::<T>() > 0 {
            let layout = std::alloc::Layout::array::<T>(self.cap).unwrap();
            unsafe {
                std::alloc::dealloc(self.ptr.as_ptr() as *mut _, layout);
            }
        }
    }
}

/// A minimal growable contiguous vector.
///
/// `MiniVec<T>` stores elements in a heap buffer and supports a small set of
/// `Vec`-like operations. Use the methods below to manipulate the collection.
///
/// # Examples
///
/// ```
/// use minivec::MiniVec;
///
/// let mut v = MiniVec::new();
/// v.push(10);
/// v.push(20);
/// assert_eq!(v.len(), 2);
/// assert!(!v.is_empty());
/// assert!(v.capacity() >= 2);
/// ```
pub struct MiniVec<T> {
    buf: RawVec<T>,
    len: usize,
}

unsafe impl<T: Send> Send for MiniVec<T> {}
unsafe impl<T: Sync> Sync for MiniVec<T> {}

impl<T> MiniVec<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    /// Creates a new, empty `MiniVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let v: MiniVec<i32> = MiniVec::new();
    /// assert_eq!(v.len(), 0);
    /// ```
    pub fn new() -> Self {
        MiniVec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    /// Returns the number of elements in the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// assert!(v.is_empty());
    /// v.push(1);
    /// assert!(!v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements the `MiniVec` can hold without reallocating.
    ///
    /// Note: capacity may be larger than the number of elements. Calling
    /// `reserve` or `reserve_exact` increases capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v: MiniVec<i32> = MiniVec::new();
    /// v.reserve(5);
    /// assert!(v.capacity() >= 5);
    /// ```
    pub fn capacity(&self) -> usize {
        self.cap()
    }

    /// Clears the vector, removing all values.
    ///
    /// This drops each element in the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(1);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    pub fn clear(&mut self) {
        while let Some(_) = self.pop() {}
    }

    /// Returns a slice containing all elements of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(1);
    /// assert_eq!(v.as_slice(), &[1]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &*self
    }

    /// Returns a mutable slice containing all elements of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(1);
    /// v.as_mut_slice()[0] = 2;
    /// assert_eq!(v.as_slice(), &[2]);
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut *self
    }

    /// Ensures the `MiniVec` can hold at least `additional` more elements without reallocating.
    ///
    /// This grows capacity exponentially where necessary.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v: MiniVec<i32> = MiniVec::new();
    /// v.reserve(10);
    /// assert!(v.capacity() >= 10);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        let required = self.len.checked_add(additional).expect("capacity overflow");
        if self.cap() >= required {
            return;
        }
        while self.cap() < required {
            self.buf.grow();
        }
    }

    /// Ensures the `MiniVec` has capacity for exactly `additional` more elements (no fewer).
    ///
    /// Unlike `reserve`, this attempts to allocate the exact requested capacity in one go.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v: MiniVec<i32> = MiniVec::new();
    /// v.reserve_exact(3);
    /// assert!(v.capacity() >= 3);
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        let required = self.len.checked_add(additional).expect("capacity overflow");
        if self.cap() >= required {
            return;
        }
        self.buf.grow_to(required);
    }

    /// Appends an element to the back of the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(5);
    /// assert_eq!(v.len(), 1);
    /// assert_eq!(v.as_slice(), &[5]);
    /// ```
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap() {
            self.buf.grow();
        }

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }

        // This operation will not fail, we will get OOM first.
        self.len += 1;
    }

    /// Removes the last element and returns it, or `None` if empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(10);
    /// assert_eq!(v.pop(), Some(10));
    /// assert_eq!(v.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    /// Inserts an element at `index`, shifting elements to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(1);
    /// v.push(3);
    /// v.insert(1, 2);
    /// assert_eq!(v.as_slice(), &[1, 2, 3]);
    /// ```
    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.len == self.cap() {
            self.buf.grow();
        }

        unsafe {
            ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr().add(index), elem);
        }

        self.len += 1;
    }

    /// Removes and returns the element at `index`, shifting elements left.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(1);
    /// v.push(2);
    /// assert_eq!(v.remove(0), 1);
    /// assert_eq!(v.as_slice(), &[2]);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr().add(index));
            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );
            result
        }
    }

    /// Removes all elements and returns an iterator that yields the removed elements.
    ///
    /// The vector's length is set to zero immediately; elements are yielded by the returned iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use minivec::MiniVec;
    /// let mut v = MiniVec::new();
    /// v.push(10);
    /// v.push(20);
    /// let drained: Vec<_> = v.drain().collect();
    /// assert_eq!(drained, vec![10, 20]);
    /// assert!(v.is_empty());
    /// ```
    pub fn drain(&'_ mut self) -> Drain<'_, T> {
        let iter = unsafe { RawValIter::new(&self) };

        self.len = 0;

        Drain {
            iter,
            vec: PhantomData,
        }
    }
}

impl<T> Drop for MiniVec<T> {
    fn drop(&mut self) {
        if self.len != 0 {
            // deallocation is handled by RawVec
            while let Some(_) = self.pop() {}
        }
    }
}

impl<T> Deref for MiniVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for MiniVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawValIter<T> {
    // unsafe construct because it has no associated lifetimes. This is
    // necessary to store RawValIter as in the same struct as its actual
    // allocation. OK to use since it's a private implementation detail.
    unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                unsafe { slice.as_ptr().add(slice.len()) }
            },
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.start = (self.start as usize + 1) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    let old_ptr = self.start;
                    self.start = self.start.offset(1);
                    Some(ptr::read(old_ptr))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len =
            (self.end as usize - self.start as usize) / if elem_size == 0 { 1 } else { elem_size };

        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.end = (self.end as usize - 1) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    self.end = self.end.offset(-1);
                    Some(ptr::read(self.end))
                }
            }
        }
    }
}

/// Iterator that yields values by value when consuming a `MiniVec`.
///
/// # Examples
///
/// ```
/// use minivec::MiniVec;
/// let mut v = MiniVec::new();
/// v.push(1);
/// v.push(2);
/// let out: Vec<_> = v.into_iter().collect();
/// assert_eq!(out, vec![1, 2]);
/// ```
pub struct IntoIter<T> {
    _buf: RawVec<T>,
    iter: RawValIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

impl<T> IntoIterator for MiniVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let iter = RawValIter::new(&self);

            let buf = ptr::read(&self.buf);
            mem::forget(self);

            IntoIter { _buf: buf, iter }
        }
    }
}

/// An iterator produced by `MiniVec::drain`.
///
/// Iterates over and yields the drained elements by value.
///
/// # Examples
///
/// ```
/// use minivec::MiniVec;
/// let mut v = MiniVec::new();
/// v.push(1);
/// v.push(2);
/// let drained: Vec<_> = v.drain().collect();
/// assert_eq!(drained, vec![1, 2]);
/// ```
pub struct Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut MiniVec<T>>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

#[cfg(test)]
mod tests {
    use crate::MiniVec;

    #[test]
    fn push_pop_roundtrip() {
        let mut v = MiniVec::new();
        v.push(1);
        v.push(2);
        v.push(3);

        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn insert_remove() {
        let mut v = MiniVec::new();
        v.push(1);
        v.push(3);
        v.insert(1, 2);

        assert_eq!(&*v, &[1, 2, 3]);
        assert_eq!(v.remove(1), 2);
        assert_eq!(&*v, &[1, 3]);
    }

    #[test]
    fn drain_consumes_elements() {
        let mut v = MiniVec::new();
        v.push(10);
        v.push(20);
        v.push(30);

        let drained: Vec<_> = v.drain().collect();
        assert_eq!(drained, vec![10, 20, 30]);
        assert_eq!(&*v, &[]);
    }

    #[test]
    fn into_iter_works() {
        let mut v = MiniVec::new();
        v.push(1);
        v.push(2);
        v.push(3);

        let collected: Vec<_> = v.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn reserve_and_capacity() {
        let mut v: MiniVec<i32> = MiniVec::new();
        v.reserve(5);
        assert!(v.capacity() >= 5);
        v.reserve_exact(10);
        assert!(v.capacity() >= 10);
    }

    #[test]
    fn clear_works() {
        let mut v = MiniVec::new();
        v.push(1);
        v.clear();
        assert!(v.is_empty());
    }

    #[test]
    fn as_slice_and_mut() {
        let mut v = MiniVec::new();
        v.push(1);
        assert_eq!(v.as_slice(), &[1]);
        v.as_mut_slice()[0] = 2;
        assert_eq!(v.as_slice(), &[2]);
    }
}
