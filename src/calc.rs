use core::fmt;

/// Matches a number from the start of the input up until it stops
fn match_number(input: &str) -> Result<Option<usize>, CalcError> {
    if input.len() == 0 {
        return Ok(None)
    }
    let mut i = 0;
    let mut has_sign = false;
    let mut found_digit = false;
    let mut peekable_chars = input.chars().into_iter();
    while i < input.len() {
        let c = peekable_chars.next().ok_or(CalcError::UnexpectedParsingError)?;
        if DIGITS.contains(&c) {
            found_digit = true;
        }
        // Once we encounter a non-number character
        if !NUM_CHARS.contains(&c) {
            if i == 0 {
                // there is no number here
                return Ok(None)
            } else {
                break;
            }
        }

        // If it's a sign indicator make sure it's in front and we will also check if it has at least one digit afterwards
        if c == '-' {
            if i == 0 {
                has_sign = true;
            } else {
                if !found_digit {
                    return Ok(None)
                    // println!("malformed number in {} near {} ('{}')", input, i, c);
                    // return Err(CalcError::MalformedNumber(input.to_string()))
                }
                return Ok(Some(i)); // The '-' was in the middle ; then we have reached the end of a number and will then get an Operator +
            }
        }
        i+=1;
    }
    // If we don't even have a digit then it's not a number
    if has_sign && i <= 1 {
        return Ok(None);
    }
    // number is in slice [0; i]
    Ok(Some(i))
}

/// Matches an operator at the beginning of the input (this assumes an operator is 1 char exactly)
fn match_operator(input: &str) -> Result<bool, CalcError> {
    if input.len() == 0 {
        return Ok(false)
    }
    return Ok(OPERATOR_CHARS.contains(&input.chars().nth(0).ok_or(CalcError::UnexpectedParsingError)?));
}

fn match_parenthesis(input: &str) -> Result<Option<Token>, CalcError> {
    if input.starts_with('(') {
        return Ok(Some(Token::ParenthesisOpen))
    }
    if input.starts_with(')') {
        return Ok(Some(Token::ParenthesisClose))
    }
    return Ok(None)
}

/// Converts a string to a number
fn parse_number(input: &str) -> Result<Option<f64>, CalcError> {
    // Greedily match from the beginning up to the first non-number token
    if input.len() == 0 {
        return Ok(None)
    }

    let mut is_negative = false;
    let mut in_decimal= false;
    let mut i = 0;
    let mut integer_part: u32 = 0;
    let mut decimal_part: u16 = 0;
    let mut decimal_part_count: u8 = 0;
    let mut peekable_chars = input.chars().into_iter();
    while i < input.len() {
        let c = peekable_chars.next().ok_or(CalcError::UnexpectedParsingError)?;
        // The number given by the tokenizer isn't valid (this is an internal issue!)
        if !NUM_CHARS.contains(&c) {
           return Err(CalcError::UnexpectedNonNumberToken) 
        }

        // if we have a leading minus we know know it's going to be a negative number
        if c == '-' {
            if i == 0 {
                is_negative = true;
                i+=1;
                continue;
            } else {
                break; // If we have met a minus in the middle of a number, then we have to stop here
            }
        }

        if c == '.' {
            // If we already have encountered a decimal divider
            if in_decimal {
                return Err(CalcError::MalformedNumber(input.to_string()))
            }
            in_decimal = true;
            i+=1;
            continue;
        }
        let d = c.to_digit(10).ok_or(CalcError::UnexpectedNonNumberToken)?;
        if in_decimal {
            decimal_part = decimal_part * 10 + d as u16;
            decimal_part_count+=1;
        } else {
            integer_part = integer_part * 10 + d as u32;
        }
        i+=1;
    }

    // Final part of the conversion
    let mut v = (integer_part as f64 + decimal_part as f64 / (10 as f64).powi(decimal_part_count as i32)) as f64;
    if is_negative {
        v *= -1.0;
    }

    return Ok(Some(v));
}

/// Converts a string to an operator
fn parse_operator(input: &str) -> Result<Operator, CalcError> {
    Ok(    
        match input {
            "+" => Operator::Add,
            "-" => Operator::Sub,
            "*" => Operator::Mul,
            "/" => Operator::Div,
            "^" => Operator::Pow,
            _ => return Err(CalcError::InvalidTokenError(input.to_string()))
        }
    )
}

const DIGITS: [char; 10] = [
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9'
];

const NUM_CHARS: [char; 12] = [
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
    '-',
    '.'
];

