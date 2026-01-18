// Each variable OPTIONALLY has its own units of measure, which are specified
// by combining other units defined in the units namespace as described below.
//
// Units of measure are specified with XMILE expressions, called the unit
// equation, restricted to the operators ^ (exponentiation), - or *
// (multiplication), and / (division) with parentheses as needed to group
// units in the numerator or denominator. Exponents MUST be integers. When
// there are no named units in the numerator (e.g., units of “per second”),
// the integer one (1), or one of its aliases as described below, MUST be used
// as a placeholder for the numerator (e.g., 1/seconds). The integer one (1)
// MAY be used at any time to represent the identity element for units and both
// Dimensionless and Dmnl are RECOMMENDED as built-in aliases for this.
//
// Units appearing in the unit equation MAY also be defined in terms of other
// units. For example, Square Miles would have the equation Miles^2. When a
// unit is defined in this way, any use of it is the equivalent of using its
// defining equation. Units with no defining equation are called primary units.
// Every unit equation can be reduced to an expression involving only primary
// units by the process of substitution.

// Unit aliases allow multiple names to have the same meaning. For example,
// People, Person, and Persons could all be considered to be the same. When a
// unit has an alias, that unit's name or any of its aliases MAY be used
// interchangeably for the purpose of specifying the units equation. Aliases
// are actually a special case of units being defined by other units, but
// allowing multiple aliases simplifies the way that information about units is
// kept. Unit aliases may be chained by specifying the name of an existing unit
// (or one of its aliases) as the equation. This allows the addition of user-
// defined aliases for built-in units in a straightforward manner.
//
// A unit is thus specified by a name, an equation, and a sequence of aliases.
// The name and equation are standard XMILE identifiers except that $ is
// allowed as the first (and often only) character in the name of a unit
// without surrounding quotes. Also, the single digit 1 is used as the unit
// identity. Like variables names, unit names are stored with underscores (_)
// but generally presented to users with spaces. A unit with no equation is a
// primary unit. A unit with an equation SHOULD, when possible, be presented to
//  the user with its name rather than its equation.

use std::{cmp::Ordering, fmt};

use serde::{Deserialize, Serialize};

use crate::{Identifier, equation::parse::unit_equation};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnitEquation {
    Integer(i32),
    Alias(Identifier),
    UnaryMinus(Box<UnitEquation>),
    Multiplication(Box<UnitEquation>, Box<UnitEquation>),
    Division(Box<UnitEquation>, Box<UnitEquation>),
    Parentheses(Box<UnitEquation>),
}

impl UnitEquation {
    pub fn integer(value: i32) -> Self {
        UnitEquation::Integer(value)
    }

    pub fn alias(identifier: Identifier) -> Self {
        UnitEquation::Alias(identifier)
    }

    pub fn unary_minus(inner: UnitEquation) -> Self {
        UnitEquation::UnaryMinus(Box::new(inner))
    }

    pub fn multiplication(left: UnitEquation, right: UnitEquation) -> Self {
        UnitEquation::Multiplication(Box::new(left), Box::new(right))
    }

    pub fn division(left: UnitEquation, right: UnitEquation) -> Self {
        UnitEquation::Division(Box::new(left), Box::new(right))
    }

    pub fn parentheses(inner: UnitEquation) -> Self {
        UnitEquation::Parentheses(Box::new(inner))
    }
}

impl<'de> Deserialize<'de> for UnitEquation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize as a string
        let s: String = Deserialize::deserialize(deserializer)?;

        // Parse the string into an unit equation
        let (output, equation) = unit_equation(&s).map_err(serde::de::Error::custom)?;

        // Ensure the entire string was consumed
        if !output.is_empty() {
            return Err(serde::de::Error::custom(format!(
                "Unexpected trailing characters after equation: '{}'",
                output
            )));
        }

        // Return the parsed equation
        Ok(equation)
    }
}

impl Serialize for UnitEquation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize the unit equation as a string
        let expr_str = self.to_string();
        serializer.serialize_str(&expr_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitOfMeasure {
    pub name: Identifier,
    pub equation: Option<UnitEquation>,
    pub aliases: Vec<Identifier>,
}

impl fmt::Display for UnitEquation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnitEquation::Integer(i) => write!(f, "{}", i),
            UnitEquation::Alias(alias) => write!(f, "{}", alias),
            UnitEquation::UnaryMinus(inner) => write!(f, "-({})", inner),
            UnitEquation::Multiplication(left, right) => write!(f, "{} * {}", left, right),
            UnitEquation::Division(left, right) => write!(f, "{}/{}", left, right),
            UnitEquation::Parentheses(inner) => write!(f, "({})", inner),
        }
    }
}

impl PartialOrd for UnitOfMeasure {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnitOfMeasure {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

pub trait Measure {
    fn units(&self) -> Option<&UnitEquation>;
}

pub mod baseline {

    // | Name         | Equation             | Aliases                       |
    // |--------------|----------------------|-------------------------------|
    // | 1            |                      | Dimensionless, Unitless, Dmnl |
    // | nanoseconds  |                      | ns, nanosecond                |
    // | microseconds |                      | us, microsecond               |
    // | milliseconds |                      | ms, milliseconds              |
    // | seconds      |                      | s, second                     |
    // | per_second   | 1/seconds            |                               |
    // | minutes      |                      | min, minute                   |
    // | per_minute   | 1/minutes            |                               |
    // | hours        |                      | hr, hour                      |
    // | per_hour     | 1/hours              |                               |
    // | days         |                      | day                           |
    // | per_day      | 1/days               |                               |
    // | weeks        |                      | wk, week                      |
    // | per_week     | 1/weeks              |                               |
    // | months       |                      | mo, month                     |
    // | per_month    | 1/months             |                               |
    // | quarters     |                      | qtr, quarter                  |
    // | per_quarter  | 1/quarters           |                               |
    // | years        |                      | yr, year                      |
    // | per_year     | 1/years              |                               |

