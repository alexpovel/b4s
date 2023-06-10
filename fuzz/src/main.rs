use afl::fuzz;
use arbitrary::{Arbitrary, Unstructured};
use b4s::AsciiChar;
use std::{fmt::Display, fs::OpenOptions};

/// Input signature for fuzzing.
///
/// Seems very inefficient, but there's limited capabilities to prepare/structure input.
/// See https://rust-fuzz.github.io/book/afl/tutorial.html. Note also:
///
/// > For performance reasons, it is recommended that you use the native type &[u8] when
/// > possible.
///
/// (Source: https://docs.rs/afl/0.13.0/afl/macro.fuzz.html)
///
/// I decided against using raw bytes, instead opting for structure via `Arbitrary`.
/// More maintainable, readable etc.
///
/// Also of interest might be what Google has to say on the topic at
/// https://github.com/google/fuzzing/blob/41d7725e6a12ec2e0a89b68fafacb4bb5af8fac7/docs/split-inputs.md#examples
/// . Note the example with regex, which matches the use case here.
#[derive(Arbitrary, Debug)]
struct Input {
    #[arbitrary(with = unstructured_to_ascii_char)]
    separator: AsciiChar,
    needle: String,
    haystack: String,
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sep = AsciiChar::from_ascii(self.separator);
        write!(
            f,
            "separator: {:?}, needle: {:?}, haystack: {:?}",
            sep, self.needle, self.haystack
        )
    }
}

fn unstructured_to_ascii_char(u: &mut Unstructured) -> arbitrary::Result<AsciiChar> {
    if let Ok(b) = u.arbitrary::<u8>() {
        if let Ok(char) = AsciiChar::from_ascii(b) {
            return Ok(char);
        }
    }

    Err(arbitrary::Error::IncorrectFormat)
}

fn main() {
    let filename = "fuzz-inputs.txt";

    let mut _data_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .expect("Cannot input data file.");

    fuzz!(|data: Input| {
        // Chances of us passing `new_checked` are exceedingly slim, so go unchecked.
        // Don't care for matches here anyway, just want to see if we panic.
        let ss = b4s::SortedString::new_unchecked(&data.haystack, data.separator);

        let _ = ss.binary_search(&data.needle);

        // It's not easy to get insight into the input data unless a panic occurred, so
        // write it out. This costs c. 20% in throughput performance. Comment out once
        // you're convinced results are good.
        // _data_file
        //     .write_all(format!("{data}\n").as_bytes())
        //     .expect("Cannot write to data file.");
    });
}
