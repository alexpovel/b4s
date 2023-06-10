<!-- markdownlint-disable MD013 -->

# b4s

Binary Search Single Sorted String: Perform binary search on a single, delimited string
slice of sorted but unevenly sized substrings.

## Usage

There are generally two ways to use this crate: at compile-time, or at runtime. The main
(only...) method of interest is [`SortedString::binary_search()`]. View its
documentation for detailed context.

### Runtime

```rust
use b4s::{AsciiChar, SortedString};

fn main() {
    match SortedString::new_checked("abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", AsciiChar::Comma) {
        Ok(ss) => {
            match ss.binary_search("ghi") {
                Ok(r) => println!("Found at range: {:?}", r),
                Err(r) => println!("Not found, last looked at range: {:?}", r),
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
```

### Compile-time

Only this function is `const fn`, and usable statically. As a tradeoff, it's potentially
unsound.

```rust
use b4s::{AsciiChar, SortedString};

static SS: SortedString =
    SortedString::new_unchecked("abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", AsciiChar::Comma);

fn main() {
    match SS.binary_search("ghi") {
        Ok(r) => println!("Found at range: {:?}", r),
        Err(r) => println!("Not found, last looked at range: {:?}", r),
    }
}
```

The source for the input string can be anything, for example a file prepared at compile
time:

```rust,ignore
static SS: SortedString =
    SortedString::new_unchecked(include_str!("path/to/file"), AsciiChar::LineFeed);
```

## Motivation

The itch to be scratched is the following:

- there's an array of strings to do lookup in, for example a word list
- the lookup is a simple containment check, with no modification
- the word list is available and prepared (sorted) at compile-time (e.g. using
  [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html))
- the word list is large (potentially much larger than the code itself)
- the list is to be distributed as part of the binary

A couple possible approaches come to mind. The summary table, where `n` is the number of
words, is:

| Approach      | Pre-compile preprocessing  | Compile time       | Runtime lookup          | Binary size |
| ------------- | -------------------------- | ------------------ | ----------------------- | ----------- |
| `b4s`         | Sort, `O(n log n)`         | Single ref: `O(1)` | Bin. search: `O(log n)` | `O(n)`      |
| `array`       | Sort, `O(n log n)`         | Many refs: `O(n)`  | Bin. search: `O(log n)` | `~ O(3n)`   |
| `phf`         | None                       | Many refs: `O(n)`  | Hash: `O(1)`            | `~ O(3n)`   |
| padded `&str` | Sort + Pad, `~ O(n log n)` | Single ref: `O(1)` | Bin. search: `O(log n)` | `~ O(n)`    |

For more context, see the individual sections below.

Note that the pre-compile preprocessing is ordinarily performed only **a single time**,
unless the word list itself changes. This column might therefore be moot, and considered
essentially zero-cost.

Therefore, this crate is an attempt to provide a solution with:

- good, not perfect runtime performance
- with very little, compile-time preprocessing needed (just sorting)
- essentially no additional startup cost (unlike, say, constructing a `HashSet` at
  runtime)
- binary sizes as small as possible
- compile times as fast as possible

It was found that the approaches using arrays and hash sets (via `phf`) absolutely
tanked developer experience, with compile times north of 20 minutes (!) for 30 MB word
lists, large binaries, and [`clippy`](https://github.com/rust-lang/rust-clippy)
imploding, taking the IDE with it. This crate was born as a solution. Its main downside
is **suboptimal runtime performance**. If that is your primary goal, opt for `phf` or
similar crates.

### Alternative approaches

#### Array

A simple array is an obvious choice, and can be generated in a build script.

```rust
static WORDS: &[&str] = &["abc", "def", "ghi", "jkl"];

fn main() {
    match WORDS.binary_search(&"ghi") {
        Ok(i) => println!("Found at index: {:?}", i),
        Err(i) => println!("Not found, could be inserted at: {:?}", i),
    }
}
```

There are two large pains in this approach:

1. compile times become very slow (in the rough ballpark of 1 minute per 100.000 words,
   YMMV considerably)
2. binary size becomes large.

   The words are *much* shorter than the `&str` they are contained in. On 64-bit
   hardware, [a `&str` is 16
   bytes](https://doc.rust-lang.org/std/primitive.str.html#representation), with a
   `usize` address pointer and [a `usize`
   length](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html). For large word
   lists, this leads to incredible bloat for the resulting binary.

#### Hash Set

Regular [`HashSet`s](https://doc.rust-lang.org/std/collections/struct.HashSet.html) are
not available at compile time. Crates like [`phf`](https://github.com/rust-phf/rust-phf)
change that:

```rust
use phf::{phf_set, Set};

static WORDS: Set<&'static str> = phf_set! {
    "abc",
    "def",
    "ghi",
    "jkl"
};

fn main() {
    if WORDS.contains(&"ghi") {
        println!("Found!");
    } else {
        println!("Not found!");
    }
}
```

Similar downsides as for the array case apply: very long compile times, and considerable
binary bloat from smart pointers. A hash set ultimately is an array with computed
indices, so this is expected.

#### Single, sorted and padded string

Another approach could be to use a single string (saving pointer bloat), but pad all
words to the longest occurring length, facilitating easy binary search (and increasing
bloat to some extent):

```rust
static WORDS: &str = "abc␣␣def␣␣ghi␣␣jklmn";

// Perform binary search...
```

The binary search implementation is then straightforward, as the elements are of known,
fixed lengths (in this case, 5). This approach was found to not perform well.
