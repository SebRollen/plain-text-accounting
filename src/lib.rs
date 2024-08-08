use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric0, char, digit1, not_line_ending, space0, space1},
    combinator::{map_res, opt, value},
    sequence::{delimited, preceded, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Cleared,
    Pending,
    Uncleared,
}

pub fn transaction_state(input: &str) -> IResult<&str, TransactionState> {
    let (input, state) = opt(alt((
        value(TransactionState::Cleared, char('*')),
        value(TransactionState::Pending, char('!')),
    )))(input)?;
    Ok((input, state.unwrap_or(TransactionState::Uncleared)))
}

pub struct Transaction<'a> {
    pub date: NaiveDate,
    pub auxillary_date: Option<NaiveDate>,
    pub state: TransactionState,
    pub code: Option<&'a str>,
    pub merchant: Option<&'a str>,
    pub memo: &'a str,
}

pub fn date(input: &str) -> IResult<&str, NaiveDate> {
    let (input, (year, _, month, _, day)) = tuple((
        map_res(digit1, str::parse),
        alt((tag("-"), tag("/"))),
        map_res(digit1, str::parse),
        alt((tag("-"), tag("/"))),
        map_res(digit1, str::parse),
    ))(input)?;
    Ok((input, NaiveDate::from_ymd_opt(year, month, day).unwrap()))
}

pub fn description(input: &str) -> IResult<&str, (Option<&str>, &str)> {
    let (input, merchant) = opt(take_until(" | "))(input)?;
    let (input, memo) = if merchant.is_some() {
        preceded(tag(" | "), not_line_ending)(input)?
    } else {
        not_line_ending(input)?
    };
    Ok((input, (merchant, memo)))
}

pub fn auxillary_date(input: &str) -> IResult<&str, Option<NaiveDate>> {
    opt(preceded(tag("="), date))(input)
}

pub fn transaction(input: &str) -> IResult<&str, Transaction> {
    let (input, date) = date(input)?;
    let (input, auxillary_date) = auxillary_date(input)?;
    let (input, _) = space1(input)?;
    let (input, state) = transaction_state(input)?;
    let (input, _) = space0(input)?;
    // TODO: accept more than alphanumerics here
    let (input, code) = opt(delimited(tag("("), alphanumeric0, tag(")")))(input)?;
    let (input, _) = space0(input)?;
    let (input, (merchant, memo)) = description(input)?;
    Ok((
        input,
        Transaction {
            date,
            auxillary_date,
            state,
            code,
            merchant,
            memo,
        },
    ))
}

mod test {
    use super::*;

    #[allow(dead_code)]
    fn test_and_extract<'a, T, F: Fn(&'a str) -> IResult<&'a str, T>>(input: &'a str, f: F) -> T {
        let (_, out) = f(input).unwrap();
        out
    }

    #[test]
    fn parse_transaction_state() {
        assert_eq!(
            TransactionState::Cleared,
            test_and_extract("*", transaction_state)
        );
        assert_eq!(
            TransactionState::Pending,
            test_and_extract("!", transaction_state)
        );
        assert_eq!(
            TransactionState::Uncleared,
            test_and_extract("", transaction_state)
        );
    }

    #[test]
    fn parse_date() {
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            test_and_extract("2024-1-1", date)
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            test_and_extract("2024-01-01", date)
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            test_and_extract("2024/1/1", date)
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            test_and_extract("2024/01/01", date)
        );
    }

    #[test]
    fn parse_description() {
        assert_eq!((None, "foo"), test_and_extract("foo", description));
        assert_eq!(
            (Some("foo"), "bar"),
            test_and_extract("foo | bar", description)
        );
    }

    #[test]
    fn parse_transaction() {
        let t = "2024-3-2=2024/03/03 * (100) Merchant | Memo";
        let parsed = test_and_extract(t, transaction);
        assert_eq!(parsed.date, NaiveDate::from_ymd_opt(2024, 3, 2).unwrap());
        assert_eq!(
            parsed.auxillary_date,
            Some(NaiveDate::from_ymd_opt(2024, 3, 3).unwrap())
        );
        assert_eq!(parsed.state, TransactionState::Cleared);
        assert_eq!(parsed.code, Some("100"));
        assert_eq!(parsed.merchant, Some("Merchant"));
        assert_eq!(parsed.memo, "Memo");
    }
}
