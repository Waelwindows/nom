//! parsers recognizing bytes streams, streaming version

use error::ErrorKind;
use error::ParseError;
use internal::{Err, IResult, Needed};
use lib::std::ops::RangeFrom;
use lib::std::result::Result::*;
use traits::{Compare, CompareResult, FindSubstring, FindToken, InputIter, InputLength, InputTake, InputTakeAtPosition, Slice, ToUsize};

/// Recognizes a pattern
///
/// The input data will be compared to the tag combinator's argument and will return the part of
/// the input that matches the argument
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::tag;
///
/// fn parser(s: &str) -> IResult<&str, &str> {
///   tag("Hello")(s)
/// }
///
/// assert_eq!(parser("Hello, World!"), Ok((", World!", "Hello")));
/// assert_eq!(parser("Something"), Err(Err::Error(("Something", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(Err::Incomplete(Needed::Size(5))));
/// ```
pub fn tag<'a, T: 'a, Input: 'a, Error: ParseError<Input>>(tag: T) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTake + Compare<T>,
  T: InputLength + Clone,
{
  move |i: Input| {
    let tag_len = tag.input_len();
    let t = tag.clone();

    let res: IResult<_, _, Error> = match i.compare(t) {
      CompareResult::Ok => Ok(i.take_split(tag_len)),
      CompareResult::Incomplete => Err(Err::Incomplete(Needed::Size(tag_len))),
      CompareResult::Error => {
        let e: ErrorKind = ErrorKind::Tag;
        Err(Err::Error(Error::from_error_kind(i, e)))
      }
    };
    res
  }
}

/// Recognizes a case insensitive pattern
///
/// The input data will be compared to the tag combinator's argument and will return the part of
/// the input that matches the argument with no regard to case
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::tag_no_case;
///
/// fn parser(s: &str) -> IResult<&str, &str> {
///   tag_no_case("hello")(s)
/// }
///
/// assert_eq!(parser("Hello, World!"), Ok((", World!", "Hello")));
/// assert_eq!(parser("hello, World!"), Ok((", World!", "hello")));
/// assert_eq!(parser("HeLlO, World!"), Ok((", World!", "HeLlO")));
/// assert_eq!(parser("Something"), Err(Err::Error(("Something", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(Err::Incomplete(Needed::Size(5))));
/// ```
pub fn tag_no_case<T, Input, Error: ParseError<Input>>(tag: T) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTake + Compare<T>,
  T: InputLength + Clone,
{
  move |i: Input| {
    let tag_len = tag.input_len();
    let t = tag.clone();

    let res: IResult<_, _, Error> = match (i).compare_no_case(t) {
      CompareResult::Ok => Ok(i.take_split(tag_len)),
      CompareResult::Incomplete => Err(Err::Incomplete(Needed::Size(tag_len))),
      CompareResult::Error => {
        let e: ErrorKind = ErrorKind::Tag;
        Err(Err::Error(Error::from_error_kind(i, e)))
      }
    };
    res
  }
}

/// Parse till certain characters are met
///
/// The parser will return the longest slice till one of the characters of the combinator's argument are met.
///
/// It doesn't consume the matched character,
///
/// It will return a `Err::Incomplete(Needed::Size(1))` if the pattern wasn't met
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::is_not;
///
/// fn not_space(s: &str) -> IResult<&str, &str> {
///   is_not(" \t\r\n")(s)
/// }
///
/// assert_eq!(not_space("Hello, World!"), Ok((" World!", "Hello,")));
/// assert_eq!(not_space("Sometimes\t"), Ok(("\t", "Sometimes")));
/// assert_eq!(not_space("Nospace"), Err(Err::Incomplete(Needed::Size(1))));
/// assert_eq!(not_space(""), Err(Err::Incomplete(Needed::Size(1))));
/// ```
pub fn is_not<T, Input, Error: ParseError<Input>>(arr: T) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTakeAtPosition,
  T: InputLength + FindToken<<Input as InputTakeAtPosition>::Item>,
{
  move |i: Input| {
    let e: ErrorKind = ErrorKind::IsNot;
    i.split_at_position1(|c| arr.find_token(c), e)
  }
}

