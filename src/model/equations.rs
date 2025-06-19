mod operators {
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
        LeftSub,
        RightSub,
        LeftParen,
        RightParen,
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
                Operator::LeftSub | Operator::RightSub => 0,
                Operator::LeftParen | Operator::RightParen => 1,
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
                Operator::LeftSub => "[",
                Operator::RightSub => "]",
                Operator::LeftParen => "(",
                Operator::RightParen => ")",
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

mod function_calls {
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

use std::collections::HashSet;

use function_calls::FunctionTarget;
use operators::Operator;

use crate::{Identifier, core::NumericConstant};

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
