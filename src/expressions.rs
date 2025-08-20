use core::{
    convert::TryFrom,
    error::Error,
    fmt::{self, Display},
};

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
                Some(Ok(And))
            }
            '|' => {
                self.position += 1;
                Some(Ok(Or))
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
            && self.input.as_bytes()[self.position] as char == ' '
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
        while let Some(op) = self.match_one_of(&[Token::Eq, Token::Neq]) {
            let right = self.parse_relational()?;
            let bop = match op {
                Token::Eq => BinaryOperator::Eq,
                Token::Neq => BinaryOperator::Neq,
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

    fn parse_relational(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_term()?;
        while let Some(op) = self.match_one_of(&[Token::Lt, Token::Le, Token::Gt, Token::Ge]) {
            let right = self.parse_term()?;
            let bop = match op {
                Token::Lt => BinaryOperator::Lt,
                Token::Le => BinaryOperator::Le,
                Token::Gt => BinaryOperator::Gt,
                Token::Ge => BinaryOperator::Ge,
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

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_factor()?;
        while let Some(op) = self.match_one_of(&[Token::Plus, Token::Minus]) {
            let right = self.parse_factor()?;
            let bop = match op {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Sub,
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
        while let Some(op) = self.match_one_of(&[Token::Asterisk, Token::Solidus]) {
            let right = self.parse_unary()?;
            let bop = match op {
                Token::Asterisk => BinaryOperator::Mul,
                Token::Solidus => BinaryOperator::Div,
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
                _ => Err(ParseError::UnexpectedToken(format!("{:?}", token))),
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

    fn match_one_of(&mut self, options: &[Token]) -> Option<Token> {
        if let Some(t) = self.tokens.get(self.position) {
            for option in options {
                if t == option {
                    self.position += 1;
                    return Some(option.clone());
                }
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Expression {
    Name(String),
    Number(f64),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Boolean(bool),
    Number(f64),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseError {
    EmptyInput,
    InvalidNumber(String),
    UnexpectedEoi,
    UnexpectedToken(String),
    UnmatchedParenthesis,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error occurred")
    }
}

impl Error for ParseError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvalError {
    DivisionByZero,
    TypeMismatch,
    UndefinedVariable(String),
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "eval error occurred")
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
            Name(_) | Number(_) => self.clone(),

            Unary {
                operator,
                expression,
            } => {
                let reduced = expression.reduce();
                match (&operator, &reduced) {
                    (UnaryOperator::Negate, Number(n)) => Number(-n),
                    (UnaryOperator::Not, Number(n)) => Number(if *n == 0.0 { 1.0 } else { 0.0 }),
                    (
                        UnaryOperator::Not,
                        Unary {
                            operator: UnaryOperator::Not,
                            expression: inner,
                        },
                    ) => *inner.clone(),
                    _ => Unary {
                        operator: operator.clone(),
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

                match (&left, &right, operator) {
                    (Number(l), Number(r), op) => {
                        let folded = match op {
                            BinaryOperator::Add => Number(l + r),
                            BinaryOperator::Sub => Number(l - r),
                            BinaryOperator::Mul => Number(l * r),
                            BinaryOperator::Div => Number(l / r),
                            BinaryOperator::Eq => Number((*l == *r) as u8 as f64),
                            BinaryOperator::Neq => Number((*l != *r) as u8 as f64),
                            BinaryOperator::Lt => Number((*l < *r) as u8 as f64),
                            BinaryOperator::Le => Number((*l <= *r) as u8 as f64),
                            BinaryOperator::Gt => Number((*l > *r) as u8 as f64),
                            BinaryOperator::Ge => Number((*l >= *r) as u8 as f64),
                            BinaryOperator::And => {
                                Number(((*l != 0.0) && (*r != 0.0)) as u8 as f64)
                            }
                            BinaryOperator::Or => Number(((*l != 0.0) || (*r != 0.0)) as u8 as f64),
                        };
                        folded
                    }
                    (Number(n), _, BinaryOperator::And) if *n == 0.0 => Number(0.0),
                    (Number(n), _, BinaryOperator::Or) if *n != 0.0 => Number(1.0),
                    (_, Number(n), BinaryOperator::And) if *n == 0.0 => Number(0.0),
                    (_, Number(n), BinaryOperator::Or) if *n != 0.0 => Number(1.0),
                    _ => Binary {
                        left: Box::new(left),
                        operator: operator.clone(),
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

            Unary {
                operator: op,
                expression,
            } => {
                let val = expression.evaluate(name_to_value)?;
                match (op, val) {
                    (Negate, Value::Number(n)) => Ok(Value::Number(-n)),
                    (Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
                    (Not, Value::Number(n)) => Ok(Value::Boolean(n == 0.0)),
                    _ => Err(EvalError::TypeMismatch),
                }
            }

            Binary {
                left,
                operator: op,
                right,
            } => {
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
                    (Value::Boolean(a), Value::Boolean(b), And) => Ok(Value::Boolean(a && b)),
                    (Value::Boolean(a), Value::Boolean(b), Or) => Ok(Value::Boolean(a || b)),
                    (Value::Number(a), Value::Number(b), And) => {
                        Ok(Value::Boolean((a != 0.0) && (b != 0.0)))
                    }
                    (Value::Number(a), Value::Number(b), Or) => {
                        Ok(Value::Boolean((a != 0.0) || (b != 0.0)))
                    }
                    _ => Err(EvalError::TypeMismatch),
                }
            }
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
            return Err(ParseError::UnexpectedToken(format!(
                "{:?}",
                parser.tokens[parser.position]
            )));
        }
        Ok(expression)
    }
}