/// Returns the longest slice of the matches the pattern
///
/// The parser will return the longest slice consisting of the characters in provided in the
/// combinator's argument
///
/// # Streaming specific
/// *Streaming version* will return a `Err::Incomplete(Needed::Size(1))` if the pattern wasn't met
/// or if the pattern reaches the end of the input
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::is_a;
///
/// fn hex(s: &str) -> IResult<&str, &str> {
///   is_a("1234567890ABCDEF")(s)
/// }
///
/// assert_eq!(hex("123 and voila"), Ok((" and voila", "123")));
/// assert_eq!(hex("DEADBEEF and others"), Ok((" and others", "DEADBEEF")));
/// assert_eq!(hex("BADBABEsomething"), Ok(("something", "BADBABE")));
/// assert_eq!(hex("D15EA5E"), Err(Err::Incomplete(Needed::Size(1))));
/// assert_eq!(hex(""), Err(Err::Incomplete(Needed::Size(1))));
/// ```
pub fn is_a<T, Input, Error: ParseError<Input>>(arr: T) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTakeAtPosition,
  T: InputLength + FindToken<<Input as InputTakeAtPosition>::Item>,
{
  move |i: Input| {
    let e: ErrorKind = ErrorKind::IsA;
    i.split_at_position1(|c| !arr.find_token(c), e)
  }
}

/// Returns the longest input slice (if any) that matches the predicate
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::Size(1))` if the pattern reaches the end of the input
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_while;
/// use nom::character::is_alphabetic;
///
/// fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while(is_alphabetic)(s)
/// }
///
/// assert_eq!(alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(alpha(b"12345"), Ok((&b"12345"[..], &b""[..])));
/// assert_eq!(alpha(b"latin"), Err(Err::Incomplete(Needed::Size(1))));
/// assert_eq!(alpha(b""), Err(Err::Incomplete(Needed::Size(1))));
/// ```
pub fn take_while<F, Input, Error: ParseError<Input>>(cond: F) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTakeAtPosition,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
{
  move |i: Input| i.split_at_position(|c| !cond(c))
}

/// Returns the longest (atleast 1) input slice that matches the predicate
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*
///
/// It will return an `Err(Err::Error((_, ErrorKind::TakeWhile1)))` if the pattern wasn't met
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::Size(1))` or if the pattern reaches the end of the input.
///
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_while1;
/// use nom::character::is_alphabetic;
///
/// fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while1(is_alphabetic)(s)
/// }
///
/// assert_eq!(alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(alpha(b"latin"), Err(Err::Incomplete(Needed::Size(1))));
/// assert_eq!(alpha(b"12345"), Err(Err::Error((&b"12345"[..], ErrorKind::TakeWhile1))));
/// ```
pub fn take_while1<F, Input, Error: ParseError<Input>>(cond: F) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTakeAtPosition,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
{
  move |i: Input| {
    let e: ErrorKind = ErrorKind::TakeWhile1;
    i.split_at_position1(|c| !cond(c), e)
  }
}

/// Returns the longest (m <= len <= n) input slice  that matches the predicate
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*
///
/// It will return an `Err::Error((_, ErrorKind::TakeWhileMN))` if the pattern wasn't met
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::Size(1))`  if the pattern reaches the end of the input or is too short.
///
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_while_m_n;
/// use nom::character::is_alphabetic;
///
/// fn short_alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while_m_n(3, 6, is_alphabetic)(s)
/// }
///
/// assert_eq!(short_alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(short_alpha(b"lengthy"), Ok((&b"y"[..], &b"length"[..])));
/// assert_eq!(short_alpha(b"latin"), Err(Err::Incomplete(Needed::Size(1))));
/// assert_eq!(short_alpha(b"ed"), Err(Err::Incomplete(Needed::Size(1))));
/// assert_eq!(short_alpha(b"12345"), Err(Err::Error((&b"12345"[..], ErrorKind::TakeWhileMN))));
/// ```
pub fn take_while_m_n<F, Input, Error: ParseError<Input>>(m: usize, n: usize, cond: F) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTake + InputIter + InputLength + Slice<RangeFrom<usize>>,
  F: Fn(<Input as InputIter>::RawItem) -> bool,
{
  move |i: Input| {
    let input = i;

    match input.position(|c| !cond(c)) {
      Some(idx) => {
        if idx >= m {
          if idx <= n {
            let res: IResult<_, _, Error> = if let Some(index) = input.slice_index(idx) {
              Ok(input.take_split(index))
            } else {
              Err(Err::Error(Error::from_error_kind(input, ErrorKind::TakeWhileMN)))
            };
            res
          } else {
            let res: IResult<_, _, Error> = if let Some(index) = input.slice_index(n) {
              Ok(input.take_split(index))
            } else {
              Err(Err::Error(Error::from_error_kind(input, ErrorKind::TakeWhileMN)))
            };
            res
          }
        } else {
          let e = ErrorKind::TakeWhileMN;
          Err(Err::Error(Error::from_error_kind(input, e)))
        }
      }
      None => {
        let len = input.input_len();
        if len >= n {
          let res: IResult<_, _, Error> = Ok(input.take_split(n));
          res
        } else {
          let needed = if m > len { m - len } else { 1 };
          Err(Err::Incomplete(Needed::Size(needed)))
        }
      }
    }
  }
}