const OPERATOR_CHARS: [char; 5] = [
    '/',
    '*',
    '-',
    '+',
    '^'
];

#[derive(Clone, PartialEq)]
pub enum Token {
    Value(f64),
    Operator(Operator),
    ParenthesisOpen,
    ParenthesisClose
}

impl Token {
    pub fn is_value(&self) -> bool {
        match self {
            Self::Value(_) => true,
            _ => false
        }
    }

    pub fn is_open_parenthesis(&self) -> bool {
        match self {
            Self::ParenthesisOpen => true,
            _ => false
        }
    }

    pub fn is_close_parenthesis(&self) -> bool {
        match self {
            Self::ParenthesisClose => true,
            _ => false
        }
    }

    pub fn is_operator(&self) -> bool {
        match self {
            Self::Operator(_) => true,
            _ => false
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParenthesisOpen => {
                write!(f, "(")
            },
            Self::ParenthesisClose => {
                write!(f, ")")
            },
            Self::Operator(op) => {
                write!(f, "OP[{:?}]", op)
            },
            Self::Value(val) => {
                write!(f, "{}", val)
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Operator {
    NoOp,
    Add,
    Sub,
    Mul,
    Div,
    Pow
}

impl Operator {
    fn priority(&self) -> u8 {
        match self {
            Operator::NoOp => 0,
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div => 2,
            Operator::Pow => 3
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Pow => write!(f, "^"),
            Self::NoOp => write!(f, "N/A")
        }
    }
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Debug)]
enum OperationContent {
    Operator(Operator),
    Value(f64)
}

#[derive(Debug)]
pub enum CalcError {
    InvalidTokenError(String),
    UnexpectedNonNumberToken,
    MismatchedParentheses,
    UnexpectedParsingError,
    MalformedNumber(String),
    UndefinedError,
    MisplacedOperator(Operator),
    EvaluationError,
    MissingOperationChild(Operator),
    DivisionByZero
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Err: {}", match self {
            Self::InvalidTokenError(token) => format!("Invalid token '{}'", token),
            Self::UnexpectedNonNumberToken => format!("Unexpected non-digit token"),
            Self::MismatchedParentheses => format!("Mismatched parentheses"),
            Self::UnexpectedParsingError => format!("Unexpected parsing error"),
            Self::MalformedNumber(str) => format!("Malformed number {}", str),
            Self::UndefinedError => format!("Undefined error?!"),
            Self::MisplacedOperator(operator) => format!("Misplaced operator {:?}", operator),
            Self::EvaluationError => format!("Error while evaluating the expression"),
            Self::MissingOperationChild(operator) => format!("Missing an operand to operator {:?}", operator),
            Self::DivisionByZero => format!("Attempted to divide by 0")

        })
    }
}

// What we should have is a struct S that holds an operator and two other S  
#[derive(Clone, Debug)]
pub struct OperationTree {
    content: OperationContent,
    child_a: Option<Box<OperationTree>>,
    child_b: Option<Box<OperationTree>>
}

impl OperationTree {
    fn new(content: OperationContent) -> Self {
        return OperationTree { content, child_a: None, child_b: None }
    }

    pub fn evaluate(&self) -> Result<f64, CalcError> {
        match &self.content {
            OperationContent::Value(v) => Ok(*v),
            OperationContent::Operator(op) => {
                    let v_a = self.child_a.as_ref().ok_or(CalcError::MissingOperationChild(*op))?.evaluate()?;
                    let v_b = self.child_b.as_ref().ok_or(CalcError::MissingOperationChild(*op))?.evaluate()?;
                    match op {
                        Operator::Add => Ok(v_a + v_b),
                        Operator::Sub => Ok(v_a - v_b),
                        Operator::Mul => Ok(v_a * v_b),
                        Operator::Div => {
                            if v_b == 0.0 {
                                return Err(CalcError::DivisionByZero)
                            }
                            Ok(v_a / v_b)
                        },
                        Operator::Pow => Ok(v_a.powf(v_b)),
                        Operator::NoOp => Err(CalcError::EvaluationError)
                    }
            }
        }
    }
}


