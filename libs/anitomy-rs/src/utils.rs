use phf::phf_map;
use uncased::UncasedStr;

pub(crate) fn from_ordinal_number(s: &str) -> Option<&'static str> {
    static LOOKUP: phf::Map<&'static UncasedStr, &'static str> = phf_map! {
        UncasedStr::new("1st") => "1",
        UncasedStr::new("2nd") => "2",
        UncasedStr::new("3rd") => "3",
        UncasedStr::new("4th") => "4",
        UncasedStr::new("5th") => "5",
        UncasedStr::new("6th") => "6",
        UncasedStr::new("7th") => "7",
        UncasedStr::new("8th") => "8",
        UncasedStr::new("9th") => "9",
        UncasedStr::new("First") =>   "1",
        UncasedStr::new("Second") =>  "2",
        UncasedStr::new("Third") =>   "3",
        UncasedStr::new("Fourth") =>  "4",
        UncasedStr::new("Fifth") =>   "5",
        UncasedStr::new("Sixth") =>   "6",
        UncasedStr::new("Seventh") => "7",
        UncasedStr::new("Eighth") =>  "8",
        UncasedStr::new("Ninth") =>   "9",
    };
    LOOKUP.get(UncasedStr::new(s)).copied()
}

pub(crate) fn from_roman_number(s: &str) -> Option<&'static str> {
    static LOOKUP: phf::Map<&'static str, &'static str> = phf_map! {
        "II" => "2",
        "III" => "3",
        "IV" => "4",
        "V" => "5",
        "VI" => "6",
        "VII" => "7",
    };
    LOOKUP.get(s).copied()
}

// This is borrowed and modified from the stdlib
// FIXME: Remove when MSRV is bumped to 1.77 (currently 1.74)
pub(crate) fn last_chunk_mut<const N: usize, T>(slice: &mut [T]) -> Option<&mut [T; N]> {
    if slice.len() < N {
        None
    } else {
        let last = slice.split_at_mut(slice.len() - N).1;

        // SAFETY: We explicitly check for the correct number of elements,
        //   do not let the reference outlive the slice,
        //   and require exclusive access to the entire slice to mutate the chunk.
        Some(unsafe { &mut *(last.as_mut_ptr().cast::<[T; N]>()) })
    }
}

// /// Returns a pair from the slice where the first element meets the predicate and the following element meets
// /// the other predicate.
// ///
// /// This returns two mutable references.
// pub(crate) fn find_window_pair_mut<T, F, P>(
//     slice: &mut [T],
//     mut first: F,
//     mut second: P,
// ) -> Option<(&mut T, &mut T)>
// where
//     F: FnMut(&T) -> bool,
//     P: FnMut(&T) -> bool,
// {
//     let total = slice.len().saturating_sub(2);
//     for start in 0..=total {
//         if first(&slice[start]) && second(&slice[start + 1]) {
//             let (left, right) = slice.split_at_mut(start + 1);
//             return Some((&mut left[start], &mut right[0]));
//         }
//     }
//     None
// }

/// Returns a pair of disjoint mutable references to the slice at index `i` and `j`.
///
/// `i` must be smaller than `j` and `j` must be smaller than the length of the slice,
/// otherwise `None` is returned.
pub(crate) fn get_pair_mut<T>(slice: &mut [T], i: usize, j: usize) -> Option<(&mut T, &mut T)> {
    if i >= j || j > slice.len() {
        None
    } else {
        let (left, right) = slice.split_at_mut(j);
        Some((&mut left[i], &mut right[0]))
    }
}

/// Finds the first element that matches the first predicate and then
/// the second element that follows it that meets the second predicate.
///
/// They do not have to be next to each other.
pub(crate) fn find_pair_mut<T, F, P>(
    slice: &mut [T],
    first: F,
    second: P,
) -> Option<(&mut T, &mut T)>
where
    F: FnMut(&T) -> bool,
    P: FnMut(&T) -> bool,
{
    let index = slice.iter().position(first)?;
    let offset = index + 1;
    let second_index = offset + slice.iter().skip(offset).position(second)?;
    let (left, right) = slice.split_at_mut(second_index);
    Some((&mut left[index], &mut right[0]))
}

pub(crate) trait LendingIterator {
    type Item<'this>
    where
        Self: 'this;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

pub(crate) struct WindowsMut<'a, T, const SIZE: usize> {
    slice: &'a mut [T],
    start: usize,
}

impl<'a, T, const SIZE: usize> LendingIterator for WindowsMut<'a, T, SIZE> {
    type Item<'this>
        = &'this mut [T; SIZE]
    where
        'a: 'this;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let result = self
            .slice
            .get_mut(self.start..)?
            .get_mut(..SIZE)?
            .try_into()
            .unwrap();
        self.start += 1;
        Some(result)
    }
}

pub(crate) fn windows_mut<T, const SIZE: usize>(slice: &mut [T]) -> WindowsMut<'_, T, SIZE> {
    assert_ne!(SIZE, 0);
    WindowsMut { slice, start: 0 }
}
