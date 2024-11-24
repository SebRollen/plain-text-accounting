use nom::{
    branch::alt,
    character::complete::{char, one_of, space1},
    combinator::{opt, recognize},
    multi::{many0, many1},
    sequence::{preceded, terminated, tuple},
    IResult,
};

pub fn float(input: &str) -> IResult<&str, &str> {
    alt((
        // Case one: .42
        recognize(tuple((
            char('.'),
            decimal,
            opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
        ))), // Case two: 42e42 and 42.42e42
        recognize(tuple((
            decimal,
            opt(preceded(char('.'), decimal)),
            one_of("eE"),
            opt(one_of("+-")),
            decimal,
        ))), // Case three: 42. and 42.42
        recognize(tuple((decimal, char('.'), opt(decimal)))),
    ))(input)
}

fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
}

pub fn space2(input: &str) -> IResult<&str, ()> {
    let (input, _) = char(' ')(input)?;
    let (input, _) = space1(input)?;
    Ok((input, ()))
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_and_extract<'a, T, F: Fn(&'a str) -> IResult<&'a str, T>>(input: &'a str, f: F) -> T {
        let (_, out) = f(input).unwrap();
        out
    }

    #[test]
    fn parse_float() {
        assert_eq!(".42", test_and_extract(".42", float));
        assert_eq!("42E42", test_and_extract("42E42", float));
        assert_eq!("42.42E42", test_and_extract("42.42E42", float));
        assert_eq!("42.", test_and_extract("42.", float));
        assert_eq!("42.42", test_and_extract("42.42", float));
    }

    #[test]
    fn parse_decimal() {
        assert_eq!("123", test_and_extract("123", decimal));
        assert_eq!("123_456", test_and_extract("123_456", decimal));
    }

    #[test]
    fn parse_space2() {
        assert_eq!((), test_and_extract("  ", space2));
        assert_eq!((), test_and_extract("         ", space2));
    }
}
