use core::{
    convert::TryFrom,
    error::Error,
    fmt::{self, Display},
};

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Identifier(String),
    Plus,
    Minus,
    Asterisk,
    Solidus,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    LeftParenthesis,
    RightParenthesis,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Token::*;
        match self {
            Number(n) => write!(f, "number({})", n),
            Identifier(s) => write!(f, "identifier({})", s),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Asterisk => write!(f, "*"),
            Solidus => write!(f, "/"),
            Eq => write!(f, "=="),
            Neq => write!(f, "!="),
            Lt => write!(f, "<"),
            Le => write!(f, "<="),
            Gt => write!(f, ">"),
            Ge => write!(f, ">="),
            And => write!(f, "&&"),
            Or => write!(f, "||"),
            Not => write!(f, "!"),
            LeftParenthesis => write!(f, "("),
            RightParenthesis => write!(f, ")"),
        }
    }
}

struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn next(&mut self) -> Option<Result<Token, ParseError>> {
        use Token::*;
        self.skip_whitespace();
        let slice = self.input.as_bytes();
        if self.position >= slice.len() {
            return None;
        }
        match slice[self.position] as char {
            '0'..='9' | '.' => Some(self.read_number()),
            'A'..='Z' | '_' | 'a'..='z' => Some(self.read_identifier()),
            '+' => {
                self.position += 1;
                Some(Ok(Plus))
            }
            '-' => {
                self.position += 1;
                Some(Ok(Minus))
            }
            '*' => {
                self.position += 1;
                Some(Ok(Asterisk))
            }
            '/' => {
                self.position += 1;
                Some(Ok(Solidus))
            }
            '!' => {
                self.position += 1;
                if self.consume('=') {
                    Some(Ok(Neq))
                } else {
                    Some(Ok(Not))
                }
            }
            '=' => {
                self.position += 1;
                if self.consume('=') {
                    Some(Ok(Eq))
                } else {
                    Some(Err(ParseError::UnexpectedToken("=".to_string())))
                }
            }
            '<' => {
                self.position += 1;
                if self.consume('=') {
                    Some(Ok(Le))
                } else {
                    Some(Ok(Lt))
                }
            }
            '>' => {
                self.position += 1;
                if self.consume('=') {
                    Some(Ok(Ge))
                } else {
                    Some(Ok(Gt))
                }
            }
            '&' => {
                self.position += 1;
                if self.consume('&') {
                    Some(Ok(And))
                } else {
                    Some(Err(ParseError::UnexpectedToken("&".to_string())))
                }
            }
            '|' => {
                self.position += 1;
                if self.consume('|') {
                    Some(Ok(Or))
                } else {
                    Some(Err(ParseError::UnexpectedToken("|".to_string())))
                }
            }
            '(' => {
                self.position += 1;
                Some(Ok(LeftParenthesis))
            }
            ')' => {
                self.position += 1;
                Some(Ok(RightParenthesis))
            }
            ch => Some(Err(ParseError::UnexpectedToken(ch.to_string()))),
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.position < self.input.len()
            && self.input.as_bytes()[self.position] as char == expected
        {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn read_number(&mut self) -> Result<Token, ParseError> {
        let start = self.position;
        while self.position < self.input.len()
            && self.input.as_bytes()[self.position].is_ascii_digit()
        {
            self.position += 1;
        }
        if self.position < self.input.len() && self.input.as_bytes()[self.position] == b'.' {
            self.position += 1;
            while self.position < self.input.len()
                && self.input.as_bytes()[self.position].is_ascii_digit()
            {
                self.position += 1;
            }
        }
        let slice = &self.input[start..self.position];
        match slice.parse::<f64>() {
            Ok(num) => Ok(Token::Number(num)),
            Err(_) => Err(ParseError::InvalidNumber(slice.to_string())),
        }
    }

    fn read_identifier(&mut self) -> Result<Token, ParseError> {
        let start = self.position;
        while self.position < self.input.len()
            && (self.input.as_bytes()[self.position].is_ascii_alphanumeric()
                || self.input.as_bytes()[self.position] == b'_')
        {
            self.position += 1;
        }
        Ok(Token::Identifier(
            self.input[start..self.position].to_string(),
        ))
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len()
            && (self.input.as_bytes()[self.position] as char).is_ascii_whitespace()
        {
            self.position += 1;
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_and()?;
        while self.match_token(&Token::Or) {
            let right = self.parse_and()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_equality()?;
        while self.match_token(&Token::And) {
            let right = self.parse_equality()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_relational()?;
        if let Some(idx) = self.match_one_of(&[Token::Eq, Token::Neq]) {
            let right = self.parse_relational()?;
            let bop = match idx {
                0 => BinaryOperator::Eq,
                1 => BinaryOperator::Neq,
                _ => unreachable!(),
            };
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: bop,
                right: Box::new(right),
            };
            if self.peek_one_of(&[Token::Eq, Token::Neq]).is_some() {
                return Err(ParseError::ChainedNonAssociative("equality (==, !=)"));
            }
        }
        Ok(expression)
    }

    fn parse_relational(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_term()?;
        if let Some(idx) = self.match_one_of(&[Token::Lt, Token::Le, Token::Gt, Token::Ge]) {
            let right = self.parse_term()?;
            let bop = match idx {
                0 => BinaryOperator::Lt,
                1 => BinaryOperator::Le,
                2 => BinaryOperator::Gt,
                3 => BinaryOperator::Ge,
                _ => unreachable!(),
            };
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: bop,
                right: Box::new(right),
            };
            if self
                .peek_one_of(&[Token::Lt, Token::Le, Token::Gt, Token::Ge])
                .is_some()
            {
                return Err(ParseError::ChainedNonAssociative(
                    "relational (<, <=, >, >=)",
                ));
            }
        }
        Ok(expression)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_factor()?;
        while let Some(idx) = self.match_one_of(&[Token::Plus, Token::Minus]) {
            let right = self.parse_factor()?;
            let bop = match idx {
                0 => BinaryOperator::Add,
                1 => BinaryOperator::Sub,
                _ => unreachable!(),
            };
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: bop,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_unary()?;
        while let Some(idx) = self.match_one_of(&[Token::Asterisk, Token::Solidus]) {
            let right = self.parse_unary()?;
            let bop = match idx {
                0 => BinaryOperator::Mul,
                1 => BinaryOperator::Div,
                _ => unreachable!(),
            };
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: bop,
                right: Box::new(right),
            };
        }
        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&Token::Minus) {
            let expression = self.parse_unary()?;
            Ok(Expression::Unary {
                operator: UnaryOperator::Negate,
                expression: Box::new(expression),
            })
        } else if self.match_token(&Token::Not) {
            let expression = self.parse_unary()?;
            Ok(Expression::Unary {
                operator: UnaryOperator::Not,
                expression: Box::new(expression),
            })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        if let Some(token) = self.tokens.get(self.position).cloned() {
            match token {
                Token::Number(n) => {
                    self.position += 1;
                    Ok(Expression::Number(n))
                }
                Token::Identifier(s) => {
                    self.position += 1;
                    Ok(Expression::Name(s))
                }
                Token::LeftParenthesis => {
                    self.position += 1;
                    let expression = self.parse_expression()?;
                    if self.match_token(&Token::RightParenthesis) {
                        Ok(expression)
                    } else {
                        Err(ParseError::UnmatchedParenthesis)
                    }
                }
                _ => Err(ParseError::UnexpectedToken(token.to_string())),
            }
        } else {
            Err(ParseError::UnexpectedEoi)
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if let Some(t) = self.tokens.get(self.position) {
            if t == token {
                self.position += 1;
                return true;
            }
        }
        false
    }

    fn match_one_of(&mut self, options: &[Token]) -> Option<usize> {
        if let Some(t) = self.tokens.get(self.position) {
            for (i, option) in options.iter().enumerate() {
                if t == option {
                    self.position += 1;
                    return Some(i);
                }
            }
        }
        None
    }

    fn peek_one_of(&self, options: &[Token]) -> Option<usize> {
        if let Some(t) = self.tokens.get(self.position) {
            for (i, option) in options.iter().enumerate() {
                if t == option {
                    return Some(i);
                }
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Name(String),
    Number(f64),
    Boolean(bool),
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    ChainedNonAssociative(&'static str),
    EmptyInput,
    InvalidNumber(String),
    UnexpectedEoi,
    UnexpectedToken(String),
    UnmatchedParenthesis,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::ChainedNonAssociative(kind) => {
                write!(f, "chained non-associative operator in {kind} expression")
            }
            ParseError::EmptyInput => write!(f, "empty input"),
            ParseError::InvalidNumber(s) => write!(f, "invalid number: '{s}'"),
            ParseError::UnexpectedEoi => write!(f, "unexpected end of input"),
            ParseError::UnexpectedToken(token) => write!(f, "unexpected token: '{token}'"),
            ParseError::UnmatchedParenthesis => write!(f, "unmatched parenthesis"),
        }
    }
}

impl Error for ParseError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalError {
    DivisionByZero,
    TypeMismatch,
    UndefinedVariable(String),
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EvalError::DivisionByZero => write!(f, "division by zero"),
            EvalError::TypeMismatch => write!(f, "type mismatch"),
            EvalError::UndefinedVariable(name) => write!(f, "undefined variable: {}", name),
        }
    }
}

impl Error for EvalError {}

impl Expression {
    pub fn collect_variables(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        self.collect_variables_impl(&mut set);
        set
    }

    fn collect_variables_impl(&self, set: &mut HashSet<String>) {
        use Expression::*;
        match self {
            Name(name) => {
                set.insert(name.clone());
            }
            Number(_) => {}
            Boolean(_) => {}
            Unary { expression, .. } => {
                expression.collect_variables_impl(set);
            }
            Binary { left, right, .. } => {
                left.collect_variables_impl(set);
                right.collect_variables_impl(set);
            }
        }
    }

    pub fn reduce(&self) -> Expression {
        use Expression::*;
        match self {
            Name(_) | Number(_) | Boolean(_) => self.clone(),

            Unary {
                operator,
                expression,
            } => {
                let reduced = expression.reduce();
                match (&operator, &reduced) {
                    (UnaryOperator::Negate, Number(n)) => Number(-n),
                    (UnaryOperator::Not, Number(n)) => Boolean(!truthy(&Value::Number(*n))),
                    (UnaryOperator::Not, Boolean(b)) => Boolean(!b),
                    (
                        UnaryOperator::Not,
                        Unary {
                            operator: UnaryOperator::Not,
                            expression: inner,
                        },
                    ) => *inner.clone(),
                    _ => Unary {
                        operator: *operator,
                        expression: Box::new(reduced),
                    },
                }
            }

            Binary {
                left,
                operator,
                right,
            } => {
                let left = left.reduce();
                let right = right.reduce();
                if *operator == BinaryOperator::Div {
                    if let (Expression::Number(_), Expression::Number(r)) = (&left, &right) {
                        if !r.is_finite() || *r == 0.0 {
                            return Expression::Binary {
                                left: Box::new(left),
                                operator: *operator,
                                right: Box::new(right),
                            };
                        }
                    }
                }
                match (&left, &right, operator) {
                    (Number(l), Number(r), BinaryOperator::Add) => Number(l + r),
                    (Number(l), Number(r), BinaryOperator::Sub) => Number(l - r),
                    (Number(l), Number(r), BinaryOperator::Mul) => Number(l * r),
                    (Number(l), Number(r), BinaryOperator::Div) => Number(l / r),

                    (Number(l), Number(r), BinaryOperator::Eq) => Boolean(*l == *r),
                    (Number(l), Number(r), BinaryOperator::Neq) => Boolean(*l != *r),
                    (Number(l), Number(r), BinaryOperator::Lt) => Boolean(*l < *r),
                    (Number(l), Number(r), BinaryOperator::Le) => Boolean(*l <= *r),
                    (Number(l), Number(r), BinaryOperator::Gt) => Boolean(*l > *r),
                    (Number(l), Number(r), BinaryOperator::Ge) => Boolean(*l >= *r),

                    (Boolean(a), Boolean(b), BinaryOperator::And) => Boolean(*a && *b),
                    (Boolean(a), Boolean(b), BinaryOperator::Or) => Boolean(*a || *b),
                    (Boolean(a), Number(b), BinaryOperator::And) => {
                        Boolean(*a && truthy(&Value::Number(*b)))
                    }
                    (Boolean(a), Number(b), BinaryOperator::Or) => {
                        Boolean(*a || truthy(&Value::Number(*b)))
                    }
                    (Number(a), Boolean(b), BinaryOperator::And) => {
                        Boolean(truthy(&Value::Number(*a)) && *b)
                    }
                    (Number(a), Boolean(b), BinaryOperator::Or) => {
                        Boolean(truthy(&Value::Number(*a)) || *b)
                    }
                    (Number(n), _, BinaryOperator::And) if !truthy(&Value::Number(*n)) => {
                        Boolean(false)
                    }
                    (Number(n), _, BinaryOperator::Or) if truthy(&Value::Number(*n)) => {
                        Boolean(true)
                    }
                    (_, Number(n), BinaryOperator::And) if !truthy(&Value::Number(*n)) => {
                        Boolean(false)
                    }
                    (_, Number(n), BinaryOperator::Or) if truthy(&Value::Number(*n)) => {
                        Boolean(true)
                    }
                    (Boolean(false), _, BinaryOperator::And) => Boolean(false),
                    (Boolean(true), _, BinaryOperator::Or) => Boolean(true),
                    (_, Boolean(false), BinaryOperator::And) => Boolean(false),
                    (_, Boolean(true), BinaryOperator::Or) => Boolean(true),
                    (Number(l), Number(r), BinaryOperator::And) => {
                        Boolean(truthy(&Value::Number(*l)) && truthy(&Value::Number(*r)))
                    }
                    (Number(l), Number(r), BinaryOperator::Or) => {
                        Boolean(truthy(&Value::Number(*l)) || truthy(&Value::Number(*r)))
                    }

                    _ => Binary {
                        left: Box::new(left),
                        operator: *operator,
                        right: Box::new(right),
                    },
                }
            }
        }
    }

    pub fn evaluate(&self, name_to_value: &HashMap<String, Value>) -> Result<Value, EvalError> {
        use BinaryOperator::*;
        use Expression::*;
        use UnaryOperator::*;
        match self {
            Name(s) => name_to_value
                .get(s)
                .cloned()
                .ok_or(EvalError::UndefinedVariable(s.clone())),

            Number(n) => Ok(Value::Number(*n)),

            Boolean(b) => Ok(Value::Boolean(*b)),

            Unary {
                operator: op,
                expression,
            } => {
                let val = expression.evaluate(name_to_value)?;
                match (op, val) {
                    (Negate, Value::Number(n)) => Ok(Value::Number(-n)),
                    (Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
                    (Not, Value::Number(n)) => Ok(Value::Boolean(!truthy(&Value::Number(n)))),
                    _ => Err(EvalError::TypeMismatch),
                }
            }

            Binary {
                left,
                operator: op,
                right,
            } => match op {
                And => {
                    let lv = left.evaluate(name_to_value)?;
                    let lb = truthy(&lv);
                    if !lb {
                        return Ok(Value::Boolean(false));
                    }
                    let rv = right.evaluate(name_to_value)?;
                    Ok(Value::Boolean(lb && truthy(&rv)))
                }
                Or => {
                    let lv = left.evaluate(name_to_value)?;
                    let lb = truthy(&lv);
                    if lb {
                        return Ok(Value::Boolean(true));
                    }
                    let rv = right.evaluate(name_to_value)?;
                    Ok(Value::Boolean(lb || truthy(&rv)))
                }
                Add | Sub | Mul | Div | Eq | Neq | Lt | Le | Gt | Ge => {
                    let l = left.evaluate(name_to_value)?;
                    let r = right.evaluate(name_to_value)?;
                    match (l, r, op) {
                        (Value::Number(a), Value::Number(b), Add) => Ok(Value::Number(a + b)),
                        (Value::Number(a), Value::Number(b), Sub) => Ok(Value::Number(a - b)),
                        (Value::Number(a), Value::Number(b), Mul) => Ok(Value::Number(a * b)),
                        (Value::Number(a), Value::Number(b), Div) => {
                            if b == 0.0 {
                                Err(EvalError::DivisionByZero)
                            } else {
                                Ok(Value::Number(a / b))
                            }
                        }
                        (Value::Number(a), Value::Number(b), Eq) => Ok(Value::Boolean(a == b)),
                        (Value::Number(a), Value::Number(b), Neq) => Ok(Value::Boolean(a != b)),
                        (Value::Number(a), Value::Number(b), Lt) => Ok(Value::Boolean(a < b)),
                        (Value::Number(a), Value::Number(b), Le) => Ok(Value::Boolean(a <= b)),
                        (Value::Number(a), Value::Number(b), Gt) => Ok(Value::Boolean(a > b)),
                        (Value::Number(a), Value::Number(b), Ge) => Ok(Value::Boolean(a >= b)),
                        _ => Err(EvalError::TypeMismatch),
                    }
                }
            },
        }
    }

    pub fn parse(s: &str, defined_variables: &HashSet<String>) -> Result<Self, Box<dyn Error>> {
        Expression::try_from(s)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
            .and_then(|expression| {
                for expression_variable in expression.collect_variables() {
                    if !defined_variables.contains(expression_variable.as_str()) {
                        return Err(Box::new(EvalError::UndefinedVariable(expression_variable))
                            as Box<dyn Error>);
                    }
                }
                Ok(expression)
            })
    }
}

impl TryFrom<&str> for Expression {
    type Error = ParseError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        if input.trim().is_empty() {
            return Err(ParseError::EmptyInput);
        }
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        while let Some(result) = lexer.next() {
            tokens.push(result?);
        }
        let mut parser = Parser::new(tokens);
        let expression = parser.parse_expression()?;
        if parser.position != parser.tokens.len() {
            return Err(ParseError::UnexpectedToken(
                parser.tokens[parser.position].to_string(),
            ));
        }
        Ok(expression)
    }
}

const fn binary_precedence(op: BinaryOperator) -> u8 {
    use BinaryOperator::*;
    match op {
        Or => 1,
        And => 2,
        Eq | Neq => 3,
        Lt | Le | Gt | Ge => 4,
        Add | Sub => 5,
        Mul | Div => 6,
    }
}

const fn unary_precedence(_op: UnaryOperator) -> u8 {
    7
}

const fn is_associative(op: BinaryOperator) -> bool {
    use BinaryOperator::*;
    matches!(op, Add | Mul | And | Or)
}

#[inline]
fn truthy(v: &Value) -> bool {
    match v {
        Value::Boolean(b) => *b,
        Value::Number(n) => n.is_finite() && *n != 0.0,
    }
}

struct Pretty<'a>(&'a Expression);

impl<'a> fmt::Display for Pretty<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Expression::*;
        match self.0 {
            Name(s) => write!(f, "{}", s),

            Number(n) => {
                let n = if *n == 0.0 { 0.0 } else { *n };
                let s = format!("{:.15}", n);
                let s = s.trim_end_matches('0').trim_end_matches('.');
                if s.is_empty() {
                    write!(f, "0")
                } else {
                    write!(f, "{}", s)
                }
            }

            Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),

            Unary {
                operator,
                expression,
            } => {
                let my_prec = unary_precedence(*operator);
                let needs_parentheses = match &**expression {
                    Binary {
                        operator: child_op, ..
                    } => binary_precedence(*child_op) < my_prec,
                    Unary {
                        operator: child_uop,
                        ..
                    } => unary_precedence(*child_uop) < my_prec,
                    _ => false,
                };
                let op_str = match operator {
                    UnaryOperator::Negate => "-",
                    UnaryOperator::Not => "!",
                };
                write!(f, "{}", op_str)?;
                if needs_parentheses {
                    write!(f, "(")?;
                }
                Pretty(expression).fmt(f)?;
                if needs_parentheses {
                    write!(f, ")")?;
                }
                Ok(())
            }

            Binary {
                left,
                operator,
                right,
            } => {
                let my_prec = binary_precedence(*operator);
                let left_needs_parentheses = match &**left {
                    Binary {
                        operator: child_op, ..
                    } => {
                        let child_prec = binary_precedence(*child_op);
                        child_prec < my_prec
                            || (child_prec == my_prec && !is_associative(*operator))
                    }
                    _ => false,
                };
                let right_needs_parentheses = match &**right {
                    Binary {
                        operator: child_op, ..
                    } => {
                        let child_prec = binary_precedence(*child_op);
                        child_prec < my_prec
                            || (child_prec == my_prec && !is_associative(*operator))
                    }
                    _ => false,
                };
                if left_needs_parentheses {
                    write!(f, "(")?;
                }
                Pretty(left).fmt(f)?;
                if left_needs_parentheses {
                    write!(f, ")")?;
                }
                let op_str = match operator {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Sub => "-",
                    BinaryOperator::Mul => "*",
                    BinaryOperator::Div => "/",
                    BinaryOperator::Eq => "==",
                    BinaryOperator::Neq => "!=",
                    BinaryOperator::Lt => "<",
                    BinaryOperator::Le => "<=",
                    BinaryOperator::Gt => ">",
                    BinaryOperator::Ge => ">=",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                };
                write!(f, " {} ", op_str)?;
                if right_needs_parentheses {
                    write!(f, "(")?;
                }
                Pretty(right).fmt(f)?;
                if right_needs_parentheses {
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Pretty(self).fmt(f)
    }
}
