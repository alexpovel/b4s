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
#![doc = include_str!("../README.md")]

#[doc(no_inline)] // https://users.rust-lang.org/t/re-exporting-type-and-rustdoc/50847
pub use ascii::AsciiChar;
use itertools::Itertools;
use std::{cmp::Ordering, error::Error, fmt::Display, ops::Range};

/// Main type to perform binary search through.
///
/// This type upholds the important invariants of [`SortedString::binary_search()`]:
///
/// - a *sorted* string is required,
/// - as well as some [`AsciiChar`] separator to split by.
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
/// let sep = '🦀';
/// let haystack = "a,b,c";
/// // Not allowed:
/// let _ = b4s::SortedString::new_checked(haystack, sep);
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SortedString<'a> {
    string: &'a str,
    sep: AsciiChar,
}

impl Display for SortedString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SortedString({:?}, {:?})", self.string, self.sep)
    }
}

/// Error returned in case of a failed search.
///
/// [Thin wrapper](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
/// for [`Range<usize>`], required to implement [`Error`], achieving a [useful
/// API](https://rust-lang.github.io/api-guidelines/interoperability.html#error-types-are-meaningful-and-well-behaved-c-good-err).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchError(
    /// Location of the last *unsuccessful* comparison.
    ///
    /// This is *not* the location where the needle could be inserted. That location is
    /// either to the left *or* right of this location, depending on how
    /// [comparison](https://doc.rust-lang.org/std/primitive.str.html#impl-PartialOrd%3Cstr%3E-for-str)
    /// goes. As this error is not particularly actionable at runtime, the exact
    /// location (left, right) is not reported, saving computation at runtime and
    /// affording a simpler implementation.
    pub Range<usize>,
);

impl Error for SearchError {}

impl Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let SearchError(range) = self;
        write!(f, "needle not found, last looked at {range:?}")
    }
}