/// Returns the longest input slice (if any) till a predicate is met
///
/// The parser will return the longest slice till the given predicate *(a function that
/// takes the input and returns a bool)*
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::Size(1))` if the match reaches the
/// end of input or if there was not match
///
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_till;
///
/// fn till_colon(s: &str) -> IResult<&str, &str> {
///   take_till(|c| c == ':')(s)
/// }
///
/// assert_eq!(till_colon("latin:123"), Ok((":123", "latin")));
/// assert_eq!(till_colon(":empty matched"), Ok((":empty matched", ""))); //allowed
/// assert_eq!(till_colon("12345"), Err(Err::Incomplete(Needed::Size(1))));
/// assert_eq!(till_colon(""), Err(Err::Incomplete(Needed::Size(1))));
/// ```
pub fn take_till<F, Input, Error: ParseError<Input>>(cond: F) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTakeAtPosition,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
{
  move |i: Input| i.split_at_position(|c| cond(c))
}

/// Returns the longest (atleast 1) input slice till a predicate is met
///
/// The parser will return the longest slice till the given predicate *(a function that
/// takes the input and returns a bool)*
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::Size(1))` if the match reaches the
/// end of input or if there was not match
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_till1;
///
/// fn till_colon(s: &str) -> IResult<&str, &str> {
///   take_till1(|c| c == ':')(s)
/// }
///
/// assert_eq!(till_colon("latin:123"), Ok((":123", "latin")));
/// assert_eq!(till_colon(":empty matched"), Err(Err::Error((":empty matched", ErrorKind::TakeTill1))));
/// assert_eq!(till_colon("12345"), Err(Err::Incomplete(Needed::Size(1))));
/// assert_eq!(till_colon(""), Err(Err::Incomplete(Needed::Size(1))));
/// ```
pub fn take_till1<F, Input, Error: ParseError<Input>>(cond: F) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTakeAtPosition,
  F: Fn(<Input as InputTakeAtPosition>::Item) -> bool,
{
  move |i: Input| {
    let e: ErrorKind = ErrorKind::TakeTill1;
    i.split_at_position1(|c| cond(c), e)
  }
}

/// Returns an input slice containing the first N input elements (Input[..N])
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::Size(N))` where N is the
/// argument if the input is less than the length provided
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take;
///
/// fn take6(s: &str) -> IResult<&str, &str> {
///   take(6usize)(s)
/// }
///
/// assert_eq!(take6("1234567"), Ok(("7", "123456")));
/// assert_eq!(take6("things"), Ok(("", "things")));
/// assert_eq!(take6("short"), Err(Err::Incomplete(Needed::Size(6)))); //N doesn't change
/// assert_eq!(take6(""), Err(Err::Incomplete(Needed::Size(6))));
/// ```
pub fn take<C, Input, Error: ParseError<Input>>(count: C) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputIter + InputTake,
  C: ToUsize,
{
  let c = count.to_usize();
  move |i: Input| match i.slice_index(c) {
    None => Err(Err::Incomplete(Needed::Size(c))),
    Some(index) => Ok(i.take_split(index)),
  }
}

