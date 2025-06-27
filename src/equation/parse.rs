use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::{map, map_res, recognize, value},
    multi::{many0, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, preceded, terminated},
};

use crate::{
    Expression, Identifier, NumericConstant, Operator,
    equation::{expression::function::FunctionTarget, identifier::IdentifierOptions},
};

/// Parse whitespace (spaces, tabs, newlines)
fn ws<'a, P, O>(inner: P) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parse an identifier (variable name, function name, etc.)
fn identifier(input: &str) -> IResult<&str, Identifier> {
    map_res(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| {
            Identifier::parse(
                s,
                IdentifierOptions {
                    allow_dollar: true,
                    allow_digit: true,
                    allow_reserved: true,
                },
            )
        },
    )
    .parse(input)
}

/// Parse a numeric constant (integer or float)
fn numeric_constant(input: &str) -> IResult<&str, NumericConstant> {
    map(double, NumericConstant).parse(input)
}

/// Parse a parenthesised expression
fn parentheses(input: &str) -> IResult<&str, Expression> {
    map(
        delimited(ws(char('(')), expression, ws(char(')'))),
        |expr| Expression::Parentheses(Box::new(expr)),
    )
    .parse(input)
}

/// Parse function parameters
fn function_parameters(input: &str) -> IResult<&str, Vec<Expression>> {
    delimited(
        ws(char('(')),
        separated_list0(ws(char(',')), expression),
        ws(char(')')),
    )
    .parse(input)
}

/// Parse a function call
fn function_call(input: &str) -> IResult<&str, Expression> {
    map(pair(identifier, function_parameters), |(name, params)| {
        Expression::FunctionCall {
            target: FunctionTarget::Function(name),
            parameters: params,
        }
    })
    .parse(input)
}

/// Parse array subscript
fn subscript(input: &str) -> IResult<&str, Expression> {
    map(
        pair(
            identifier,
            delimited(
                ws(char('[')),
                separated_list1(ws(char(',')), expression),
                ws(char(']')),
            ),
        ),
        |(name, indices)| Expression::Subscript(name, indices),
    )
    .parse(input)
}

/// Parse an if-else expression
fn if_else(input: &str) -> IResult<&str, Expression> {
    map(
        (
            preceded(ws(tag_no_case("if")), expression),
            preceded(ws(tag_no_case("then")), expression),
            preceded(ws(tag_no_case("else")), expression),
        ),
        |(condition, then_branch, else_branch)| Expression::IfElse {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        },
    )
    .parse(input)
}

/// Parse inline comments
fn inline_comment(input: &str) -> IResult<&str, Expression> {
    map(
        preceded(ws(tag("//")), take_while1(|c| c != '\n' && c != '\r')),
        |comment: &str| Expression::InlineComment(comment.trim().to_string()),
    )
    .parse(input)
}

/// Parse primary expressions (atoms)
fn primary(input: &str) -> IResult<&str, Expression> {
    alt((
        map(numeric_constant, Expression::Constant),
        inline_comment,
        if_else,
        // Try subscript before function call since both start with identifier
        subscript,
        function_call,
        map(identifier, |id| Expression::Subscript(id, vec![])), // Variable reference
        parentheses,
    ))
    .parse(input)
}

/// Parse unary expressions (unary operators)
fn unary(input: &str) -> IResult<&str, Expression> {
    alt((
        map(preceded(ws(char('+')), unary), |expr| {
            Expression::UnaryPlus(Box::new(expr))
        }),
        map(preceded(ws(char('-')), unary), |expr| {
            Expression::UnaryMinus(Box::new(expr))
        }),
        map(preceded(ws(tag_no_case("not")), unary), |expr| {
            Expression::Not(Box::new(expr))
        }),
        primary,
    ))
    .parse(input)
}

/// Parse exponentiation (right-associative)
fn exponentiation(input: &str) -> IResult<&str, Expression> {
    let (input, first) = unary(input)?;

    if let Ok((input, _)) = ws(char('^')).parse(input) {
        let (input, second) = exponentiation(input)?; // Right-associative
        Ok((
            input,
            Expression::Exponentiation(Box::new(first), Box::new(second)),
        ))
    } else {
        Ok((input, first))
    }
}

