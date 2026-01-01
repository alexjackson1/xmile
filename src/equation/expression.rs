use serde::{Deserialize, Serialize};

use std::fmt;

use function::FunctionTarget;
use operator::Operator;

use crate::equation::parse::expression;

use super::{Identifier, NumericConstant};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant(NumericConstant),
    // Operators
    Subscript(Identifier, Vec<Expression>),
    Parentheses(Box<Expression>),
    Exponentiation(Box<Expression>, Box<Expression>),
    UnaryPlus(Box<Expression>),
    UnaryMinus(Box<Expression>),
    Not(Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    LessThan(Box<Expression>, Box<Expression>),
    LessThanOrEq(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    GreaterThanOrEq(Box<Expression>, Box<Expression>),
    Equal(Box<Expression>, Box<Expression>),
    NotEqual(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    // Function Calls
    FunctionCall {
        target: FunctionTarget,
        parameters: Vec<Expression>,
    },
    // Control Structures
    IfElse {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },
    // Comments
    InlineComment(String),
}

impl Expression {
    pub fn constant(value: NumericConstant) -> Self {
        Expression::Constant(value)
    }

    pub fn subscript(identifier: Identifier, params: Vec<Expression>) -> Self {
        Expression::Subscript(identifier, params)
    }

    pub fn parentheses(expr: Expression) -> Self {
        Expression::Parentheses(Box::new(expr))
    }

    pub fn exponentiation(base: Expression, exponent: Expression) -> Self {
        Expression::Exponentiation(Box::new(base), Box::new(exponent))
    }

    pub fn unary_plus(expr: Expression) -> Self {
        Expression::UnaryPlus(Box::new(expr))
    }

    pub fn unary_minus(expr: Expression) -> Self {
        Expression::UnaryMinus(Box::new(expr))
    }

    pub fn not(expr: Expression) -> Self {
        Expression::Not(Box::new(expr))
    }

    pub fn multiply(lhs: Expression, rhs: Expression) -> Self {
        Expression::Multiply(Box::new(lhs), Box::new(rhs))
    }

    pub fn divide(lhs: Expression, rhs: Expression) -> Self {
        Expression::Divide(Box::new(lhs), Box::new(rhs))
    }

    pub fn modulo(lhs: Expression, rhs: Expression) -> Self {
        Expression::Modulo(Box::new(lhs), Box::new(rhs))
    }

    pub fn add(lhs: Expression, rhs: Expression) -> Self {
        Expression::Add(Box::new(lhs), Box::new(rhs))
    }

    pub fn subtract(lhs: Expression, rhs: Expression) -> Self {
        Expression::Subtract(Box::new(lhs), Box::new(rhs))
    }

    pub fn less_than(lhs: Expression, rhs: Expression) -> Self {
        Expression::LessThan(Box::new(lhs), Box::new(rhs))
    }

    pub fn less_than_or_eq(lhs: Expression, rhs: Expression) -> Self {
        Expression::LessThanOrEq(Box::new(lhs), Box::new(rhs))
    }

    pub fn greater_than(lhs: Expression, rhs: Expression) -> Self {
        Expression::GreaterThan(Box::new(lhs), Box::new(rhs))
    }

    pub fn greater_than_or_eq(lhs: Expression, rhs: Expression) -> Self {
        Expression::GreaterThanOrEq(Box::new(lhs), Box::new(rhs))
    }

    pub fn equal(lhs: Expression, rhs: Expression) -> Self {
        Expression::Equal(Box::new(lhs), Box::new(rhs))
    }

    pub fn not_equal(lhs: Expression, rhs: Expression) -> Self {
        Expression::NotEqual(Box::new(lhs), Box::new(rhs))
    }

    pub fn and(lhs: Expression, rhs: Expression) -> Self {
        Expression::And(Box::new(lhs), Box::new(rhs))
    }

    pub fn or(lhs: Expression, rhs: Expression) -> Self {
        Expression::Or(Box::new(lhs), Box::new(rhs))
    }

    pub fn function_call(target: FunctionTarget, parameters: Vec<Expression>) -> Self {
        Expression::FunctionCall { target, parameters }
    }

    pub fn if_else(
        condition: Expression,
        then_branch: Expression,
        else_branch: Expression,
    ) -> Self {
        Expression::IfElse {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }
    }

    pub fn inline_comment(comment: String) -> Self {
        Expression::InlineComment(comment)
    }

    pub fn top_operator(&self) -> Option<operator::Operator> {
        match self {
            Expression::Subscript(_, _) => Some(Operator::Subscript),
            Expression::Parentheses(_) => Some(Operator::Paren),
            Expression::Exponentiation(_, _) => Some(Operator::Exponentiation),
            Expression::UnaryPlus(_) => Some(Operator::UnaryPlus),
            Expression::UnaryMinus(_) => Some(Operator::UnaryMinus),
            Expression::Not(_) => Some(Operator::Not),
            Expression::Multiply(_, _) => Some(Operator::Multiply),
            Expression::Divide(_, _) => Some(Operator::Divide),
            Expression::Modulo(_, _) => Some(Operator::Modulo),
            Expression::Add(_, _) => Some(Operator::Add),
            Expression::Subtract(_, _) => Some(Operator::Subtract),
            Expression::LessThan(_, _) => Some(Operator::LessThan),
            Expression::LessThanOrEq(_, _) => Some(Operator::LessThanOrEq),
            Expression::GreaterThan(_, _) => Some(Operator::GreaterThan),
            Expression::GreaterThanOrEq(_, _) => Some(Operator::GreaterThanOrEq),
            Expression::Equal(_, _) => Some(Operator::Equal),
            Expression::NotEqual(_, _) => Some(Operator::NotEqual),
            Expression::And(_, _) => Some(Operator::And),
            Expression::Or(_, _) => Some(Operator::Or),
            Expression::Constant(_) => None,
            Expression::FunctionCall { .. } => None,
            Expression::IfElse { .. } => None,
            Expression::InlineComment(_) => None,
        }
    }

    pub fn operators(&self) -> Vec<Operator> {
        let mut acc = Vec::new();
        self.operators_recursive(&mut acc);
        acc
    }

    fn operators_recursive(&self, acc: &mut Vec<Operator>) {
        if let Some(op) = self.top_operator() {
            acc.push(op);
        }
        match self {
            Expression::Subscript(_, params) => {
                for param in params {
                    param.operators_recursive(acc);
                }
            }
            Expression::Parentheses(expr) => expr.operators_recursive(acc),
            Expression::Exponentiation(base, exponent) => {
                base.operators_recursive(acc);
                exponent.operators_recursive(acc);
            }
            Expression::UnaryPlus(expr) | Expression::UnaryMinus(expr) | Expression::Not(expr) => {
                expr.operators_recursive(acc)
            }
            Expression::Multiply(lhs, rhs)
            | Expression::Divide(lhs, rhs)
            | Expression::Modulo(lhs, rhs)
            | Expression::Add(lhs, rhs)
            | Expression::Subtract(lhs, rhs)
            | Expression::LessThan(lhs, rhs)
            | Expression::LessThanOrEq(lhs, rhs)
            | Expression::GreaterThan(lhs, rhs)
            | Expression::GreaterThanOrEq(lhs, rhs)
            | Expression::Equal(lhs, rhs)
            | Expression::NotEqual(lhs, rhs)
            | Expression::And(lhs, rhs)
            | Expression::Or(lhs, rhs) => {
                lhs.operators_recursive(acc);
                rhs.operators_recursive(acc);
            }
            Expression::FunctionCall { parameters, .. } => {
                for param in parameters {
                    param.operators_recursive(acc);
                }
            }
            Expression::IfElse {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.operators_recursive(acc);
                then_branch.operators_recursive(acc);
                else_branch.operators_recursive(acc);
            }
            Expression::InlineComment(_) => {}
            Expression::Constant(_) => {}
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Constant(value) => write!(f, "{}", value),
            Expression::Subscript(id, params) => {
                // For serialization, use raw form to preserve quotes if originally quoted
                // For display, we could use normalized, but for XML serialization we need raw
                let id_str = id.raw();
                
                if params.is_empty() {
                    return write!(f, "{}", id_str);
                }

                write!(f, "{}[", id_str)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, "]")
            }
            Expression::Parentheses(expr) => write!(f, "({})", expr),
            Expression::Exponentiation(base, exponent) => write!(f, "{} ^ {}", base, exponent),
            Expression::UnaryPlus(expr) => write!(f, "+{}", expr),
            Expression::UnaryMinus(expr) => write!(f, "-{}", expr),
            Expression::Not(expr) => write!(f, "NOT {}", expr),
            Expression::Multiply(lhs, rhs) => write!(f, "{} * {}", lhs, rhs),
            Expression::Divide(lhs, rhs) => write!(f, "{} / {}", lhs, rhs),
            Expression::Modulo(lhs, rhs) => write!(f, "{} MOD {}", lhs, rhs),
            Expression::Add(lhs, rhs) => write!(f, "{} + {}", lhs, rhs),
            Expression::Subtract(lhs, rhs) => write!(f, "{} - {}", lhs, rhs),
            Expression::LessThan(lhs, rhs) => write!(f, "{} < {}", lhs, rhs),
            Expression::LessThanOrEq(lhs, rhs) => write!(f, "{} <= {}", lhs, rhs),
            Expression::GreaterThan(lhs, rhs) => write!(f, "{} > {}", lhs, rhs),
            Expression::GreaterThanOrEq(lhs, rhs) => write!(f, "{} >= {}", lhs, rhs),
            Expression::Equal(lhs, rhs) => write!(f, "{} = {}", lhs, rhs),
            Expression::NotEqual(lhs, rhs) => write!(f, "{} <> {}", lhs, rhs),
            Expression::And(lhs, rhs) => write!(f, "{} AND {}", lhs, rhs),
            Expression::Or(lhs, rhs) => write!(f, "{} OR {}", lhs, rhs),
            Expression::FunctionCall {
                target: _,
                parameters,
            } => {
                write!(f, "{}(", parameters[0])?;
                for (i, param) in parameters.iter().enumerate().skip(1) {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Expression::IfElse {
                condition,
                then_branch,
                else_branch,
            } => write!(
                f,
                "IF {} THEN {} ELSE {}",
                condition, then_branch, else_branch
            ),
            Expression::InlineComment(comment) => write!(f, "// {}", comment),
        }
    }
}

impl<'de> Deserialize<'de> for Expression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize as a string
        let s: String = Deserialize::deserialize(deserializer)?;

        // Parse the string into an Expression
        let (output, expression) = expression(&s).map_err(serde::de::Error::custom)?;

        // Ensure the entire string was consumed
        if !output.is_empty() {
            return Err(serde::de::Error::custom(format!(
                "Unexpected trailing characters after expression: '{}'",
                output
            )));
        }

        // Return the parsed Expression
        Ok(expression)
    }
}

impl Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize the Expression as a string
        let expr_str = self.to_string();
        serializer.serialize_str(&expr_str)
    }
}

