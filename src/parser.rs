use std::{cmp::Ordering, collections::HashMap};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};

use crate::error::{Error, Result};

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Symbol(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Object(l0), Self::Object(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::String(l0), Self::Number(r0)) => l0 == &r0.to_string(),
            (Self::Number(l0), Self::String(r0)) => &l0.to_string() == r0,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0.partial_cmp(r0),
            (Self::Number(l0), Self::Number(r0)) => l0.partial_cmp(r0),
            (Self::Boolean(l0), Self::Boolean(r0)) => l0.partial_cmp(r0),
            (Self::Symbol(l0), Self::Symbol(r0)) => l0.partial_cmp(r0),
            (Self::Object(l0), Self::Object(r0)) => {
                let l_keys: Vec<_> = l0.keys().collect();
                let r_keys: Vec<_> = r0.keys().collect();
                match l_keys.cmp(&r_keys) {
                    Ordering::Equal => {
                        for key in l_keys {
                            match l0[key].partial_cmp(&r0[key]) {
                                Some(Ordering::Equal) => continue,
                                ord => return ord,
                            }
                        }
                        Some(Ordering::Equal)
                    }
                    ord => Some(ord),
                }
            }
            (Self::Array(l0), Self::Array(r0)) => {
                for (l, r) in l0.iter().zip(r0.iter()) {
                    match l.partial_cmp(r) {
                        Some(Ordering::Equal) => continue,
                        ord => return ord,
                    }
                }
                Some(l0.len().cmp(&r0.len()))
            }
            (Self::String(l0), Self::Number(r0)) => l0.parse::<f64>().unwrap().partial_cmp(r0),
            (Self::Number(l0), Self::String(r0)) => l0.partial_cmp(&r0.parse::<f64>().unwrap()),
            _ => None,
        }
    }

    fn ge(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap() == Ordering::Greater
            || self.partial_cmp(other).unwrap() == Ordering::Equal
    }

    fn gt(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap() == Ordering::Greater
    }

    fn le(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap() == Ordering::Less
            || self.partial_cmp(other).unwrap() == Ordering::Equal
    }

    fn lt(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap() == Ordering::Less
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinOp {
    Eq,
    NotEq,
    Gt,
    Ge,
    Lt,
    Le,
    Dot,
    Index,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Dollar,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Value(Value),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Function {
        name: String,
        arg: Box<Expr>,
    },
}

fn string_expr(s: &str) -> Expr {
    Expr::Value(Value::String(s.to_string()))
}

fn number_expr(num: &str) -> Expr {
    Expr::Value(Value::Number(num.parse().unwrap()))
}

fn bool_expr(b: bool) -> Expr {
    Expr::Value(Value::Boolean(b))
}

fn symbol_expr(s: &str) -> Expr {
    Expr::Value(Value::Symbol(s.to_string()))
}

fn parse_value(input: &str) -> IResult<&str, Expr> {
    alt((
        // String
        map(
            delimited(char('\''), take_while1(|c| c != '\''), char('\'')),
            string_expr,
        ),
        // Number
        map(
            recognize(pair(digit1, opt(pair(char('.'), digit1)))),
            number_expr,
        ),
        // Boolean
        map(
            alt((map(tag("true"), |_| true), map(tag("false"), |_| false))),
            bool_expr,
        ),
        // Symbol
        map(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            symbol_expr,
        ),
    ))(input)
}

fn parse_function(input: &str) -> IResult<&str, Expr> {
    let (input, name) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, _) = preceded(nom::character::complete::multispace0, char('('))(input)?;
    let (input, arg) = preceded(nom::character::complete::multispace0, parse_binary)(input)?;
    let (input, _) = preceded(nom::character::complete::multispace0, char(')'))(input)?;

    Ok((
        input,
        Expr::Function {
            name: name.to_string(),
            arg: Box::new(arg),
        },
    ))
}

fn parse_unary(input: &str) -> IResult<&str, Expr> {
    alt((
        // Function expression
        parse_function,
        // Dollar expression with optional dot or index access
        map(
            pair(
                preceded(char('$'), parse_value),
                many0(alt((
                    // ドットアクセス
                    map(
                        preceded(
                            preceded(nom::character::complete::multispace0, char('.')),
                            parse_value,
                        ),
                        |field| (BinOp::Dot, field),
                    ),
                    // インデックスアクセス
                    map(
                        delimited(
                            preceded(nom::character::complete::multispace0, char('[')),
                            parse_binary,
                            preceded(nom::character::complete::multispace0, char(']')),
                        ),
                        |expr| (BinOp::Index, expr),
                    ),
                ))),
            ),
            |(expr, accesses)| {
                let mut result = Expr::UnaryOp {
                    op: UnaryOp::Dollar,
                    expr: Box::new(expr),
                };

                for (op, access_expr) in accesses {
                    result = Expr::BinaryOp {
                        op,
                        left: Box::new(result),
                        right: Box::new(access_expr),
                    };
                }

                result
            },
        ),
        // Not expression
        map(preceded(char('!'), parse_value), |expr| Expr::UnaryOp {
            op: UnaryOp::Not,
            expr: Box::new(expr),
        }),
        parse_value,
    ))(input)
}