/// Returns the longest input slice till it matches the pattern.
///
/// It doesn't consume the pattern
///
/// # Streaming Specific
/// *Streaming version* will return a `Err::Incomplete(Needed::Size(N))` if the input doesn't
/// contain the pattern or if the input is smaller than the pattern
/// # Example
/// ```rust
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// use nom::bytes::streaming::take_until;
///
/// fn until_eof(s: &str) -> IResult<&str, &str> {
///   take_until("eof")(s)
/// }
///
/// assert_eq!(until_eof("hello, worldeof"), Ok(("eof", "hello, world")));
/// assert_eq!(until_eof("hello, world"), Err(Err::Incomplete(Needed::Size(3))));
/// assert_eq!(until_eof(""), Err(Err::Incomplete(Needed::Size(3))));
/// ```
pub fn take_until<T, Input, Error: ParseError<Input>>(tag: T) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: InputTake + FindSubstring<T>,
  T: InputLength + Clone,
{
  move |i: Input| {
    let len = tag.input_len();
    let t = tag.clone();

    let res: IResult<_, _, Error> = match i.find_substring(t) {
      None => Err(Err::Incomplete(Needed::Size(len))),
      Some(index) => Ok(i.take_split(index)),
    };
    res
  }
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not accept the control character),
/// * the second argument is the control character (like `\` in most languages),
/// * the third argument matches the escaped characters
///
/// # Example
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::{Err, error::ErrorKind, Needed, IResult};
/// # use nom::character::complete::digit1;
/// use nom::bytes::streaming::escaped;
/// use nom::character::streaming::one_of;
///
/// fn esc(s: &str) -> IResult<&str, &str> {
///   escaped(digit1, '\\', one_of("\"n\\"))(s)
/// }
///
/// assert_eq!(esc("123;"), Ok((";", "123")));
/// assert_eq!(esc("12\\\"34;"), Ok((";", "12\\\"34")));
/// ```
///
pub fn escaped<Input, Error, F, G, O1, O2>(normal: F, control_char: char, escapable: G) -> impl Fn(Input) -> IResult<Input, Input, Error>
where
  Input: Clone + ::traits::Offset + InputLength + InputTake + InputTakeAtPosition + Slice<RangeFrom<usize>> + InputIter,
  <Input as InputIter>::Item: ::traits::AsChar,
  F: Fn(Input) -> IResult<Input, O1, Error>,
  G: Fn(Input) -> IResult<Input, O2, Error>,
  Error: ParseError<Input>,
{
  use traits::AsChar;

  move |input: Input| {
    let mut i = input.clone();

    while i.input_len() > 0 {
      match normal(i.clone()) {
        Ok((i2, _)) => {
          if i2.input_len() == 0 {
            return Err(Err::Incomplete(Needed::Unknown));
          } else {
            i = i2;
          }
        }
        Err(Err::Error(_)) => {
          // unwrap() should be safe here since index < $i.input_len()
          if i.iter_elements().next().unwrap().as_char() == control_char {
            let next = control_char.len_utf8();
            if next >= i.input_len() {
              return Err(Err::Incomplete(Needed::Size(1)));
            } else {
              match escapable(i.slice(next..)) {
                Ok((i2, _)) => {
                  if i2.input_len() == 0 {
                    return Err(Err::Incomplete(Needed::Unknown));
                  } else {
                    i = i2;
                  }
                }
                Err(e) => return Err(e),
              }
            }
          } else {
            let index = input.offset(&i);
            return Ok(input.take_split(index));
          }
        }
        Err(e) => {
          return Err(e);
        }
      }
    }

    Err(Err::Incomplete(Needed::Unknown))
  }
}

