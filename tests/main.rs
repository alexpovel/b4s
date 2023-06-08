use b4s::{SearchError, SearchResult, SortedString, Span};
use rstest::rstest;

fn base_test(needle: &str, haystack: &str, sep: char, expected: SearchResult) {
    let ss = SortedString::new_checked(haystack, sep).unwrap();
    assert_eq!(ss.binary_search(needle), expected);
}

#[rstest]
#[case("abc", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(0, 3)))]
#[case("def", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(4, 7)))]
#[case("ghi", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(8, 11)))]
#[case("jkl", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(12, 15)))]
#[case("mno", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(16, 19)))]
#[case("pqr", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(20, 23)))]
#[case("stu", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(24, 27)))]
#[case("vwx", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(28, 31)))]
#[case("yz", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', Ok(Span(32, 34)))]
fn test_base_cases(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("mn", "abc,mno,yz", ',', Err(SearchError::NotFound(Span(0, 3))))]
#[case("a", "abc,def,yz", ',', Err(SearchError::NotFound(Span(0, 3))))]
#[case("z", "abc,def,yz", ',', Err(SearchError::NotFound(Span(8, 10))))]
fn test_needle_shorter_than_any_haystack_item(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abcd", "abc,def,yz", ',', Err(SearchError::NotFound(Span(0, 3))))]
#[case("xyz", "abc,def,yz", ',', Err(SearchError::NotFound(Span(4, 7))))]
#[case("zyz", "abc,def,yz", ',', Err(SearchError::NotFound(Span(8, 10))))]
fn test_needle_longer_than_any_haystack_item(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc", "a,b,c", ',', Err(SearchError::NotFound(Span(0, 1))))]
fn test_single_character_haystack(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("a", "a,def,yz", ',', Ok(Span(0, 1)))]
#[case("a", "abc,def,yz", ',', Err(SearchError::NotFound(Span(0, 3))))]
#[case("z", "abc,def,z", ',', Ok(Span(8, 9)))]
#[case("z", "abc,def,yz", ',', Err(SearchError::NotFound(Span(8, 10))))]
fn test_single_character_needle(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("a", "a,b,c", ',', Ok(Span(0, 1)))]
#[case("c", "a,b,c", ',', Ok(Span(4, 5)))]
#[case("d", "a,b,c", ',', Err(SearchError::NotFound(Span(4, 5))))]
fn test_single_character_needle_and_haystack(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("aaa", "aaa,def,yz", ',', Ok(Span(0, 3)))]
#[case("aaa", "abc,def,yz", ',', Err(SearchError::NotFound(Span(0, 3))))]
#[case("zzz", "abc,def,zzz", ',', Ok(Span(8, 11)))]
#[case("zzz", "abc,def,yz", ',', Err(SearchError::NotFound(Span(8, 10))))]
fn test_repeated_character_needle(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("", ",", ',', Ok(Span(0, 0)))]
#[case("", ",,", ',', Ok(Span(1, 1)))]
#[case("", "abc", ',', Err(SearchError::NotFound(Span(0, 3))))]
fn test_empty_needle(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc", "abc", ',', Ok(Span(0, 3)))]
#[case("abc", "abc,def", ',', Ok(Span(0, 3)))]
#[case("abc", ",", ',', Err(SearchError::NotFound(Span(0, 0))))]
#[case("", ",,,,", ',', Ok(Span(2, 2)))]
#[case("abc", ",,,abc", ',', Ok(Span(3, 6)))]
fn test_oddly_shaped_haystack(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("nmo", "abc,mno,yz", ',', Err(SearchError::NotFound(Span(4, 7))))]
#[case("cba", "abc,def,yz", ',', Err(SearchError::NotFound(Span(0, 3))))]
fn test_switched_characters_needle(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc", "abc-def-yz", '-', Ok(Span(0, 3)))]
#[case("abc", "abc\0def\0yz", '\0', Ok(Span(0, 3)))]
#[case("defg", "abc\0def\0yz", '\0', Err(SearchError::NotFound(Span(4, 7))))]
fn test_different_separators(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case(
    "abc",
    "Hund\nKatze\nMaus",
    '\n',
    Err(SearchError::NotFound(Span(11, 15)))
)]
#[case(
    "ABC",
    "Hund\nKatze\nMaus",
    '\n',
    Err(SearchError::NotFound(Span(0, 4)))
)]
#[case("Hund", "Hund\nKatze\nMaus", '\n', Ok(Span(0, 4)))]
#[case("Katze", "Hund\nKatze\nMaus", '\n', Ok(Span(5, 10)))]
#[case("Maus", "Hund\nKatze\nMaus", '\n', Ok(Span(11, 15)))]
fn test_real_world_examples(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case(
    "Hündin",
    "Hund\nKatze\nMaus",
    '\n',
    Err(SearchError::NotFound(Span(5, 10)))
)]
#[case("Hündin", "Hündin\nKatze\nMaus", '\n', Ok(Span(0, 7)))]
#[case("Mäuschen", "Hündin\nKatze\nMäuschen", '\n', Ok(Span(14, 23)))]
fn test_real_world_examples_with_multibyte_characters(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case(
    "Abdämpfung",
    "Abdämpfung\nAbenteuer\nAbschluss",
    '\n',
    Ok(Span(0, 11))
)]
#[case("Abdämpfung", "Abdrehen\nAbdämpfung\nAbschluss", '\n', Ok(Span(9, 20)))]
fn test_real_world_examples_with_common_prefixes(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("abc,def", "abc,def,ghi", ',', Err(SearchError::NotFound(Span(0, 3))))]
fn test_needle_contains_separator(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: SearchResult,
) {
    base_test(needle, haystack, sep, expected)
}

#[rstest]
#[case("", ",a", ',', true)]
#[should_panic]
fn test_buggy_cases_to_be_fixed(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: bool,
) {
    let ss = SortedString::new_checked(haystack, sep).unwrap();
    assert_eq!(ss.binary_search(needle).is_ok(), expected);
}

#[rstest]
#[case("c,b,a", ',')]
#[case("a,", ',')]
#[case("a,b,c,", ',')]
#[case("a,,", ',')]
#[case(",a,", ',')]
#[case("Hello,Cruel,World", ',')]
fn test_unsorted_haystack(#[case] haystack: &str, #[case] sep: char) {
    let ss = SortedString::new_checked(haystack, sep);
    assert_eq!(ss, Err(b4s::SortedStringCreationError::NotSorted));
}

#[rstest]
#[case("", ',')]
#[case("", '-')]
#[case("", '\0')]
fn test_empty_haystack(#[case] haystack: &str, #[case] sep: char) {
    let ss = SortedString::new_checked(haystack, sep);
    assert_eq!(ss, Err(b4s::SortedStringCreationError::EmptyHaystack));
}
