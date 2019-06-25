// mods

// [[file:~/Workspace/Programming/gchemol-rs/text-parser/text-parser.note::*mods][mods:1]]
mod parser;

pub use crate::parser::*;
// mods:1 ends here

// common

// [[file:~/Workspace/Programming/gchemol-rs/text-parser/text-parser.note::*common][common:1]]
pub(crate) mod common {
    pub use quicli::prelude::*;
    pub type Result<T> = ::std::result::Result<T, Error>;
}
// common:1 ends here

// nom

// [[file:~/Workspace/Programming/gchemol-rs/text-parser/text-parser.note::*nom][nom:1]]
pub use nom::branch::alt;
pub use nom::bytes::streaming::{is_not, tag, tag_no_case, take_until};
pub use nom::character::streaming::{
    alpha1, alphanumeric1, digit1, line_ending, multispace1, not_line_ending, space0, space1,
};
pub use nom::combinator::{map_res, opt, recognize};
pub use nom::number::streaming::double;

pub use nom::multi::{count, many0, many1};

pub use nom::sequence::{pair, preceded, terminated, tuple};

// macros
use nom::IResult;
use nom::{do_parse, named};
// nom:1 ends here

// eof
// 没有必要了?

// [[file:~/Workspace/Programming/gchemol-rs/text-parser/text-parser.note::*eof][eof:1]]
use crate::common::*;

// Indicating the end of streaming
const MAGIC_EOF: &str = "####---MAGIC_END_OF_FILE---####";

// Indicating the end of streaming
pub fn eof(s: &str) -> IResult<&str, &str> {
    nom::bytes::complete::tag(MAGIC_EOF)(s)
}

#[test]
fn test_eof() {
    assert_eq!(true, eof(MAGIC_EOF).is_ok());
}
// eof:1 ends here

// lines

// [[file:~/Workspace/Programming/gchemol-rs/text-parser/text-parser.note::*lines][lines:1]]
/// Anything except whitespace, this parser will not consume "\n" character
pub fn not_space(s: &str) -> IResult<&str, &str> {
    is_not(" \t\r\n")(s)
}

/// Match line ending preceded with zero or more whitespace chracters
pub fn eol(s: &str) -> IResult<&str, &str> {
    nom::sequence::terminated(space0, line_ending)(s)
}

/// Match one unsigned integer: 123
pub fn unsigned_digit(s: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(s)
}

/// Read the remaining line
pub fn read_until_eol(s: &str) -> IResult<&str, &str> {
    nom::sequence::terminated(not_line_ending, line_ending)(s)
}

/// Match a blank line containing zero or more whitespace character
pub fn blank_line(s: &str) -> IResult<&str, &str> {
    let (r, _) = nom::sequence::pair(space0, line_ending)(s)?;
    Ok((r, ""))
}

#[test]
fn test_unsigned_digit() {
    let (_, n) = unsigned_digit("123\n").expect("usize");
    assert_eq!(n, 123);
}

#[test]
fn test_blank_line() {
    let (r, _) = blank_line("\t \nend\n").expect("blank_line");
    assert_eq!(r, "end\n");
}

#[test]
fn test_read_until_eol() {
    let (r, _) = read_until_eol("this is the end\nok\n").expect("parser: read_until_eol");
    assert_eq!(r, "ok\n");

    let (r, _) = read_until_eol("\n").expect("parser: read_until_eol empty line");
    assert_eq!(r, "");
}

#[test]
#[ignore]
fn test_no_space() {
    let x = "1002 xx b";

    let v = unsigned_digit(x);
    dbg!(v);
}
// lines:1 ends here

// numbers

// [[file:~/Workspace/Programming/gchemol-rs/text-parser/text-parser.note::*numbers][numbers:1]]
/// Parse a line containing an unsigned integer
pub fn read_usize(s: &str) -> IResult<&str, usize> {
    // allow white spaces
    let p = nom::sequence::delimited(space0, unsigned_digit, space0);
    nom::sequence::terminated(p, line_ending)(s)
}

/// Parse a line containing many unsigned integers
pub fn read_usize_many(s: &str) -> IResult<&str, Vec<usize>> {
    nom::sequence::terminated(
        nom::sequence::delimited(
            space0,
            nom::multi::separated_nonempty_list(space1, unsigned_digit),
            space0,
        ),
        line_ending,
    )(s)
}

/// Parse a line containing a float number
pub fn read_f64(s: &str) -> IResult<&str, f64> {
    // allow white spaces
    let p = nom::sequence::delimited(space0, double, space0);
    nom::sequence::terminated(p, line_ending)(s)
}

/// Parse a line containing many float numbers
pub fn read_f64_many(s: &str) -> IResult<&str, Vec<f64>> {
    nom::sequence::terminated(
        nom::sequence::delimited(
            space0,
            nom::multi::separated_nonempty_list(space1, double),
            space0,
        ),
        line_ending,
    )(s)
}

#[test]
fn test_read_numbers() {
    let (_, ns) = read_usize_many("11 2 3 4 5\r\n\n").expect("usize parser");
    assert_eq!(5, ns.len());
    let _ = read_usize_many(" 11 2 3 4 5 \n").expect("usize parser");
    let _ = read_usize_many("11 2 3 4 5 \r\n").expect("usize parser");

    let line = " 1.2  3.4 -5.7 0.2 \n";
    let (_, fs) = read_f64_many(line).expect("f64 parser");
    assert_eq!(4, fs.len());
}
// numbers:1 ends here

// coordinates

// [[file:~/Workspace/Programming/gchemol-rs/text-parser/text-parser.note::*coordinates][coordinates:1]]
/// Consume three float numbers separated by one or more spaces
/// Return xyz array
pub fn xyz_array(s: &str) -> IResult<&str, [f64; 3]> {
    let (r, (x, _, y, _, z)) = nom::sequence::tuple((double, space1, double, space1, double))(s)?;

    Ok((r, [x, y, z]))
}

/// Read xyz coordinates specified in a line
named!(pub read_xyz<&str, [f64; 3]>, do_parse!(
       space0 >>
    x: double >> space1      >>
    y: double >> space1      >>
    z: double >> eol         >>
    (
        [x, y, z]
    )
));

#[test]
fn test_parse_xyz() {
    let (_, x) = xyz_array("-11.4286  1.7645  0.0000 ").unwrap();
    assert_eq!(x[2], 0.0);

    let (_, x) = xyz_array("-11.4286  1.7645  0.0000\n").unwrap();
    assert_eq!(x[2], 0.0);

    let (_, x) = read_xyz("-11.4286\t1.7E-5  0.0000 \n").unwrap();
    assert_eq!(x[2], 0.0);
}
// coordinates:1 ends here
