use super::Name;
use crate::ir::util::spaces;
use nom::{
    branch::alt,
    character::complete::{alphanumeric1, digit1},
    combinator::map,
    error::VerboseError,
    sequence::preceded,
    IResult,
};

pub fn parse<'a>(source: &'a str) -> IResult<&'a str, Name, VerboseError<&'a str>> {
    preceded(
        spaces,
        alt((
            map(digit1, |i: &'a str| Name::Number(i.parse().unwrap())),
            map(alphanumeric1, |n: &'a str| Name::Name(n.to_string())),
        )),
    )(source)
}