use b4s::SortedString;
use rstest::rstest;

#[rstest]
// Base cases, all elements present in any position.
#[case("abc", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
#[case("def", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
#[case("ghi", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
#[case("jkl", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
#[case("mno", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
#[case("pqr", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
#[case("stu", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
#[case("vwx", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
#[case("yz", "abc,def,ghi,jkl,mno,pqr,stu,vwx,yz", ',', true)]
// Shorter needle than any haystack item.
#[case("mn", "abc,mno,yz", ',', false)]
#[case("a", "abc,def,yz", ',', false)]
#[case("z", "abc,def,yz", ',', false)]
// Longer needle than any haystack item.
#[case("abcd", "abc,def,yz", ',', false)]
#[case("xyz", "abc,def,yz", ',', false)]
#[case("xyz", "abc,def,yz", ',', false)]
// Single-character haystack.
#[case("abc", "a,b,c", ',', false)]
// Single-character needle and haystack.
#[case("a", "a,b,c", ',', true)]
#[case("c", "a,b,c", ',', true)]
#[case("d", "a,b,c", ',', false)]
// Single-character needle.
#[case("a", "a,def,yz", ',', true)]
#[case("a", "abc,def,yz", ',', false)]
#[case("z", "abc,def,z", ',', true)]
#[case("z", "abc,def,yz", ',', false)]
// Repeated-character needle.
#[case("aaa", "aaa,def,yz", ',', true)]
#[case("aaa", "abc,def,yz", ',', false)]
#[case("zzz", "abc,def,zzz", ',', true)]
#[case("zzz", "abc,def,yz", ',', false)]
// Empty needle
#[case("", ",", ',', true)]
#[case("", ",,", ',', true)]
#[case("", "abc", ',', false)]
// Oddly-shaped haystack
#[case("abc", "abc", ',', true)]
#[case("abc", "abc,def", ',', true)]
#[case("abc", ",", ',', false)]
#[case("", ",", ',', true)]
#[case("", ",,,,", ',', true)]
#[case("abc", ",,,abc", ',', true)]
// Switched characters.
#[case("nmo", "abc,mno,yz", ',', false)]
#[case("cba", "abc,def,yz", ',', false)]
// Different separators.
#[case("abc", "abc-def-yz", '-', true)]
#[case("abc", "abc\0def\0yz", '\0', true)]
// Real-world examples.
#[case("abc", "Hund\nKatze\nMaus", '\n', false)]
#[case("Hund", "Hund\nKatze\nMaus", '\n', true)]
#[case("Katze", "Hund\nKatze\nMaus", '\n', true)]
#[case("Maus", "Hund\nKatze\nMaus", '\n', true)]
// Real-world examples with multi-byte (UTF-8) characters.
#[case("Hündin", "Hund\nKatze\nMaus", '\n', false)]
#[case("Hündin", "Hündin\nKatze\nMaus", '\n', true)]
#[case("Mäuschen", "Hündin\nKatze\nMäuschen", '\n', true)]
// Real-world examples with common prefixes.
#[case("Abdämpfung", "Abdämpfung\nAbenteuer\nAbschluss", '\n', true)]
#[case("Abdämpfung", "Abdrehen\nAbdämpfung\nAbschluss", '\n', true)]
// Needle contains separator.
#[case("abc,def", "abc,def,ghi", ',', false)]
fn test_binary_search(
    #[case] needle: &str,
    #[case] haystack: &str,
    #[case] sep: char,
    #[case] expected: bool,
) {
    let ss = SortedString::new_checked(haystack, sep).unwrap();
    assert_eq!(ss.binary_search(needle).is_ok(), expected);
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
#[case("", ',')]
#[case("", '-')]
#[case("", '\0')]
fn test_empty_haystack(#[case] haystack: &str, #[case] sep: char) {
    let ss = SortedString::new_checked(haystack, sep);
    assert_eq!(ss, Err(b4s::SortedStringCreationError::EmptyHaystack));
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