fn parse_binary(input: &str) -> IResult<&str, Expr> {
    let (input, left) = parse_unary(input)?;

    let (input, op) = opt(preceded(
        nom::character::complete::multispace0,
        alt((
            map(tag("=="), |_| BinOp::Eq),
            map(tag("!="), |_| BinOp::NotEq),
            map(tag(">="), |_| BinOp::Ge),
            map(tag(">"), |_| BinOp::Gt),
            map(tag("<="), |_| BinOp::Le),
            map(tag("<"), |_| BinOp::Lt),
            map(tag("."), |_| BinOp::Dot),
        )),
    ))(input)?;

    match op {
        Some(op) => {
            let (input, right) =
                preceded(nom::character::complete::multispace0, parse_unary)(input)?;
            Ok((
                input,
                Expr::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            ))
        }
        None => Ok((input, left)),
    }
}

pub fn parse(input: &str) -> Result<Expr> {
    match parse_binary(input.trim()) {
        Ok(("", expr)) => Ok(expr),
        Ok((remain, _)) => Err(Error::TrailingInput(remain.to_string())),
        Err(e) => Err(Error::ParseError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse("'hello'"),
            Ok(Expr::Value(Value::String("hello".to_string())))
        );
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse("123.45"), Ok(Expr::Value(Value::Number(123.45))));
    }

    #[test]
    fn test_parse_boolean() {
        assert_eq!(parse("true"), Ok(Expr::Value(Value::Boolean(true))));
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(
            parse("variable_name"),
            Ok(Expr::Value(Value::Symbol("variable_name".to_string())))
        );
    }

    #[test]
    fn test_parse_dollar() {
        assert_eq!(
            parse("$state"),
            Ok(Expr::UnaryOp {
                op: UnaryOp::Dollar,
                expr: Box::new(Expr::Value(Value::Symbol("state".to_string())))
            })
        );
    }

    #[test]
    fn test_parse_binary() {
        assert_eq!(
            parse("value == 1"),
            Ok(Expr::BinaryOp {
                op: BinOp::Eq,
                left: Box::new(Expr::Value(Value::Symbol("value".to_string()))),
                right: Box::new(Expr::Value(Value::Number(1.0)))
            })
        );
    }

    #[test]
    fn test_parse_condition() {
        assert_eq!(
            parse("$args.value == 1"),
            Ok(Expr::BinaryOp {
                op: BinOp::Eq,
                left: Box::new(Expr::BinaryOp {
                    op: BinOp::Dot,
                    left: Box::new(Expr::UnaryOp {
                        op: UnaryOp::Dollar,
                        expr: Box::new(Expr::Value(Value::Symbol("args".to_string()))),
                    }),
                    right: Box::new(Expr::Value(Value::Symbol("value".to_string()))),
                }),
                right: Box::new(Expr::Value(Value::Number(1.0))),
            })
        );
    }

    #[test]
    fn test_parse_too_many_dots() {
        assert_eq!(
            parse("$value.field.field"),
            Ok(Expr::BinaryOp {
                op: BinOp::Dot,
                left: Box::new(Expr::BinaryOp {
                    op: BinOp::Dot,
                    left: Box::new(Expr::UnaryOp {
                        op: UnaryOp::Dollar,
                        expr: Box::new(Expr::Value(Value::Symbol("value".to_string()))),
                    }),
                    right: Box::new(Expr::Value(Value::Symbol("field".to_string()))),
                }),
                right: Box::new(Expr::Value(Value::Symbol("field".to_string()))),
            })
        );
    }

    #[test]
    fn test_parse_function() {
        assert_eq!(
            parse("len('hello')"),
            Ok(Expr::Function {
                name: "len".to_string(),
                arg: Box::new(Expr::Value(Value::String("hello".to_string()))),
            })
        );

        assert_eq!(
            parse("contains($state.name)"),
            Ok(Expr::Function {
                name: "contains".to_string(),
                arg: Box::new(Expr::BinaryOp {
                    op: BinOp::Dot,
                    left: Box::new(Expr::UnaryOp {
                        op: UnaryOp::Dollar,
                        expr: Box::new(Expr::Value(Value::Symbol("state".to_string()))),
                    }),
                    right: Box::new(Expr::Value(Value::Symbol("name".to_string()))),
                }),
            })
        );
    }

    #[test]
    fn test_parse_index_access() {
        assert_eq!(
            parse("$args.users['admin']"),
            Ok(Expr::BinaryOp {
                op: BinOp::Index,
                left: Box::new(Expr::BinaryOp {
                    op: BinOp::Dot,
                    left: Box::new(Expr::UnaryOp {
                        op: UnaryOp::Dollar,
                        expr: Box::new(Expr::Value(Value::Symbol("args".to_string()))),
                    }),
                    right: Box::new(Expr::Value(Value::Symbol("users".to_string()))),
                }),
                right: Box::new(Expr::Value(Value::String("admin".to_string()))),
            })
        );

        assert_eq!(
            parse("$args.data[$key]"),
            Ok(Expr::BinaryOp {
                op: BinOp::Index,
                left: Box::new(Expr::BinaryOp {
                    op: BinOp::Dot,
                    left: Box::new(Expr::UnaryOp {
                        op: UnaryOp::Dollar,
                        expr: Box::new(Expr::Value(Value::Symbol("args".to_string()))),
                    }),
                    right: Box::new(Expr::Value(Value::Symbol("data".to_string()))),
                }),
                right: Box::new(Expr::UnaryOp {
                    op: UnaryOp::Dollar,
                    expr: Box::new(Expr::Value(Value::Symbol("key".to_string()))),
                }),
            })
        );
    }
}