/// Parse multiplication, division, and modulo (left-associative)
fn multiplicative(input: &str) -> IResult<&str, Expression> {
    let (mut input, mut left) = exponentiation(input)?;

    loop {
        let op_result = alt((
            value(Operator::Multiply, ws(char('*'))),
            value(Operator::Divide, ws(char('/'))),
            value(Operator::Modulo, ws(tag_no_case("mod"))),
        ))
        .parse(input);

        if let Ok((new_input, op)) = op_result {
            let (new_input, right) = exponentiation(new_input)?;
            input = new_input;
            left = match op {
                Operator::Multiply => Expression::Multiply(Box::new(left), Box::new(right)),
                Operator::Divide => Expression::Divide(Box::new(left), Box::new(right)),
                Operator::Modulo => Expression::Modulo(Box::new(left), Box::new(right)),
                _ => unreachable!(),
            };
        } else {
            break;
        }
    }

    Ok((input, left))
}

/// Parse addition and subtraction (left-associative)
fn additive(input: &str) -> IResult<&str, Expression> {
    let (mut input, mut left) = multiplicative(input)?;

    loop {
        let op_result = alt((
            value(Operator::Add, ws(char('+'))),
            value(Operator::Subtract, ws(char('-'))),
        ))
        .parse(input);

        if let Ok((new_input, op)) = op_result {
            let (new_input, right) = multiplicative(new_input)?;
            input = new_input;
            left = match op {
                Operator::Add => Expression::Add(Box::new(left), Box::new(right)),
                Operator::Subtract => Expression::Subtract(Box::new(left), Box::new(right)),
                _ => unreachable!(),
            };
        } else {
            break;
        }
    }

    Ok((input, left))
}

/// Parse relational operators
fn relational(input: &str) -> IResult<&str, Expression> {
    let (mut input, mut left) = additive(input)?;

    loop {
        let op_result = alt((
            value(Operator::LessThanOrEq, ws(tag("<="))),
            value(Operator::GreaterThanOrEq, ws(tag(">="))),
            value(Operator::LessThan, ws(char('<'))),
            value(Operator::GreaterThan, ws(char('>'))),
        ))
        .parse(input);

        if let Ok((new_input, op)) = op_result {
            let (new_input, right) = additive(new_input)?;
            input = new_input;
            left = match op {
                Operator::LessThan => Expression::LessThan(Box::new(left), Box::new(right)),
                Operator::LessThanOrEq => Expression::LessThanOrEq(Box::new(left), Box::new(right)),
                Operator::GreaterThan => Expression::GreaterThan(Box::new(left), Box::new(right)),
                Operator::GreaterThanOrEq => {
                    Expression::GreaterThanOrEq(Box::new(left), Box::new(right))
                }
                _ => unreachable!(),
            };
        } else {
            break;
        }
    }

    Ok((input, left))
}

/// Parse equality operators
fn equality(input: &str) -> IResult<&str, Expression> {
    let (mut input, mut left) = relational(input)?;

    loop {
        let op_result = alt((
            value(Operator::Equal, ws(char('='))),
            value(Operator::NotEqual, ws(tag("<>"))),
        ))
        .parse(input);

        if let Ok((new_input, op)) = op_result {
            let (new_input, right) = relational(new_input)?;
            input = new_input;
            left = match op {
                Operator::Equal => Expression::Equal(Box::new(left), Box::new(right)),
                Operator::NotEqual => Expression::NotEqual(Box::new(left), Box::new(right)),
                _ => unreachable!(),
            };
        } else {
            break;
        }
    }

    Ok((input, left))
}

/// Parse logical AND
fn logical_and(input: &str) -> IResult<&str, Expression> {
    let (mut input, mut left) = equality(input)?;

    loop {
        if let Ok((new_input, _)) = ws(tag_no_case("and")).parse(input) {
            let (new_input, right) = equality(new_input)?;
            input = new_input;
            left = Expression::And(Box::new(left), Box::new(right));
        } else {
            break;
        }
    }

    Ok((input, left))
}

