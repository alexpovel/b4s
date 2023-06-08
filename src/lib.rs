#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::multiple_crate_versions)]
//! Binary Search Single Sorted String: Perform binary search on a single, delimited
//! string slice of sorted but unevenly sized substrings.

use ascii::AsciiChar;
use itertools::Itertools;
use std::{cmp::Ordering, error::Error, fmt::Display};

/// Main type to perform binary search through.
///
/// This type upholds the important invariants of [`SortedString::binary_search()`]:
///
/// - a *sorted* string is required,
/// - as well as some [`char`] separator to split by.
///
/// Access to binary search is gated behind this type. For this to not be too painful,
/// the type is designed to be cheap, as it doesn't own the potentially large haystack
/// to perform binary search on.
///
/// The terms *haystack* and *needle* are used here as they are in
/// [`std::str::pattern`].
///
/// # Instance Creation
///
/// There are two avenues:
///
/// 1. [`SortedString::new_checked()`]: recommended, ensuring soundness
/// 2. [`SortedString::new_unchecked()`]: unsafe, faster
///
/// ## Note on ASCII requirement
///
/// On instance creation, the separator has to be [within the ASCII
/// range](https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii). In other
/// words, **multi-byte (in the sense of UTF-8) separators are not supported**. This
/// leaves common separators like `\n`, `\t`, `-`, `,` and [many more][ascii::AsciiChar]
/// available, while arbitrary [`char`]s are not allowed:
///
/// ```compile_fail
/// use b4s::SortedString;
///
/// let sep = '🦀';
/// let haystack = "a,b,c";
/// // Not allowed:
/// let _ = SortedString::new_checked(haystack, sep);
/// ```
///
/// UTF-8 being a variable-length encoding, allowing *any* separator [`char`] would
/// require an initial [linear scan of the input
/// string](https://doc.rust-lang.org/std/primitive.str.html#method.char_indices):
///
/// - either at instance creation time, then saving the results,
/// - or before each search.
///
/// The former takes up a lot of space ([`usize`] pointers, which on 64-bit are much
/// larger than the separator itself), the latter time. Both are undesirable.
///
/// As UTF-8 is fully ASCII-compatible, allowing only that means the string can be
/// efficiently scanned as bytes. The needle and haystack can still be any UTF-8 string.
/// No multi-byte code point (with its continuation bytes) [contains bytes in the ASCII
/// range](https://en.wikipedia.org/wiki/UTF-8#Encoding). Scanning for such an ASCII
/// separator is therefore sound:
///
/// - it won't be found at [non-char
///   boundaries](https://doc.rust-lang.org/std/primitive.str.html#method.is_char_boundary),
/// - and it itself will be a boundary.
///
/// These properties simplify scanning and allow it to be fast at both instance creation
/// and search time. The downside is hopefully negligible and in the common case
/// unnoticeable. Please contact the author if your use case requires multi-byte
/// support.
///
/// The ASCII requirement is manifested in the [type
/// system](https://dusted.codes/the-type-system-is-a-programmers-best-friend),
/// enforcing correct usage [at compile time](https://cliffle.com/blog/rust-typestate/).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortedString<'a> {
    string: &'a str,
    sep: AsciiChar,
}

/// The result of a [`SortedString::binary_search()`].
///
/// In the success case, the location where the match was found.
/// In the error case, an appropriate error.
pub type SearchResult = Result<Span, SearchError>;

/// Error returned in case of an unsuccessful search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchError {
    /// The needle was not found.
    NotFound(
        /// Location of the last *unsuccessful* comparison.
        ///
        /// This is *not* the location where the needle could be inserted. That location
        /// is either to the left *or* right of this location, depending on how
        /// [comparison](https://doc.rust-lang.org/std/primitive.str.html#impl-PartialOrd%3Cstr%3E-for-str)
        /// goes. As this error is not particularly actionable at runtime, the exact
        /// location (left, right) is not reported, saving computation at runtime.
        Span,
    ),
    /// An internal encoding error occurred when slicing UTF8 from raw bytes.
    /// Should never happen, but variant exists to prevent panics.
    EncodingError,
}

