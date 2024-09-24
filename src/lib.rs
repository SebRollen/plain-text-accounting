use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, char, digit1, line_ending, not_line_ending, space0, space1},
    combinator::{map, map_res, opt, value},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use util::{float, space2, ws};

mod util;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Account<'a> {
    name: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Amount<'a> {
    currency: &'a str,
    amount: Decimal,
}

fn amount(input: &str) -> IResult<&str, Amount> {
    let (input, (currency, amount)) = alt((
        separated_pair(alpha1, space0, float),
        separated_pair(alpha1, space0, digit1),
        map(separated_pair(float, space0, alpha1), |(a, c)| (c, a)),
        map(separated_pair(digit1, space0, alpha1), |(a, c)| (c, a)),
    ))(input)?;
    let amount = Amount {
        currency,
        amount: Decimal::from_str(amount).unwrap(),
    };
    Ok((input, amount))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Posting<'a> {
    account: Account<'a>,
    amount: Option<Amount<'a>>,
}

fn posting(input: &str) -> IResult<&str, Posting> {
    let (input, account) = map(take_until(" "), |name| Account { name })(input)?;
    let (input, _) = space2(input)?;
    let (input, amount) = opt(amount)(input)?;
    Ok((input, Posting { account, amount }))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction<'a> {
    pub date: NaiveDate,
    pub auxillary_date: Option<NaiveDate>,
    pub state: TransactionState,
    pub code: Option<&'a str>,
    pub merchant: Option<&'a str>,
    pub memo: &'a str,
    pub postings: Vec<Posting<'a>>,
}

pub fn date(input: &str) -> IResult<&str, NaiveDate> {
    let (input, (year, _, month, _, day)) = tuple((
        map_res(digit1, str::parse),
        alt((tag("-"), tag("/"))),
        map_res(digit1, str::parse),
        alt((tag("-"), tag("/"))),
        map_res(digit1, str::parse),
    ))(input)?;
    Ok((
        input,
        NaiveDate::from_ymd_opt(year, month, day).expect("Invalid date"),
    ))
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

pub fn auxillary_date(input: &str) -> IResult<&str, NaiveDate> {
    preceded(tag("="), date)(input)
}

pub fn code(input: &str) -> IResult<&str, &str> {
    delimited(tag("("), take_until(")"), tag(")"))(input)
}

pub fn transaction(input: &str) -> IResult<&str, Transaction> {
    let (input, date) = date(input)?;
    let (input, auxillary_date) = alt(char(' '), opt(auxillary_date))(input)?;
    let (input, state) = transaction_state(input)?;
    let (input, code) = opt(code)(input)?;
    let (input, (merchant, memo)) = description(input)?;
    let (input, postings) = separated_list0(line_ending, posting)(input)?;
    Ok((
        input,
        Transaction {
            date,
            auxillary_date,
            state,
            code,
            merchant,
            memo,
            postings,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_and_extract<'a, T, F: Fn(&'a str) -> IResult<&'a str, T>>(input: &'a str, f: F) -> T {
        let (_, out) = f(input).unwrap();
        out
    }

    #[test]
    fn parse_amount() {
        assert_eq!(
            Amount {
                currency: "USD",
                amount: Decimal::new(2000, 2)
            },
            test_and_extract("USD 20", amount)
        );
        assert_eq!(
            Amount {
                currency: "USD",
                amount: Decimal::new(2000, 2)
            },
            test_and_extract("20.00 USD", amount)
        );
        assert_eq!(
            Amount {
                currency: "USD",
                amount: Decimal::new(2000, 2)
            },
            test_and_extract("USD20.00", amount)
        );
        assert_eq!(
            Amount {
                currency: "USD",
                amount: Decimal::new(2000, 2)
            },
            test_and_extract("20USD", amount)
        );
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
        let t = "2024-3-2=2024/03/03 * (#100) Merchant | Memo\n\tExpenses:Food  USD20.00";
        let parsed = test_and_extract(t, transaction);
        assert_eq!(parsed.date, NaiveDate::from_ymd_opt(2024, 3, 2).unwrap());
        assert_eq!(
            parsed.auxillary_date,
            Some(NaiveDate::from_ymd_opt(2024, 3, 3).unwrap())
        );
        assert_eq!(parsed.state, TransactionState::Cleared);
        assert_eq!(parsed.code, Some("#100"));
        assert_eq!(parsed.merchant, Some("Merchant"));
        assert_eq!(parsed.memo, "Memo");
        assert_eq!(
            parsed.postings,
            vec![Posting {
                account: Account {
                    name: "Expenses:Food"
                },
                amount: Some(Amount {
                    currency: "USD",
                    amount: Decimal::new(2000, 2)
                })
            }]
        );
    }
}