    use crate::{Identifier, UnitEquation, UnitOfMeasure};

    pub fn baseline_units() -> Vec<UnitOfMeasure> {
        vec![
            _dimensionless_unit(),
            _nanoseconds_unit(),
            _microseconds_unit(),
            _milliseconds_unit(),
            _seconds_unit(),
            _per_second_unit(),
            _minutes_unit(),
            _per_minute_unit(),
            _hours_unit(),
            _per_hour_unit(),
            _days_unit(),
            _per_day_unit(),
            _weeks_unit(),
            _per_week_unit(),
            _months_unit(),
            _per_month_unit(),
            _quarters_unit(),
            _per_quarter_unit(),
            _years_unit(),
            _per_year_unit(),
        ]
    }

    fn _ident<S>(name: S) -> Identifier
    where
        S: AsRef<str>,
    {
        Identifier::parse_unit_name(name.as_ref()).expect("Invalid identifier")
    }

    fn _idents<S>(names: &[S]) -> Vec<Identifier>
    where
        S: AsRef<str>,
    {
        names.iter().map(_ident).collect()
    }

    fn _dimensionless_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("1"),
            equation: Some(UnitEquation::Integer(1)),
            aliases: _idents(&["Dimensionless", "Unitless", "Dmnl"]),
        }
    }

    fn _nanoseconds_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("nanoseconds"),
            equation: None,
            aliases: _idents(&["ns", "nanosecond"]),
        }
    }

    fn _microseconds_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("microseconds"),
            equation: None,
            aliases: _idents(&["us", "microsecond"]),
        }
    }

    fn _milliseconds_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("milliseconds"),
            equation: None,
            aliases: _idents(&["ms", "millisecond"]),
        }
    }

    fn _seconds_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("seconds"),
            equation: None,
            aliases: _idents(&["s", "second"]),
        }
    }

    fn _per_second_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("per_second"),
            equation: Some(UnitEquation::Division(
                Box::new(UnitEquation::Integer(1)),
                Box::new(UnitEquation::Alias(_ident("seconds"))),
            )),
            aliases: vec![],
        }
    }

    fn _minutes_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("minutes"),
            equation: None,
            aliases: _idents(&["min", "minute"]),
        }
    }

    fn _per_minute_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("per_minute"),
            equation: Some(UnitEquation::Division(
                Box::new(UnitEquation::Integer(1)),
                Box::new(UnitEquation::Alias(_ident("minutes"))),
            )),
            aliases: vec![],
        }
    }

    fn _hours_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("hours"),
            equation: None,
            aliases: _idents(&["hr", "hour"]),
        }
    }

    fn _per_hour_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("per_hour"),
            equation: Some(UnitEquation::Division(
                Box::new(UnitEquation::Integer(1)),
                Box::new(UnitEquation::Alias(_ident("hours"))),
            )),
            aliases: vec![],
        }
    }

    fn _days_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("days"),
            equation: None,
            aliases: vec![_ident("day")],
        }
    }

    fn _per_day_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("per_day"),
            equation: Some(UnitEquation::Division(
                Box::new(UnitEquation::Integer(1)),
                Box::new(UnitEquation::Alias(_ident("days"))),
            )),
            aliases: vec![],
        }
    }

    fn _weeks_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("weeks"),
            equation: None,
            aliases: _idents(&["wk", "week"]),
        }
    }

    fn _per_week_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("per_week"),
            equation: Some(UnitEquation::Division(
                Box::new(UnitEquation::Integer(1)),
                Box::new(UnitEquation::Alias(_ident("weeks"))),
            )),
            aliases: vec![],
        }
    }

    fn _months_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("months"),
            equation: None,
            aliases: _idents(&["mo", "month"]),
        }
    }

    fn _per_month_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("per_month"),
            equation: Some(UnitEquation::Division(
                Box::new(UnitEquation::Integer(1)),
                Box::new(UnitEquation::Alias(_ident("months"))),
            )),
            aliases: vec![],
        }
    }

    fn _quarters_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("quarters"),
            equation: None,
            aliases: _idents(&["qtr", "quarter"]),
        }
    }

    fn _per_quarter_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("per_quarter"),
            equation: Some(UnitEquation::Division(
                Box::new(UnitEquation::Integer(1)),
                Box::new(UnitEquation::Alias(_ident("quarters"))),
            )),
            aliases: vec![],
        }
    }

    fn _years_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("years"),
            equation: None,
            aliases: _idents(&["yr", "year"]),
        }
    }

    fn _per_year_unit() -> UnitOfMeasure {
        UnitOfMeasure {
            name: _ident("per_year"),
            equation: Some(UnitEquation::Division(
                Box::new(UnitEquation::Integer(1)),
                Box::new(UnitEquation::Alias(_ident("years"))),
            )),
            aliases: vec![],
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_baseline_units() {
            baseline_units();
        }
    }
}