pub fn lexer(mut raw_input: String) -> Result<Vec<Token>, CalcError> {
    let mut last_token_type_was_num = false;
    let mut last_token_type_was_grp = false;
    let mut tokens: Vec<Token> = Vec::new();
    raw_input = raw_input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut input = raw_input.as_mut_str();

    while input.len() > 0 {
        match match_parenthesis(&input)? {
                Some(parenthesis) => {
                    last_token_type_was_num = false;
                    if parenthesis == Token::ParenthesisClose {
                        last_token_type_was_grp = true;
                    }
                    input = &mut input[1..];
                    tokens.push(parenthesis);
                    continue;
                },
                None => {}
            }

            match match_number(&input)? {
                Some(num_end) => {
                    // If last token is a number, then the + or - in front of what could be a number is actually an operator!
                    if !last_token_type_was_num && !(last_token_type_was_grp && (input.starts_with("+") || input.starts_with("-")))
                    {
                        let raw_num = &input[..num_end];
                        tokens.push(Token::Value(parse_number(raw_num)?.ok_or(CalcError::UndefinedError)?));
                        input = &mut input[num_end..];
                        last_token_type_was_num = true;
                        last_token_type_was_grp = false;
                        continue;
                    }
                },
                None => {}
            }

            if match_operator(&input)? {
                let raw_op = &input[..1];
                tokens.push(Token::Operator(parse_operator(raw_op)?));
                input = &mut input[1..];
                last_token_type_was_num = false;
                last_token_type_was_grp = false;
                continue;
            }

            break;
    }

    Ok(tokens)
}

pub fn parse(tokens: &[Token]) -> Result<OperationTree, CalcError> {
    let mut operations: Vec<Option<OperationTree>> = Vec::new();
    let mut parentheses_depth = 0;
    let mut subgroup_start: usize = 0;

    for (i, t) in tokens.iter().enumerate() {
        match t {
            Token::Value(v) => {
                if parentheses_depth != 0 { continue; }
                operations.push(Some(OperationTree::new(OperationContent::Value(*v))))
            },
            Token::Operator(op) => {
                if parentheses_depth != 0 { continue; }
                operations.push(Some(OperationTree::new(OperationContent::Operator(*op))))
            },
            Token::ParenthesisOpen => {
                if parentheses_depth == 0 {
                    subgroup_start = i;
                }
                parentheses_depth += 1;
            },
            Token::ParenthesisClose => {
                parentheses_depth -= 1;
                if parentheses_depth == 0 {
                    let subgroup_optree = parse(&tokens[subgroup_start+1..i])?;
                    operations.push(Some(subgroup_optree));
                }
            }
        }
    }

    // Then, until we have merged everything into a single tree
    let mut left_to_handle = operations.len();
    while left_to_handle > 1 {
        // First we find the biggest operator yet
        let mut biggest_op: Operator = Operator::NoOp;
        let mut op_position = 0;
        for (i, op) in operations.iter().enumerate() {
            if let Some(operation) = op {
                if operation.child_a.is_some() && operation.child_b.is_some() { 
                    continue;
                    }
                if let OperationContent::Operator(operator) = &operation.content {
                    if operator.priority() > biggest_op.priority() {
                        biggest_op = *operator;
                        op_position = i;
                    }
                }
            }
        }

        if op_position == 0 || op_position >= operations.len()-1 {
            return Err(CalcError::MisplacedOperator(biggest_op))
        }
        // Then we merge that operator with its two operands
        let op_left_pos = nearest_some_left(&operations, op_position).ok_or(CalcError::MissingOperationChild(biggest_op))?;
        let op_a = operations.get_mut(op_left_pos).ok_or(CalcError::UnexpectedParsingError)?.take().ok_or(CalcError::MissingOperationChild(biggest_op))?;
        let op_right_pos = nearest_some_right(&operations, op_position).ok_or(CalcError::MissingOperationChild(biggest_op))?;
        let op_b = operations.get_mut(op_right_pos).ok_or(CalcError::UnexpectedParsingError)?.take().ok_or(CalcError::MissingOperationChild(biggest_op))?;
        if let Some(operation) = operations.get_mut(op_position).ok_or(CalcError::UndefinedError)? {
            operation.child_a = Some(Box::new(op_a));
            operation.child_b = Some(Box::new(op_b));
            left_to_handle -= 2;
        }
    };

    operations.retain(|x| x.is_some());
    let final_operation = operations.first().ok_or(CalcError::UnexpectedParsingError)?.clone();
    return Ok(final_operation.ok_or(CalcError::UnexpectedParsingError)?);
}

fn nearest_some_left(vec: &Vec<Option<OperationTree>>, from: usize) -> Option<usize> {
    let mut i = from-1;
    if i == 0 {
        if vec.get(i).is_some() {
            return Some(i)
        } else {
            return None
        }
    }
    while i > 0 {
        if vec.get(i).is_some_and(|x| x.is_some()) {
            return Some(i)
        }
        i-=1;
    }
    return None
}

