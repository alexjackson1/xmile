pub use common::{identifier, numeric_constant, parentheses, parse_integer, ws};
pub use expression::expression;
pub use units::unit_equation;

pub mod common {
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
        combinator::{map, map_res, opt, recognize},
        multi::many0,
        number::complete::double,
        sequence::{delimited, pair},
    };

    use crate::{
        Identifier, NumericConstant, UnitEquation, equation::identifier::IdentifierOptions,
    };

    /// Parse whitespace (spaces, tabs, newlines)
    pub fn ws<'a, P, O>(
        inner: P,
    ) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
    where
        P: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
    {
        delimited(multispace0, inner, multispace0)
    }

    /// Parse an identifier (variable name, function name, etc.)
    pub fn identifier(input: &str) -> IResult<&str, Identifier> {
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
    pub fn numeric_constant(input: &str) -> IResult<&str, NumericConstant> {
        map(double, NumericConstant).parse(input)
    }

    /// Parse parentheses around an expression
    pub fn parentheses<'a, F, G, T>(
        x: F,
        tx: G,
    ) -> impl Parser<&'a str, Output = T, Error = nom::error::Error<&'a str>>
    where
        F: Parser<&'a str, Output = T, Error = nom::error::Error<&'a str>>,
        G: Fn(T) -> T,
    {
        map(delimited(ws(char('(')), x, ws(char(')'))), tx)
    }

    /// Parse an integer (with optional unary minus)
    pub fn parse_integer(input: &str) -> IResult<&str, UnitEquation> {
        map_res(recognize(pair(opt(char('-')), digit1)), |s: &str| {
            s.parse::<i32>().map(UnitEquation::Integer)
        })
        .parse(input)
    }
}

pub mod expression {
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::{tag, tag_no_case, take_while1},
        character::complete::char,
        combinator::{map, value},
        multi::{separated_list0, separated_list1},
        sequence::{delimited, pair, preceded},
    };

    use crate::{Expression, Operator, equation::expression::function::FunctionTarget};

    use super::common::*;

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
            parentheses(expression, |expr| Expression::Parentheses(Box::new(expr))),
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
                    Operator::LessThanOrEq => {
                        Expression::LessThanOrEq(Box::new(left), Box::new(right))
                    }
                    Operator::GreaterThan => {
                        Expression::GreaterThan(Box::new(left), Box::new(right))
                    }
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
}

pub mod units {
    use nom::{
        IResult, Parser, branch::alt, character::complete::char, combinator::map,
        sequence::preceded,
    };

    use crate::UnitEquation;

    use super::common::*;

    /// Parse an alias (identifier)
    fn alias(input: &str) -> IResult<&str, UnitEquation> {
        map(identifier, UnitEquation::Alias).parse(input)
    }

    /// Parse a primary expression (integer, alias, or parentheses)
    fn atomic(input: &str) -> IResult<&str, UnitEquation> {
        alt((
            parse_integer,
            alias,
            parentheses(unit_equation, |eq| UnitEquation::Parentheses(Box::new(eq))),
        ))
        .parse(input)
    }

    /// Parse a unary expression (handles unary minus)
    fn unary(input: &str) -> IResult<&str, UnitEquation> {
        alt((
            map(preceded(ws(char('-')), unary), |expr| {
                UnitEquation::UnaryMinus(Box::new(expr))
            }),
            atomic,
        ))
        .parse(input)
    }
    /// Parse multiplication and division (left-associative, same precedence)
    fn multiplicative(input: &str) -> IResult<&str, UnitEquation> {
        let (input, mut left) = unary(input)?;

        let mut remaining = input;
        loop {
            if let Ok((new_input, _)) = ws(char('*')).parse(remaining) {
                let (new_input, right) = unary(new_input)?;
                left = UnitEquation::Multiplication(Box::new(left), Box::new(right));
                remaining = new_input;
            } else if let Ok((new_input, _)) = ws(char('/')).parse(remaining) {
                let (new_input, right) = unary(new_input)?;
                left = UnitEquation::Division(Box::new(left), Box::new(right));
                remaining = new_input;
            } else {
                break;
            }
        }

        Ok((remaining, left))
    }

    /// Parse a complete unit equation
    pub fn unit_equation(input: &str) -> IResult<&str, UnitEquation> {
        ws(|input| multiplicative.parse(input)).parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod common {}

    mod expression {
        use crate::{Expression, NumericConstant, equation::expression::function::FunctionTarget};

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

    mod units {
        use crate::UnitEquation;

        use super::*;

        #[test]
        fn test_integer() {
            let result = unit_equation("42");
            assert!(result.is_ok());
            if let Ok((_, UnitEquation::Integer(n))) = result {
                assert_eq!(n, 42);
            }
        }

        #[test]
        fn test_negative_integer() {
            let result = unit_equation("-42");
            assert!(result.is_ok());
            if let Ok((_, UnitEquation::Integer(n))) = result {
                assert_eq!(n, -42);
            }
        }

        #[test]
        fn test_unary_minus() {
            let result = unit_equation("- 42");
            assert!(result.is_ok());
            if let Ok((_, UnitEquation::UnaryMinus(_))) = result {
                // Success - we have a unary minus
            } else {
                panic!("Expected UnaryMinus variant");
            }
        }

        #[test]
        fn test_multiplication() {
            let result = unit_equation("2 * 3");
            assert!(result.is_ok());
            if let Ok((_, UnitEquation::Multiplication(_, _))) = result {
                // Success
            } else {
                panic!("Expected Multiplication variant");
            }
        }

        #[test]
        fn test_division() {
            let result = unit_equation("6 / 2");
            assert!(result.is_ok());
            if let Ok((_, UnitEquation::Division(_, _))) = result {
                // Success
            } else {
                panic!("Expected Division variant");
            }
        }

        #[test]
        fn test_parentheses() {
            let result = unit_equation("(42)");
            assert!(result.is_ok());
            if let Ok((_, UnitEquation::Parentheses(_))) = result {
                // Success
            } else {
                panic!("Expected Parentheses variant");
            }
        }

        #[test]
        fn test_complex_expression() {
            let result = unit_equation("miles * seconds / 1");
            println!("Result: {:?}", result);
            assert!(result.is_ok());
        }

        #[test]
        fn test_left_associativity() {
            // 2 * 3 * 4 should be parsed as (2 * 3) * 4
            let result = unit_equation("2 * 3 * 4");
            assert!(result.is_ok());
        }
    }
}
