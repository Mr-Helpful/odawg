pub trait IsSorted: Iterator + Sized
where
    Self::Item: Ord,
{
    /// Tests if an iterator is sorted using `Ord::cmp`.
    /// ```
    /// # use odawg::utils::IsSorted;
    /// let sorted = [1, 2, 3, 4, 5].into_iter();
    /// assert_eq!(sorted.sorted(), true);
    ///
    /// let unsorted = [1, 2, 4, 3, 5].into_iter();
    /// assert_eq!(unsorted.sorted(), false)
    /// ```
    fn sorted(mut self) -> bool {
        let Some(mut last) = self.next() else {
            return true;
        };

        for item in self {
            if last > item {
                return false;
            }
            last = item;
        }

        true
    }
}

impl<I: Iterator> IsSorted for I where I::Item: Ord {}