/// Parse logical OR
fn logical_or(input: &str) -> IResult<&str, Expression> {
    let (mut input, mut left) = logical_and(input)?;

    loop {
        if let Ok((new_input, _)) = ws(tag_no_case("or")).parse(input) {
            let (new_input, right) = logical_and(new_input)?;
            input = new_input;
            left = Expression::Or(Box::new(left), Box::new(right));
        } else {
            break;
        }
    }

    Ok((input, left))
}

/// Parse a complete expression
pub fn expression(input: &str) -> IResult<&str, Expression> {
    ws(logical_or).parse(input)
}

/// Parse multiple expressions separated by semicolons or newlines
pub fn expressions(input: &str) -> IResult<&str, Vec<Expression>> {
    separated_list0(alt((ws(char(';')), ws(char('\n')))), expression).parse(input)
}

/// Main parsing function that handles the entire input
pub fn parse_expressions(input: &str) -> IResult<&str, Vec<Expression>> {
    terminated(expressions, multispace0).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_constant() {
        assert!(matches!(
            expression("42"),
            Ok((_, Expression::Constant(NumericConstant(42.0))))
        ));
        assert!(matches!(
            expression("3.14"),
            Ok((_, Expression::Constant(NumericConstant(3.14))))
        ));
    }

    #[test]
    fn test_arithmetic() {
        let result = expression("2 + 3 * 4").unwrap().1;
        // Should parse as 2 + (3 * 4) due to precedence
        match result {
            Expression::Add(left, right) => {
                assert!(matches!(*left, Expression::Constant(NumericConstant(2.0))));
                assert!(matches!(*right, Expression::Multiply(_, _)));
            }
            _ => panic!("Expected addition"),
        }
    }

    #[test]
    fn test_parentheses() {
        let result = expression("(2 + 3) * 4").unwrap().1;
        // Should parse as (2 + 3) * 4
        match result {
            Expression::Multiply(left, right) => {
                assert!(matches!(*left, Expression::Parentheses(_)));
                assert!(matches!(*right, Expression::Constant(NumericConstant(4.0))));
            }
            _ => panic!("Expected multiplication"),
        }
    }

    #[test]
    fn test_function_call() {
        let result = expression("ABS(-5)").unwrap().1;
        match result {
            Expression::FunctionCall { target, parameters } => {
                assert!(matches!(target, FunctionTarget::Function(_)));
                assert_eq!(parameters.len(), 1);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_subscript() {
        let result = expression("array[1, 2]").unwrap().1;
        match result {
            Expression::Subscript(_, indices) => {
                assert_eq!(indices.len(), 2);
            }
            _ => panic!("Expected subscript"),
        }
    }

    #[test]
    fn test_if_else() {
        let result = expression("if x > 0 then 1 else -1").unwrap().1;
        assert!(matches!(result, Expression::IfElse { .. }));
    }

    #[test]
    fn test_logical_operators() {
        let result = expression("x > 0 and y < 10 or z = 5").unwrap().1;
        // Should parse as ((x > 0) and (y < 10)) or (z = 5)
        assert!(matches!(result, Expression::Or(_, _)));
    }

    #[test]
    fn test_unary_operators() {
        assert!(matches!(
            expression("-5"),
            Ok((_, Expression::UnaryMinus(_)))
        ));
        assert!(matches!(
            expression("not true"),
            Ok((_, Expression::Not(_)))
        ));
    }

    #[test]
    fn test_exponentiation_right_associative() {
        let result = expression("2 ^ 3 ^ 4").unwrap().1;
        // Should parse as 2 ^ (3 ^ 4) - right associative
        match result {
            Expression::Exponentiation(left, right) => {
                assert!(matches!(*left, Expression::Constant(NumericConstant(2.0))));
                assert!(matches!(*right, Expression::Exponentiation(_, _)));
            }
            _ => panic!("Expected exponentiation"),
        }
    }

    #[test]
    fn test_complex_expression() {
        let input = "if (x + y) * 2 > threshold then ABS(x - y) else 0";
        let result = expression(input);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().1, Expression::IfElse { .. }));
    }
}