pub mod operator {
    //! ### XMILE Operators (Section 3.3.1)
    //! The following table lists the supported operators in precedence order.
    //! All but exponentiation and the unary operators have left-to-right
    //! associativity (right-to-left is the only thing that makes sense for
    //! unary operators).
    //!
    //! | Operators | Precedence Group                 |
    //! |:---------:|:---------------------------------|
    //! | [ ]       | Subscripts                       |
    //! | ( )       | Parentheses                      |
    //! | ^         | Exponentiation                   |
    //! | + – NOT   | Unary operators                  |
    //! | * / MOD   | Multiplication, division, modulo |
    //! | + –       | Addition, subtraction            |
    //! | < <= > >= | Relational operators             |
    //! | = <>      | Equality operators               |
    //! | AND       | Logical and                      |
    //! | OR        | Logical or                       |
    //!
    //! Note the logical, relational, and equality operators are all defined to
    //! return zero (0) if the result is false and one (1) if the result is
    //! true.  
    //!
    //! Modulo is defined to return the floored modulus proposed by Knuth. In
    //! this form, the sign of the result always follows the sign of the
    //! divisor, as one would expect.
    //!
    //! #### Sample Expressions
    //!
    //! The following are some sample expressions that illustrate the use of
    //! the operators:
    //!
    //! ```text
    //! a * b
    //! (x < 5) and (y >= 3)
    //! (–3)^x
    //! ```