impl Error for SearchError {}

impl Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::NotFound(span) => {
                write!(f, "Not found, last unsuccessful comparison at {span:?}")
            }
            SearchError::EncodingError => write!(f, "Encoding error"),
        }
    }
}

impl<'a> SortedString<'a> {
    /// Creates a new instance of [`SortedString`], performing sanity checks.
    ///
    /// See [`SortedString::new_unchecked()`] for a version without checks.
    ///
    /// The performed checks and failure modes are outlined in the [error
    /// examples](#errors) below.
    ///
    /// # Example
    ///
    /// ```
    /// use ascii::AsciiChar;
    /// use b4s::SortedString;
    ///
    /// let sep = AsciiChar::Comma;
    /// let haystack = "a,b,c";
    /// let sorted_string = SortedString::new_checked(haystack, sep);
    ///
    /// assert!(sorted_string.is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// This method returns a [`SortedStringCreationError`] in the cases outlined below.
    ///
    /// ## Example: Creation Fails (Haystack Not Sorted)
    ///
    /// Binary search requires prior sorting, so creation has to fail.
    ///
    /// ```
    /// use ascii::AsciiChar;
    /// use b4s::SortedString;
    ///
    /// let sep = AsciiChar::Comma;
    /// let unsorted_haystack = "a,c,b";
    /// let sorted_string = SortedString::new_checked(unsorted_haystack, sep);
    ///
    /// assert_eq!(sorted_string, Err(b4s::SortedStringCreationError::NotSorted));
    /// ```
    ///
    /// ## Example: Creation Fails (Haystack Empty)
    ///
    /// Supplying an empty haystack [`str`] is likely an error and therefore fails.
    ///
    /// ```
    /// use ascii::AsciiChar;
    /// use b4s::SortedString;
    ///
    /// let sep = AsciiChar::Comma;
    /// let haystack = "";
    /// let sorted_string = SortedString::new_checked(haystack, sep);
    ///
    /// assert_eq!(sorted_string, Err(b4s::SortedStringCreationError::EmptyHaystack));
    /// ```
    pub fn new_checked(
        haystack: &'a str,
        sep: AsciiChar,
    ) -> Result<Self, SortedStringCreationError> {
        if haystack.is_empty() {
            return Err(SortedStringCreationError::EmptyHaystack);
        }

        let sorted_string = Self::new(haystack, sep);

        if sorted_string.is_sorted() {
            Ok(sorted_string)
        } else {
            Err(SortedStringCreationError::NotSorted)
        }
    }

    /// Searches for a needle inside this [`SortedString`].
    ///
    /// The return type aims to imitate [`binary_search` of the standard
    /// library](https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search),
    /// returning a [`Result`]. The success case is a [`Span`], not a single [`usize`],
    /// as the haystack is variable-length. For the error case, see below.
    ///
    /// # Examples
    ///
    ///
    ///
    /// # Errors
    ///
    /// This method returns a [`usize`] in the error case, indicating where the needle
    /// would have had to be, and where it could be inserted.
    ///
    /// The special-case result of `Err(usize::MAX)` indicates that UTF-8 conversions
    /// went wrong. It's an ugly API that might be improved in the future. Its purpose
    /// is to avoid panics, giving the user a chance to recover.
    pub fn binary_search<U>(&self, needle: U) -> SearchResult
    where
        U: AsRef<str>,
    {
        let haystack = self.string;
        let needle = needle.as_ref();

        let leftmost = 0;
        let rightmost = haystack.len();

        let mut low = leftmost;
        let mut high = rightmost;

        let mut start = leftmost;
        let mut end = rightmost;

        let haystack = haystack.as_bytes();

        let pred = |c: &&u8| **c == self.sep.as_byte();

        while low < high {
            let mid = low + (high - low) / 2;

            start = match haystack[..mid].iter().rev().find_position(pred) {
                Some((delta, _)) => mid - delta,
                None => leftmost,
            };

            end = match haystack[mid..].iter().find_position(pred) {
                Some((delta, _)) => mid + delta,
                None => rightmost,
            };

            let Ok(haystack_word) = std::str::from_utf8(&haystack[start..end])
                else { return Err(SearchError::EncodingError) };

            match needle.cmp(haystack_word) {
                Ordering::Less => high = mid.saturating_sub(1),
                Ordering::Equal => return Ok(Span(start, end)),
                Ordering::Greater => low = mid + 1,
            }
        }

        Err(SearchError::NotFound(Span(start, end)))
    }