#[doc(hidden)]
pub fn escapedc<Input, Error, F, G, O1, O2>(i: Input, normal: F, control_char: char, escapable: G) -> IResult<Input, Input, Error>
where
  Input: Clone + ::traits::Offset + InputLength + InputTake + InputTakeAtPosition + Slice<RangeFrom<usize>> + InputIter,
  <Input as InputIter>::Item: ::traits::AsChar,
  F: Fn(Input) -> IResult<Input, O1, Error>,
  G: Fn(Input) -> IResult<Input, O2, Error>,
  Error: ParseError<Input>,
{
  escaped(normal, control_char, escapable)(i)
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not match the control character),
/// * the second argument is the control character (like `\` in most languages),
/// * the third argument matches the escaped characters and transforms them.
///
/// As an example, the chain `abc\tdef` could be `abc    def` (it also consumes the control character)
///
/// # Example
/// Currently Unavailable
pub fn escaped_transform<Input, Error, F, G, O1, O2, ExtendItem, Output>(
  normal: F,
  control_char: char,
  transform: G,
) -> impl Fn(Input) -> IResult<Input, Output, Error>
where
  Input: Clone + ::traits::Offset + InputLength + InputTake + InputTakeAtPosition + Slice<RangeFrom<usize>> + InputIter,
  Input: ::traits::ExtendInto<Item = ExtendItem, Extender = Output>,
  O1: ::traits::ExtendInto<Item = ExtendItem, Extender = Output>,
  O2: ::traits::ExtendInto<Item = ExtendItem, Extender = Output>,
  Output: core::iter::Extend<<Input as ::traits::ExtendInto>::Item>,
  Output: core::iter::Extend<<O1 as ::traits::ExtendInto>::Item>,
  Output: core::iter::Extend<<O2 as ::traits::ExtendInto>::Item>,
  <Input as InputIter>::Item: ::traits::AsChar,
  F: Fn(Input) -> IResult<Input, O1, Error>,
  G: Fn(Input) -> IResult<Input, O2, Error>,
  Error: ParseError<Input>,
{
  use traits::AsChar;

  move |input: Input| {
    let mut index = 0;
    let mut res = input.new_builder();

    let i = input.clone();

    while index < i.input_len() {
      let remainder = i.slice(index..);
      match normal(remainder.clone()) {
        Ok((i2, o)) => {
          o.extend_into(&mut res);
          if i2.input_len() == 0 {
            return Err(Err::Incomplete(Needed::Unknown));
          } else {
            index = input.offset(&i2);
          }
        }
        Err(Err::Error(_)) => {
          // unwrap() should be safe here since index < $i.input_len()
          if remainder.iter_elements().next().unwrap().as_char() == control_char {
            let next = index + control_char.len_utf8();
            let input_len = input.input_len();

            if next >= input_len {
              return Err(Err::Incomplete(Needed::Unknown));
            } else {
              match transform(i.slice(next..)) {
                Ok((i2, o)) => {
                  o.extend_into(&mut res);
                  if i2.input_len() == 0 {
                    return Err(Err::Incomplete(Needed::Unknown));
                  } else {
                    index = input.offset(&i2);
                  }
                }
                Err(e) => return Err(e),
              }
            }
          } else {
            return Ok((remainder, res));
          }
        }
        Err(e) => return Err(e),
      }
    }
    Err(Err::Incomplete(Needed::Unknown))
  }
}

#[doc(hidden)]
pub fn escaped_transformc<Input, Error, F, G, O1, O2, ExtendItem, Output>(
  i: Input,
  normal: F,
  control_char: char,
  transform: G,
) -> IResult<Input, Output, Error>
where
  Input: Clone + ::traits::Offset + InputLength + InputTake + InputTakeAtPosition + Slice<RangeFrom<usize>> + InputIter,
  Input: ::traits::ExtendInto<Item = ExtendItem, Extender = Output>,
  O1: ::traits::ExtendInto<Item = ExtendItem, Extender = Output>,
  O2: ::traits::ExtendInto<Item = ExtendItem, Extender = Output>,
  Output: core::iter::Extend<<Input as ::traits::ExtendInto>::Item>,
  Output: core::iter::Extend<<O1 as ::traits::ExtendInto>::Item>,
  Output: core::iter::Extend<<O2 as ::traits::ExtendInto>::Item>,
  <Input as InputIter>::Item: ::traits::AsChar,
  F: Fn(Input) -> IResult<Input, O1, Error>,
  G: Fn(Input) -> IResult<Input, O2, Error>,
  Error: ParseError<Input>,
{
  escaped_transform(normal, control_char, transform)(i)

}