fn nearest_some_right(vec: &Vec<Option<OperationTree>>, from: usize) -> Option<usize> {
    let mut i = from+1;
    while i < vec.len() {
        if vec.get(i).is_some_and(|x| x.is_some()) {
            return Some(i)
        }
        i+=1;
    }
    return None
}

pub fn run_calculation(input: String) -> Result<f64, CalcError>{
    let tokens = lexer(input)?;
    let optree = parse(&tokens.as_slice())?;
    Ok(optree.evaluate()?)
}


#[cfg(test)]
mod test_parse_and_evaluate {
    use super::*;

    #[test]
    fn test_simple_add() {
        let result = run_calculation("3+4".to_string()).unwrap();
        assert_eq!(result, 7.0);
    }

    #[test]
    fn test_simple_sub() {
        let result = run_calculation("10-5".to_string()).unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_simple_sub_reverse() {
        let result = run_calculation("5-10".to_string()).unwrap();
        assert_eq!(result, -5.0);
    }

    #[test]
    fn test_simple_mul() {
        let result = run_calculation("6*7".to_string()).unwrap();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_simple_div() {
        let result = run_calculation("10/2".to_string()).unwrap();
        assert_eq!(result, 5.0);
    }


    #[test]
    fn test_negative_add() {
        let result = run_calculation("-5+8".to_string()).unwrap();
        assert_eq!(result, 3.0);
    }

    #[test]
    fn test_negative_add_reverse() {
        let result = run_calculation("5+(-8)".to_string()).unwrap();
        assert_eq!(result, -3.0);
    }

    #[test]
    fn test_negative_mul() {
        let result = run_calculation("(-3)*3".to_string()).unwrap();
        assert_eq!(result, -9.0);
    }

    #[test]
    fn test_negative_div() {
        let result = run_calculation("(-20)/2".to_string()).unwrap();
        assert_eq!(result, -10.0);
    }

    #[test]
    fn test_operator_priority() {
        let result = run_calculation("19-8+5*4/2^2".to_string()).unwrap();
        assert_eq!(result, 16.0);
    }

    #[test]
    fn test_complex_expr1() {
        let result = run_calculation("(12.5+3.5)*2-(18/3+4)*(7-2)+10/2".to_string()).unwrap();
        assert_eq!(result, -13.0);
    }

    #[test]
    fn test_complex_expr2() {
        let result = run_calculation("( 15.5 - 3 * ( 2.2 + 1.8 ) ^ 2 + 144 / 12 / 2 - ( -5.5 + 1.5 ) * 3.14159 ^ 0 ) * ( ( 10 - 8 ) ^ 3 / 2 ) + 2.5 * 4 ^ 2.5 / ( 2 ^ 3.5 / 2 ^ 0.5 - 3 )".to_string()).unwrap();
        assert_eq!(result, -74.0);
    }

}


#[cfg(test)]
mod test_parse_tokens {
    use super::{Operator, parse_number, parse_operator};

    #[test]
    fn num_parse_int() {
        assert_eq!(parse_number("16").unwrap().unwrap(), 16.0);
    }

    #[test]
    fn num_parse_negative_int() {
        assert_eq!(parse_number("-6").unwrap().unwrap(), -6.0);
    }

    #[test]
    fn num_parse_float() {
        assert_eq!(parse_number("42.67").unwrap().unwrap(), 42.67);
    }

    #[test]
    fn num_parse_float_negative() {
        assert_eq!(parse_number("-55.56").unwrap().unwrap(), -55.56);
    }

    #[test]
    fn num_fail_parse_notnumber() {
        println!("{:?}", parse_number("a"));
        assert!(parse_number("a").is_err());
    }

    #[test]
    fn num_fail_parse_emptystr() {
        assert!(parse_number("").is_ok_and(|x| x.is_none()));
    }

    #[test]
    fn operator_parse_all() {
        assert!(parse_operator("+").is_ok_and(|x| x == Operator::Add));
        assert!(parse_operator("-").is_ok_and(|x| x == Operator::Sub));
        assert!(parse_operator("*").is_ok_and(|x| x == Operator::Mul));
        assert!(parse_operator("/").is_ok_and(|x| x == Operator::Div));
        assert!(parse_operator("^").is_ok_and(|x| x == Operator::Pow));
    }

    #[test]
    fn operator_parse_fail_notoperator() {
        assert!(parse_operator("1").is_err());
    }
}