    /// Creates an instance of [`SortedString`] without performing sanity checks.
    ///
    /// This is essentially what conventionally would be a simple
    /// [`new()`](https://rust-lang.github.io/api-guidelines/naming.html#casing-conforms-to-rfc-430-c-case),
    /// but specifically named to alert users to the dangers.
    ///
    /// # Example: Simple Use
    ///
    /// ```
    /// use ascii::AsciiChar;
    /// use b4s::SortedString;
    ///
    /// let sep = AsciiChar::Comma;
    /// let haystack = "a,b,c";
    /// let sorted_string = SortedString::new_unchecked(haystack, sep);
    /// ```
    ///
    /// # Example: Incorrect Use
    ///
    /// ```
    /// use ascii::AsciiChar;
    /// use b4s::{SortedString, Span};
    /// use b4s::SearchError::NotFound;
    ///
    /// let sep = AsciiChar::Comma;
    /// let unsorted_haystack = "a,c,b";
    /// let sorted_string = SortedString::new_unchecked(unsorted_haystack, sep);
    ///
    /// // Unable to find element in unsorted haystack
    /// assert_eq!(sorted_string.binary_search("b"), Err(NotFound(Span(0, 1))));
    /// ```
    #[must_use]
    pub fn new_unchecked(string: &'a str, sep: AsciiChar) -> Self {
        Self::new(string, sep)
    }

    /// Convenience method to sort a [`str`] by a given separator, returning an owned
    /// version.
    ///
    /// As [`SortedString`] is designed to be thin and doesn't own its data (expect for
    /// the `sep`), this convenience method helps creating a sorted [`String`] in the
    /// required format.
    ///
    /// # Example
    ///
    /// ```
    /// use ascii::AsciiChar;
    /// use b4s::{SortedString, Span};
    ///
    /// let sep = AsciiChar::LineFeed;
    /// // Perhaps this was read directly from a file, where sorting is unreliable/unknown.
    /// let unsorted_haystack = "c\nb\na";
    /// let sorted_haystack = SortedString::sort(unsorted_haystack, sep);
    ///
    /// // Passes fine
    /// let sorted_string = SortedString::new_checked(&sorted_haystack, sep).unwrap();
    ///
    /// assert_eq!(sorted_string.binary_search("c"), Ok(Span(4, 5)));
    /// ```
    ///
    #[must_use]
    pub fn sort(string: &str, sep: AsciiChar) -> String {
        string
            .split(sep.as_char())
            .sorted()
            .collect::<Vec<&str>>()
            .join(&sep.to_string())
    }

    fn new(string: &'a str, sep: AsciiChar) -> Self {
        Self { string, sep }
    }

    fn is_sorted(&self) -> bool {
        self.string
            .split(self.sep.as_char())
            .tuple_windows()
            .all(|(a, b)| a <= b)
    }
}

/// Error that can occur when creating a [`SortedString`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortedStringCreationError {
    /// The passed haystack was not sorted.
    NotSorted,
    /// The passed haystack was empty.
    EmptyHaystack,
}

impl Error for SortedStringCreationError {}

impl Display for SortedStringCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSorted => write!(f, "The provided string is not sorted."),
            Self::EmptyHaystack => write!(f, "The provided string is empty."),
        }
    }
}

/// A span of `start` and `end` indices.
///
/// Indicating the start and end of a substring in a haystack, **in raw byte indices**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span(
    /// Start index of the substring.
    pub usize,
    /// End index of the substring.
    pub usize,
);
