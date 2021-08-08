use std::mem::ManuallyDrop;

// https://stackoverflow.com/a/48330314/195651

type IndexType = u32;
const SENTINEL: IndexType = IndexType::MAX;

/// Provides an indexed free list with constant-time removals from anywhere
/// in the list without invalidating indices. T must be trivially constructible
/// and destructible.
pub struct FreeList<T>
where
    T: Default,
{
    data: Vec<FreeElement<T>>,
    /// The index of the the most recently freed element, or `SENTINEL` if no
    /// element is free.
    first_free: IndexType,
}

union FreeElement<T> {
    /// This field contains the data as long as the element was not removed.
    element: ManuallyDrop<T>,
    /// If the element was "removed", this index is pointing to the next index
    /// of an element that is also freed, or `SENTINEL` if no other element is free.
    next: IndexType,
}

impl<T> Default for FreeList<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            data: Vec::default(),
            first_free: SENTINEL,
        }
    }
}

impl<T> FreeList<T>
where
    T: Default,
{
    /// Inserts an element to the free list and returns an index to it.
    pub fn insert(&mut self, element: T) -> IndexType {
        return if self.first_free != SENTINEL {
            let index = self.first_free;

            // Set the "first free" pointer to the next free index.
            self.first_free = unsafe { self.data[self.first_free as usize].next };

            // Place the element into the previously free location.
            self.data[index as usize].element = ManuallyDrop::new(element);
            index
        } else {
            let fe = FreeElement {
                element: ManuallyDrop::new(element),
            };
            self.data.push(fe);
            (self.data.len() - 1) as IndexType
        };
    }

    /// Removes the nth element from the free list.
    pub fn erase(&mut self, n: IndexType) {
        self.first_free = SENTINEL;
        if self.data.is_empty() {
            return;
        }

        debug_assert!(!self.is_in_free_list(n));

        unsafe { ManuallyDrop::drop(&mut self.data[n as usize].element) };
        self.data[n as usize].next = self.first_free;
        self.first_free = n;
    }

    /// Removes all elements from the free list.
    pub fn clear(&mut self) {
        if self.data.is_empty() {
            assert_eq!(self.first_free, SENTINEL);
            return;
        }

        // Collect all free indexes and sort them such that they
        // are in ascending order.
        let mut free_indexes = Vec::new();
        let mut token = self.first_free;
        while token != SENTINEL {
            free_indexes.push(token);
            token = unsafe { self.data[token as usize].next };
        }
        free_indexes.sort();

        // As long as there are free indexes, pop elements from the
        // vector and ignore them if they correspond to a free index.
        if !free_indexes.is_empty() {
            for (i, entry) in self.data.iter_mut().enumerate() {
                if free_indexes.is_empty() || *free_indexes.last().unwrap() != (i as IndexType) {
                    // This is not a pointer entry, drop required.
                    unsafe { ManuallyDrop::drop(&mut entry.element) };
                } else {
                    // The entry only contains a index to another free spot; nothing to drop.
                    let _ = free_indexes.pop();
                }
            }
        }

        // At this point there are no free indexes anymore, so the
        // list can be trivially cleared.
        self.data.clear();
        self.first_free = SENTINEL;
    }

    /// Gets a reference to the value at the specified index.
    ///
    /// # Safety
    ///
    /// If the element at the specified index was erased, the union now acts
    ///  as a pointer to the next free element. Accessing the same index again after that.
    ///  is undefined behavior.
    pub unsafe fn at(&self, index: IndexType) -> &T {
        assert_ne!(index, SENTINEL);
        debug_assert!(!self.is_in_free_list(index));
        &self.data[index as usize].element
    }

    /// Gets a mutable reference to the value at the specified index.
    ///
    /// # Safety
    ///
    /// If the element at the specified index was erased, the union now acts
    /// as a pointer to the next free element. Accessing the same index again after that.
    /// is undefined behavior.
    pub unsafe fn at_mut(&mut self, index: IndexType) -> &mut T {
        assert_ne!(index, SENTINEL);
        debug_assert!(!self.is_in_free_list(index));
        &mut self.data[index as usize].element
    }

    /// Gets the current capacity of the list.
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    #[cfg(any(debug_assertions, test))]
    fn is_in_free_list(&self, n: IndexType) -> bool {
        assert_ne!(n, SENTINEL);
        let mut token = self.first_free;
        while token != SENTINEL {
            if n == token {
                return true;
            }
            token = unsafe { self.data[token as usize].next };
        }
        return false;
    }
}

impl<T> Drop for FreeList<T>
where
    T: Default,
{
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Default, Debug, PartialEq, PartialOrd)]
    struct Complex(f64, f64);

    impl Drop for Complex {
        fn drop(&mut self) {
            self.0 = 42.;
            self.1 = 1337.0;
        }
    }

    #[test]
    fn after_construction_has_no_first_free() {
        let list = FreeList::<Complex>::default();
        assert_eq!(list.first_free, SENTINEL);
        assert_eq!(list.capacity(), 0);
    }

    #[test]
    fn after_insertion_has_no_first_free() {
        let mut list = FreeList::<Complex>::default();
        assert_eq!(list.insert(Complex::default()), 0);
        assert_eq!(list.first_free, SENTINEL);
        assert_eq!(list.capacity(), 1);
    }

    #[test]
    fn after_deletion_has_a_first_free() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex::default());
        list.erase(0);
        assert_eq!(list.first_free, 0);
        assert_eq!(list.capacity(), 1);
    }

    #[test]
    fn insert_after_delete_has_no_free() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex::default());
        list.erase(0);
        list.insert(Complex::default());
        assert_eq!(list.first_free, SENTINEL);
        assert_eq!(list.capacity(), 1);
    }

    #[test]
    fn first_free_points_to_last_freed_index() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.erase(0);
        list.erase(1);
        assert_eq!(list.first_free, 1);
        assert_eq!(list.capacity(), 2);
    }

    #[test]
    fn erase_in_ascending_order() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.erase(0);
        list.erase(1);
        list.erase(2);
        list.erase(3);
        assert_eq!(list.first_free, 3);
        assert_eq!(list.capacity(), 4);
    }

    #[test]
    fn erase_in_descending_order() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.erase(3);
        list.erase(2);
        list.erase(1);
        list.erase(0);
        assert_eq!(list.first_free, 0);
        assert_eq!(list.capacity(), 4);
    }

    #[test]
    fn erase_in_mixed_order() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.erase(0);
        list.erase(3);
        list.erase(1);
        list.erase(2);
        assert_eq!(list.first_free, 2);
        assert_eq!(list.capacity(), 4);
    }

    #[test]
    fn clear_works() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.erase(1);
        list.clear();
        list.clear();
        assert_eq!(list.first_free, SENTINEL);
        assert_eq!(list.capacity(), 0);
    }

    #[test]
    fn is_in_free_list_works() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex::default());
        list.insert(Complex::default());
        list.erase(0);
        assert!(list.is_in_free_list(0));
        assert!(!list.is_in_free_list(1));
    }

    #[test]
    fn at_works() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex(1., 2.));
        list.insert(Complex::default());
        let element = unsafe { list.at(0) };
        assert_eq!(*element, Complex(1., 2.));
    }

    #[test]
    fn at_mut_works() {
        let mut list = FreeList::<Complex>::default();
        list.insert(Complex(1., 2.));
        list.insert(Complex::default());

        // Mutably access the element and exchange it.
        let element = unsafe { list.at_mut(0) };
        *element = Complex::default();

        // Get a new reference and verify.
        let element = unsafe { list.at(0) };
        assert_eq!(*element, Complex(0., 0.));
    }
}