/// The result of a [`SortedString::binary_search()`].
///
///
/// ## Success case
///
/// The location where the match was found.
///
/// ## Error case
///
/// Refer to [`SearchError`] for details.
pub type SearchResult = Result<Range<usize>, SearchError>;

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
    /// let sep = b4s::AsciiChar::Comma;
    /// let haystack = "a,b,c";
    /// let sorted_string = b4s::SortedString::new_checked(haystack, sep);
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
    /// let sep = b4s::AsciiChar::Comma;
    /// let unsorted_haystack = "a,c,b";
    /// let sorted_string = b4s::SortedString::new_checked(unsorted_haystack, sep);
    ///
    /// assert_eq!(sorted_string, Err(b4s::SortedStringCreationError::NotSorted));
    /// ```
    ///
    /// This error might also creep in when passing a newline-separated file containing
    /// a trailing newline. The empty string will be in last position, contradicting
    /// sorting.
    ///
    /// ```
    /// let sep = b4s::AsciiChar::LineFeed;
    /// let haystack = "Aachen\nAmpel\nAngel\nApfel\n";
    /// let ss = b4s::SortedString::new_checked(haystack, sep);
    ///
    /// assert_eq!(ss, Err(b4s::SortedStringCreationError::NotSorted));
    /// ```
    ///
    /// ## Example: Creation Fails (Haystack Empty)
    ///
    /// Supplying an empty haystack [`str`] is likely an error and therefore fails.
    ///
    /// ```
    /// let sep = b4s::AsciiChar::Comma;
    /// let haystack = "";
    /// let sorted_string = b4s::SortedString::new_checked(haystack, sep);
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
    /// returning a [`Result`]. The success case is a [`std::ops::Range`], not a single
    /// [`usize`], as the haystack is variable-length. For the error case, see below.
    ///
    /// # Examples
    ///
    /// For common examples, see the [crate-level documentation](crate). The following
    /// are more specific examples.
    ///
    /// ## Multi-byte needle and haystack
    ///
    /// The needle and haystack can be *any* UTF-8 string. However, note that the
    /// returned [`Range`] is returned in raw byte positions, similar to
    /// [`std::str::len`](https://doc.rust-lang.org/std/primitive.str.html#method.len):
    ///
    /// > This length is in bytes, not chars or graphemes. In other words, it might not
    /// > be what a human considers the length of the string.
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let sep = b4s::AsciiChar::VerticalBar;
    /// let haystack = "Apfel|Bäume|Zäune|zäunen|Äpfel|Öfen|ƒoo|مرحبًا|你好|😂";
    /// let ss = b4s::SortedString::new_checked(haystack, sep)?;
    ///
    /// let needle = "Bäume";
    /// let result = ss.binary_search(needle);
    ///
    /// assert_eq!(result, Ok(std::ops::Range { start: 6, end: 12 }));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Needle contains separator
    ///
    /// This situation not handled specially. Such needles will be impossible to find:
    ///
    /// ```
    /// use std::{error::Error, ops::Range};
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let sep = b4s::AsciiChar::A;
    ///     let haystack = "Aachen\nAmpel\nAngel\nApfel\n";
    ///     let ss = b4s::SortedString::new_checked(haystack, sep)?;
    ///
    ///     let needle = "Angel";
    ///     let result = ss.binary_search(needle);
    ///     assert_eq!(result, Err(b4s::SearchError(Range { start: 0, end: 0 })));
    ///
    ///     let needle = "ngel\n";
    ///     let range = ss.binary_search(needle)?; // Works (note ? operator also works)
    ///     assert_eq!(range, Range { start: 14, end: 19 });
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Refer to [`SearchError`] for more info.
    ///
    /// # Panics
    ///
    /// This method panics when slicing into the `haystack` fails. Occurrences of such
    /// panics are [logic
    /// bugs](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html#cases-in-which-you-have-more-information-than-the-compiler)
    /// and should never occur (see the below assumptions). Please report them if they
    /// do.
    ///
    /// The panic allows the API to be simple. No panic implies converting the failure
    /// to some error value, like an `enum`. That `enum` would then have a variant such
    /// as `EncodingError`. However, the user would have no sensible way of handling
    /// that, either, and might as well panic anyway. Keeping the panic inside the
    /// function allows the return type to be very simple (no `enum` required).
    ///
    /// See also [this
    /// thread](https://web.archive.org/web/20230610095730/https://old.reddit.com/r/rust/comments/we33nd/when_is_unwrap_idiomatic/iinvqqh/?context=3),
    /// where people who know what they're talking about in terms of Rust
    /// ([matklad](https://github.com/matklad),
    /// [BurntSushi](https://github.com/BurntSushi)) argue in a similar fashion. The
    /// author [took their
    /// advice](https://en.wikipedia.org/wiki/Argument_from_authority) to gauge
    /// idiomaticity.
    ///
    /// ## Background
    ///
    /// Fundamental assumptions for panic-freedom are:
    ///
    /// - The haystack is `&str`, aka valid UTF-8. **Users cannot pass malformed
    ///   UTF-8.**
    /// - The separator is ASCII, as ensured by signatures working off [`AsciiChar`],
    ///   providing type-level guarantees. **Users can only pass single-byte UTF-8
    ///   values.**
    /// - The separator being ASCII ([highest bit
    ///   0](https://en.wikipedia.org/wiki/ASCII#8-bit_codes)), it cannot be part of any
    ///   code point encoded as [multiple bytes using UTF-8 (highest bit
    ///   1)](https://en.wikipedia.org/wiki/UTF-8#Encoding), as
    ///   [validated](https://doc.rust-lang.org/std/primitive.char.html#validity) in
    ///   this sanity check:
    ///
    ///      ```
    ///      for code_point in 0..=0x10FFFF {
    ///          if let Some(c) = std::char::from_u32(code_point) {
    ///              if c.is_ascii() {
    ///                  continue;
    ///              }
    ///              for byte in c.to_string().bytes() {
    ///                  assert!(!byte.is_ascii());
    ///              }
    ///          }
    ///      }
    ///      ```
    /// - *Single*-byte values with highest bit 1 are outside the ASCII range and
    ///   **invalid UTF-8**:
    ///
    ///   ```
    ///   for byte in 0x80u8..=0xFFu8 {
    ///       assert!(!byte.is_ascii());
    ///       assert!(std::str::from_utf8(&[byte]).is_err());
    ///   }
    ///   ```
    ///
    /// ### Fuzz Testing
    ///
    /// To further strengthen confidence in panic-freedom, fuzz testing was conducted
    /// using [`afl`](https://crates.io/crates/afl).
    ///
    /// If you have access to the crate's source code repository, run fuzz testing
    /// yourself from the root using
    ///
    /// ```bash
    /// cargo install just && just fuzz
    /// ```
    ///
    /// The author let a fuzz test run for over 5 billion iterations, finding no panics:
    ///
    /// ```text
    ///      american fuzzy lop ++4.06c {default} (target/debug/afl-target) [fast]
    /// ┌─ process timing ────────────────────────────────────┬─ overall results ────┐
    /// │        run time : 0 days, 19 hrs, 7 min, 6 sec      │  cycles done : 192k  │
    /// │   last new find : 0 days, 18 hrs, 56 min, 54 sec    │ corpus count : 39    │
    /// │last saved crash : none seen yet                     │saved crashes : 0     │
    /// │ last saved hang : none seen yet                     │  saved hangs : 0     │
    /// ├─ cycle progress ─────────────────────┬─ map coverage┴──────────────────────┤
    /// │  now processing : 31.444679 (79.5%)  │    map density : 3.31% / 6.22%      │
    /// │  runs timed out : 0 (0.00%)          │ count coverage : 2.77 bits/tuple    │
    /// ├─ stage progress ─────────────────────┼─ findings in depth ─────────────────┤
    /// │  now trying : splice 8               │ favored items : 12 (30.77%)         │
    /// │ stage execs : 27/28 (96.43%)         │  new edges on : 14 (35.90%)         │
    /// │ total execs : 5.15G                  │ total crashes : 0 (0 saved)         │
    /// │  exec speed : 73.5k/sec              │  total tmouts : 2 (0 saved)         │
    /// ├─ fuzzing strategy yields ────────────┴─────────────┬─ item geometry ───────┤
    /// │   bit flips : disabled (default, enable with -D)   │    levels : 6         │
    /// │  byte flips : disabled (default, enable with -D)   │   pending : 0         │
    /// │ arithmetics : disabled (default, enable with -D)   │  pend fav : 0         │
    /// │  known ints : disabled (default, enable with -D)   │ own finds : 33        │
    /// │  dictionary : n/a                                  │  imported : 0         │
    /// │havoc/splice : 31/1.98G, 2/3.17G                    │ stability : 100.00%   │
    /// │py/custom/rq : unused, unused, 0/1126, 0/19.4k      ├───────────────────────┘
    /// │    trim/eff : 1.58%/6355, disabled                 │          [cpu003: 12%]
    /// └────────────────────────────────────────────────────┘
    /// ```
    ///
    /// Note: at the time of writing,
    /// [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) and
    /// [`cargo-afl`](https://github.com/rust-fuzz/afl.rs) were available. The former
    /// currently only wraps [`libFuzzer`](https://llvm.org/docs/LibFuzzer.html), the
    /// latter works with [`afl`](https://github.com/google/AFL). **Both were already
    /// deprecated** at the time of writing, but continue to work well. `afl` was chosen
    /// as it didn't require a nightly toolchain.
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

            let haystack_word = std::str::from_utf8(&haystack[start..end])
                .expect("Indices aren't valid for slicing into haystack. They are at ASCII chars and therefore always assumed valid.");

            match needle.cmp(haystack_word) {
                Ordering::Less => high = mid.saturating_sub(1),
                Ordering::Equal => return Ok(Range { start, end }),
                Ordering::Greater => low = mid + 1,
            }
        }

        Err(SearchError(Range { start, end }))
    }

    /// Creates an instance of [`SortedString`] without performing sanity checks.
    ///
    /// This is essentially what conventionally would be a simple
    /// [`new()`](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits),
    /// but specifically named to alert users to the dangers.
    ///
    /// # Example: Simple Use
    ///
    /// ```
    /// let sep = b4s::AsciiChar::Comma;
    /// let haystack = "a,b,c";
    /// let sorted_string = b4s::SortedString::new_unchecked(haystack, sep);
    /// ```
    ///
    /// # Example: Incorrect Use
    ///
    /// ```
    /// use std::ops::Range;
    ///
    /// let sep = b4s::AsciiChar::Comma;
    /// let unsorted_haystack = "a,c,b";
    /// let sorted_string = b4s::SortedString::new_unchecked(unsorted_haystack, sep);
    ///
    /// // Unable to find element in unsorted haystack
    /// assert_eq!(
    ///     sorted_string.binary_search("b"),
    ///     Err(b4s::SearchError(Range { start: 0, end: 1 }))
    /// );
    /// ```
    #[must_use]
    pub const fn new_unchecked(string: &'a str, sep: AsciiChar) -> Self {
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let sep = b4s::AsciiChar::LineFeed;
    /// // Perhaps this was read directly from a file, where sorting is unreliable/unknown.
    /// let unsorted_haystack = "c\nb\na";
    /// let sorted_haystack = b4s::SortedString::sort(unsorted_haystack, sep);
    ///
    /// // Passes fine
    /// let sorted_string = b4s::SortedString::new_checked(&sorted_haystack, sep)?;
    ///
    /// assert_eq!(
    ///     sorted_string.binary_search("c"),
    ///     Ok(std::ops::Range { start: 4, end: 5 })
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn sort(string: &str, sep: AsciiChar) -> String {
        string
            .split(sep.as_char())
            .sorted()
            .collect::<Vec<&str>>()
            .join(&sep.to_string())
    }

    const fn new(string: &'a str, sep: AsciiChar) -> Self {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
