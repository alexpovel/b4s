use b4s::AsciiChar;
use b4s::SortedString;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use std::collections::HashSet;

fn generate_hashset(words: Vec<&str>) -> HashSet<&str> {
    HashSet::from_iter(words.iter().cloned())
}

/// Turns a vector of strings into a single string like:
/// `Foo␠␠␠␠␠␠Hello␠␠␠␠World␠␠␠␠Bar␠␠␠␠␠␠Automatic`
fn generate_padded_string_without_delimiter(
    words: Vec<&str>,
    padding: char,
    pad_to: usize,
) -> String {
    let mut out = String::new();

    for word in words {
        let padding = String::from(padding).repeat(pad_to - word.len());
        out.push_str(&format!("{}{}", word, padding));
    }

    out
}

/// Implements binary search over the output of `generate_padded_string_without_delimiter`.
fn binary_search_padded(word: &str, string: &str, block_size: usize) -> bool {
    let num_blocks = string.len() / block_size;

    let mut left = 0;
    let mut right = num_blocks;

    while left < right {
        let mid = left + (right - left) / 2;

        let start = mid * block_size;
        let end = start + block_size;

        let block = &string[start..end];
        let block_word = block.trim_end();

        match block_word.cmp(word) {
            std::cmp::Ordering::Equal => return true,
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }

    false
}

/// Compresses a list of `n` items to `m` items, preserving order and overall range. It
/// doesn't just take the first `m` items, nor does it simply step by `n/m`, as that
/// fraction might not be an integer.
fn compress_list<T, I>(items: I, target_size: usize) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    T: Clone,
{
    let items = items.into_iter().collect_vec();
    let n = items.len();
    if target_size >= n {
        return items;
    }

    let mut compressed = Vec::new();
    let step = n as f64 / target_size as f64;
    let mut index = 0.0;

    for _ in 0..target_size {
        compressed.push(items[index as usize].clone());
        index += step;
    }

    compressed
}

pub fn criterion_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");

    let short_run = true;
    if short_run {
        group.sample_size(10); // Default is 100
        group.warm_up_time(std::time::Duration::from_secs(1)); // Default is 3s
    }

    let words = include_str!("de-short.txt").split('\n').collect::<Vec<_>>();

    for length in &[
        10, 100, 1_000, 5_000, 10_000, 15_000, 20_000, 30_000, 50_000, 100_000, 200_000, 300_000,
        400_000, 500_000,
    ] {
        let words = compress_list(words.clone(), *length);

        // Write out the compressed list to a file, so we can use it in the tests.
        let wordss = words.join("\n");
        std::fs::write(format!("de-short-{}.txt", length), wordss).unwrap();

        let words_set = generate_hashset(words.clone());

        const DELIMITER: char = '\n';
        let words_single_string_with_delimiter = words.clone().join(&DELIMITER.to_string());
        let sorted_string = SortedString::new_checked(
            &words_single_string_with_delimiter,
            AsciiChar::from_ascii(DELIMITER).unwrap(),
        )
        .unwrap();

        let longest_word = words.iter().max_by_key(|w| w.len()).unwrap();
        let longest_word_length = longest_word.len();
        let shortest_word = words.iter().min_by_key(|w| w.len()).unwrap();

        const PADDING: char = ' ';
        let words_single_padded_string_without_delimiter =
            generate_padded_string_without_delimiter(words.clone(), PADDING, longest_word_length);

        let representative_words = vec![
            // Collect a couple words from different positions of the total array;
            // doesn't affect e.g. the hashset, but others.
            words[2],               // From beginning
            words[words.len() / 2], // From middle
            words[words.len() - 2], // From end
            longest_word,
            shortest_word,
        ];

        for repr_word in representative_words.iter() {
            let parameter_string = format!("'{repr_word}' within {length} entries");

            group.throughput(criterion::Throughput::Elements(words.len() as u64));

            group.bench_with_input(
                BenchmarkId::new("array", &parameter_string),
                repr_word,
                |b, i| b.iter(|| words.binary_search(black_box(i))),
            );

            group.bench_with_input(
                BenchmarkId::new("hashset", &parameter_string),
                repr_word,
                |b, i| b.iter(|| words_set.contains(black_box(i))),
            );

            group.bench_with_input(
                BenchmarkId::new("b4s", &parameter_string),
                repr_word,
                |b, i| b.iter(|| sorted_string.binary_search(black_box(i))),
            );

            group.bench_with_input(
                BenchmarkId::new("padded", &parameter_string),
                repr_word,
                |b, i| {
                    b.iter(|| {
                        binary_search_padded(
                            i,
                            &words_single_padded_string_without_delimiter,
                            longest_word_length,
                        )
                    })
                },
            );

            // `phf` not benchmarked as it's expected to be identical to stdlib's
            // `hashset`.

            let results = vec![
                words.binary_search(repr_word).is_ok(),
                words_set.contains(repr_word),
                sorted_string.binary_search(black_box(repr_word)).is_ok(),
                binary_search_padded(
                    repr_word,
                    &words_single_padded_string_without_delimiter,
                    longest_word_length,
                ),
            ];

            assert!(
                // Pretty useful assertion, as it's increasingly unlikely that all
                // benchmarks contain the same bug; the most likely case is one being
                // wrong, followed by two, etc., which will be caught.
                results.iter().all_equal(),
                "Mismatch in results: {:?}",
                results
            );
        }
    }

    group.finish();
}

criterion_group!(benches, criterion_bench);
criterion_main!(benches);