    use std::{cmp, fmt};

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Operator {
        Subscript,
        Paren,
        Exponentiation,
        UnaryPlus,
        UnaryMinus,
        Not,
        Multiply,
        Divide,
        Modulo,
        Add,
        Subtract,
        LessThan,
        LessThanOrEq,
        GreaterThan,
        GreaterThanOrEq,
        Equal,
        NotEqual,
        And,
        Or,
    }

    impl Operator {
        pub fn precedence(&self) -> u8 {
            match self {
                Operator::Subscript => 0,
                Operator::Paren => 1,
                Operator::Exponentiation => 2,
                Operator::UnaryPlus | Operator::UnaryMinus | Operator::Not => 3,
                Operator::Multiply | Operator::Divide | Operator::Modulo => 4,
                Operator::Add | Operator::Subtract => 5,
                Operator::LessThan
                | Operator::LessThanOrEq
                | Operator::GreaterThan
                | Operator::GreaterThanOrEq => 6,
                Operator::Equal | Operator::NotEqual => 7,
                Operator::And => 8,
                Operator::Or => 9,
            }
        }
    }

    // impl compare for Operator
    impl fmt::Display for Operator {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let symbol = match self {
                Operator::Subscript => "[]",
                Operator::Paren => "()",
                Operator::Exponentiation => "^",
                Operator::UnaryPlus => "+",
                Operator::UnaryMinus => "-",
                Operator::Not => "NOT",
                Operator::Multiply => "*",
                Operator::Divide => "/",
                Operator::Modulo => "MOD",
                Operator::Add => "+",
                Operator::Subtract => "-",
                Operator::LessThan => "<",
                Operator::LessThanOrEq => "<=",
                Operator::GreaterThan => ">",
                Operator::GreaterThanOrEq => ">=",
                Operator::Equal => "=",
                Operator::NotEqual => "<>",
                Operator::And => "AND",
                Operator::Or => "OR",
            };
            write!(f, "{}", symbol)
        }
    }

    impl PartialOrd for Operator {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            Some(self.precedence().cmp(&other.precedence()))
        }
    }

    impl Ord for Operator {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            self.precedence().cmp(&other.precedence())
        }
    }
}

