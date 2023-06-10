use ascii::AsciiChar;
use b4s::{SearchResult, SortedString};
use rstest::rstest;
use std::ops::Range;

fn base_test(needle: &str, haystack: &str, sep: AsciiChar, expected: SearchResult) {
    let ss = SortedString::new_checked(haystack, sep).unwrap();
    assert_eq!(ss.binary_search(needle), expected);
}

#[rstest]
#[case(
    "abc",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 0, end: 3 })
)]
#[case(
    "def",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 4, end: 7 })
)]
#[case(
    "ghi",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 8, end: 11 })
)]
#[case(
    "jkl",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 12, end: 15 })
)]
#[case(
    "mno",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 16, end: 19 })
)]
#[case(
    "pqr",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 20, end: 23 })
)]
#[case(
    "stu",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 24, end: 27 })
)]
#[case(
    "vwx",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 28, end: 31 })
)]
#[case(
    "yz",
    "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz",
    AsciiChar::Comma,
    Ok(Range { start: 32, end: 34 })
)]
fn test_base_cases(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("mn", "abc,mno,yz", AsciiChar::Comma, Err(Range { start: 0, end: 3 }))]
#[case("a", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 0, end: 3 }))]
#[case("z", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 8, end: 10 }))]
fn test_needle_shorter_than_any_haystack_item(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abcd", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 0, end: 3 }))]
#[case("xyz", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 4, end: 7 }))]
#[case("zyz", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 8, end: 10 }))]
fn test_needle_longer_than_any_haystack_item(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc", "a,b,c", AsciiChar::Comma, Err(Range { start: 0, end: 1 }))]
fn test_single_character_haystack(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("a", "a,def,yz", AsciiChar::Comma, Ok(Range { start: 0, end: 1 }))]
#[case("a", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 0, end: 3 }))]
#[case("z", "abc,def,z", AsciiChar::Comma, Ok(Range { start: 8, end: 9 }))]
#[case("z", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 8, end: 10 }))]
fn test_single_character_needle(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("a", "a,b,c", AsciiChar::Comma, Ok(Range { start: 0, end: 1 }))]
#[case("c", "a,b,c", AsciiChar::Comma, Ok(Range { start: 4, end: 5 }))]
#[case("d", "a,b,c", AsciiChar::Comma, Err(Range { start: 4, end: 5 }))]
fn test_single_character_needle_and_haystack(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("aaa", "aaa,def,yz", AsciiChar::Comma, Ok(Range { start: 0, end: 3 }))]
#[case("aaa", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 0, end: 3 }))]
#[case("zzz", "abc,def,zzz", AsciiChar::Comma, Ok(Range { start: 8, end: 11 }))]
#[case("zzz", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 8, end: 10 }))]
fn test_repeated_character_needle(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("", ",", AsciiChar::Comma, Ok(Range { start: 0, end: 0 }))]
#[case("", ",,", AsciiChar::Comma, Ok(Range { start: 1, end: 1 }))]
#[case("", "abc", AsciiChar::Comma, Err(Range { start: 0, end: 3 }))]
fn test_empty_needle(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc", "abc", AsciiChar::Comma, Ok(Range { start: 0, end: 3 }))]
#[case("abc", "abc,def", AsciiChar::Comma, Ok(Range { start: 0, end: 3 }))]
#[case("abc", ",", AsciiChar::Comma, Err(Range { start: 0, end: 0 }))]
#[case("", ",,,,", AsciiChar::Comma, Ok(Range { start: 2, end: 2 }))]
#[case("abc", ",,,abc", AsciiChar::Comma, Ok(Range { start: 3, end: 6 }))]
fn test_oddly_shaped_haystack(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("nmo", "abc,mno,yz", AsciiChar::Comma, Err(Range { start: 4, end: 7 }))]
#[case("cba", "abc,def,yz", AsciiChar::Comma, Err(Range { start: 0, end: 3 }))]
fn test_switched_characters_needle(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc", "abc-def-yz", AsciiChar::Minus, Ok(Range { start: 0, end: 3 }))]
#[case("abc", "abc\0def\0yz", AsciiChar::Null, Ok(Range { start: 0, end: 3 }))]
#[case("defg", "abc\0def\0yz", AsciiChar::Null, Err(Range { start: 4, end: 7 }))]
fn test_different_separators(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc", "Hund\nKatze\nMaus", AsciiChar::LineFeed, Err(Range { start: 11, end: 15 }))]
#[case("ABC", "Hund\nKatze\nMaus", AsciiChar::LineFeed, Err(Range { start: 0, end: 4 }))]
#[case("Hund", "Hund\nKatze\nMaus", AsciiChar::LineFeed, Ok(Range { start: 0, end: 4 }))]
#[case("Katze", "Hund\nKatze\nMaus", AsciiChar::LineFeed, Ok(Range { start: 5, end: 10 }))]
#[case("Maus", "Hund\nKatze\nMaus", AsciiChar::LineFeed, Ok(Range { start: 11, end: 15 }))]
fn test_real_world_examples(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("Hündin", "Hund\nKatze\nMaus", AsciiChar::LineFeed, Err(Range { start: 5, end: 10 }))]
#[case("Hündin", "Hündin\nKatze\nMaus", AsciiChar::LineFeed, Ok(Range { start: 0, end: 7 }))]
#[case(
    "Mäuschen",
    "Hündin\nKatze\nMäuschen",
    AsciiChar::LineFeed,
    Ok(Range { start: 14, end: 23 })
)]
fn test_real_world_examples_with_multibyte_characters(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case(
    "Abdämpfung",
    "Abdämpfung\nAbenteuer\nAbschluss",
    AsciiChar::LineFeed,
    Ok(Range { start: 0, end: 11 })
)]
#[case(
    "Abdämpfung",
    "Abdrehen\nAbdämpfung\nAbschluss",
    AsciiChar::LineFeed,
    Ok(Range { start: 9, end: 20 })
)]
fn test_real_world_examples_with_common_prefixes(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc,def", "abc,def,ghi", AsciiChar::Comma, Err(Range { start: 0, end: 3 }))]
fn test_needle_contains_separator(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("", ",a", AsciiChar::Comma, Ok(Range { start: 0, end: 0 }))]
#[should_panic]
fn test_buggy_cases_to_be_fixed(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: AsciiChar,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("c,b,a", AsciiChar::Comma)]
#[case("a,", AsciiChar::Comma)]
#[case("a,b,c,", AsciiChar::Comma)]
#[case("a,,", AsciiChar::Comma)]
#[case(",a,", AsciiChar::Comma)]
#[case("Hello,Cruel,World", AsciiChar::Comma)]
fn test_unsorted_haystack(#[case] haystack: &str, #[case] sep: AsciiChar) {
    let ss = SortedString::new_checked(haystack, sep);
    assert_eq!(ss, Err(b4s::SortedStringCreationError::NotSorted));
}

#[rstest]
#[case("", AsciiChar::Comma)]
#[case("", AsciiChar::Minus)]
#[case("", AsciiChar::Null)]
fn test_empty_haystack(#[case] haystack: &str, #[case] sep: AsciiChar) {
    let ss = SortedString::new_checked(haystack, sep);
    assert_eq!(ss, Err(b4s::SortedStringCreationError::EmptyHaystack));
}