pub mod function {
    //! ### XMILE Function Calls (Section 3.3.2)
    //!
    //! Parentheses are also used to provide parameters to function calls,
    //! e.g., ABS(x). In this case, they take precedence over all operators (as
    //! do the commas separating parameters). Note that functions that do not
    //! take parameters do not include parentheses when used in an equation,
    //! e.g., TIME. There are several cases where variable names MAY be
    //! (syntactically) used like a function in equations:
    //!
    //! - Named graphical function:  The graphical function is evaluated at the passed value, e.g., given the graphical function named `cost_f`, `cost_f(2003)` evaluates the graphical function at `x = 2003`.
    //! - Named model:  A model that has a name, defined submodel inputs, and one submodel output can be treated as a function in an equation, e.g., given the model named `maximum` with one submodel input and one submodel output that gives the maximum value of the input over this run, `maximum(Balance)` evaluates to the maximum value of `Balance` during this run. When there is more than one submodel input, the order of the parameters must be defined as they are for a macro definition. For more information, see Sections 3.6.1 (macros) and 3.7.4 (submodels).
    //! - Array name:  An array name can be passed the flat index (i.e., the linear row-major index) of an element to access that element. Since functions can only return one value, this can be useful when a function must identify an element across a multidimensional array (e.g., the RANK built-in). For example, given the three-dimensional array `A` with bounds `[2, 3, 4]`, `A(10)` refers to the tenth element in row-major order, i.e., element `A[1, 3, 2]`. See Section 3.7.1 for more information about arrays.

    use crate::Identifier;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum FunctionTarget {
        /// Named function, e.g., `ABS(x)`
        Function(Identifier),
        /// Named graphical function, e.g., `cost_f(2003)`
        GraphicalFunction(Identifier),
        /// Named model with defined inputs and one output, e.g., `maximum(Balance)`
        Model(Identifier),
        /// Array name with flat index, e.g., `A(10)` for a three-dimensional array `A` with bounds `[2, 3, 4]`
        Array(Identifier),
    }
}
