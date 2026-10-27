use core::fmt;

/// Matches a number from the start of the input up until it stops
fn match_number(input: &String) -> Result<Option<usize>, CalcError> {
    if input.len() == 0 {
        return Ok(None)
    }
    let mut i = 0;
    let mut has_sign = false;
    let mut found_digit = false;
    while i < input.len() {
        let c = input.chars().nth(i).ok_or(CalcError::UnexpectedParsingError)?;
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
fn match_operator(input: &String) -> Result<bool, CalcError> {
    if input.len() == 0 {
        return Ok(false)
    }
    return Ok(OPERATOR_CHARS.contains(&input.chars().nth(0).ok_or(CalcError::UnexpectedParsingError)?));
}

/// Matches a groupped expression (parentheses) at the beginning of the input until it ends
fn match_group(input: &String) -> Result<Option<usize>, CalcError> {
    if input.len() == 0 {
        return Ok(None)
    }
    let c = input.chars().nth(0).ok_or(CalcError::UnexpectedParsingError)?;
    // if we don't start with a group, then stop
    if c != '(' {
        return Ok(None)
    }
    
    // Handle nested parentheses
    let mut i = 0;
    let mut depth = 0;
    while i < input.len() {
        if input.chars().nth(i) == Some('(') {
            depth+=1;
        }
        if input.chars().nth(i) == Some(')') {
            depth-=1;
            if depth == 0 {
                break;
            }
        }
        i+=1;
    }

    if depth != 0 {
        println!("Reached end with unfinished parentheses");
        println!("depth of {}", depth);
        println!("in input: {}", input);
        return Err(CalcError::MismatchedParentheses);
    }
    
    return Ok(Some(i));
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

    while i < input.len() {
        let c = input.chars().nth(i).ok_or(CalcError::UnexpectedParsingError)?;
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

#[derive(Clone)]
pub enum Token {
    Value(f64),
    Operator(Operator),
    Group(Vec<Token>)
}

impl Token {
    pub fn is_value(&self) -> bool {
        match self {
            Self::Value(_) => true,
            _ => false
        }
    }

    pub fn is_group(&self) -> bool {
        match self {
            Self::Group(_) => true,
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
            Self::Group(toks) => {
                write!(f, "GRP[{:?}]", toks)
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

pub struct ExprGroup {
    contents_consumed: String,
    parsed_content: Vec<Token>
}


// What we should have is a struct S that holds an operator and two other S  
#[derive(Clone, Debug)]
pub struct OperationTree {
    content: OperationContent,
    child_a: Option<Box<OperationTree>>,
    child_b: Option<Box<OperationTree>>
}

impl OperationTree {
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

impl ExprGroup {
    /// Create a new Expression Group (a unit of parseable and parsed input)
    pub fn new(contents_raw: String) -> Self {
        Self {
            contents_consumed: contents_raw,
            parsed_content: Vec::new()
        }
    }

    fn with_parsed_content(parsed_content: Vec<Token>) -> Self {
        Self {
            contents_consumed: String::new(),
            parsed_content
        }
    }

    /// Parse the expression inside this ExprGroup into a Group token containing subtokens
    pub fn parse(&mut self) -> Result<Token, CalcError> {
        // remove all whitespaces (spaces, tabs, etc...) from input
        self.contents_consumed = self.contents_consumed.chars().filter(|c| !c.is_whitespace()).collect();
        
        let mut last_token_type_was_num = false;
        let mut last_token_type_was_grp = false;
        // Match in order for groups, numbers then operators extracting a list of tokens from the raw 
        while self.contents_consumed.len() > 0 {
            match match_group(&self.contents_consumed)? {
                Some(grp_end) => {
                    let mut subgroup_raw_contents = self.contents_consumed.clone();
                    self.contents_consumed = subgroup_raw_contents.split_off(grp_end); // cut the end (there's still a ')' left we trim it later)
                    subgroup_raw_contents = subgroup_raw_contents.split_off(1); // remove first parenthesis
                    self.contents_consumed = self.contents_consumed.split_off(1); // trim off the remaining ')'
                    self.parsed_content.push(ExprGroup::new(subgroup_raw_contents).parse()?);
                    last_token_type_was_num = false;
                    last_token_type_was_grp = true;
                    continue;
                },
                None => {}
            }

            match match_number(&self.contents_consumed)? {
                Some(num_end) => {
                    // If last token is a number, then the + or - in front of what could be a number is actually an operator!
                    if !last_token_type_was_num && !(last_token_type_was_grp && (self.contents_consumed.starts_with("+") || self.contents_consumed.starts_with("-")))
                    {
                        let raw_num = &self.contents_consumed[..num_end];
                        self.parsed_content.push(Token::Value(parse_number(raw_num)?.ok_or(CalcError::UndefinedError)?));
                        self.contents_consumed = self.contents_consumed[num_end..].to_string();
                        last_token_type_was_num = true;
                        last_token_type_was_grp = false;
                        continue;
                    }
                },
                None => {}
            }

            if match_operator(&self.contents_consumed)? {
                let raw_op = &self.contents_consumed[..1];
                self.parsed_content.push(Token::Operator(parse_operator(raw_op)?));
                self.contents_consumed = self.contents_consumed[1..].to_string();
                last_token_type_was_num = false;
                last_token_type_was_grp = false;
                continue;
            }

            break;
        }

        // post-processing to find implied operations
        let mut insertions_to_perform: Vec<(usize, Token)> = Vec::new();
        for (i, t) in self.parsed_content.iter().enumerate() {
            if i == 0 {continue};
            let prev_tok = self.parsed_content.get(i-1).ok_or(CalcError::UnexpectedParsingError)?;
            if (prev_tok.is_group() || prev_tok.is_value()) && !t.is_operator() {
                insertions_to_perform.push((i, Token::Operator(Operator::Mul)));
            }
        }

        // Store the offset created by the insertions
        let mut offset = 0;
        for (i, t) in insertions_to_perform {
            self.parsed_content.insert(i+offset, t);
            offset+=1;
        }

        Ok(Token::Group(self.parsed_content.clone()))
    }

    pub fn into_operation_tree(&self) -> Result<OperationTree, CalcError> {
        let mut operations: Vec<OperationTree> = Vec::new();
        // First we convert all tokens to OperationTree singleton elements
        for t in &self.parsed_content {
            operations.push(match t {
                Token::Operator(op) => OperationTree { content: OperationContent::Operator(*op), child_a: None, child_b: None },
                Token::Value(v) => OperationTree { content: OperationContent::Value(*v), child_a: None, child_b: None },
                Token::Group(grp) =>   ExprGroup::with_parsed_content(grp.to_vec()).into_operation_tree()?
            });
        };

        // Then, until we have merged everything into a single tree
        while operations.len() > 1 {
            // First we find the biggest operator yet
            let mut biggest_op: Operator = Operator::NoOp;
            let mut op_position = 0;
            let mut i = 0;
            for operation in &operations {
                if operation.child_a.is_some() && operation.child_b.is_some() { 
                    i+=1;
                    continue;
                 }
                if let OperationContent::Operator(operator) = &operation.content {
                    if operator.priority() > biggest_op.priority() {
                        biggest_op = *operator;
                        op_position = i;
                    }
                }
                i+=1;
            }

            if op_position == 0 || op_position >= operations.len()-1 {
                return Err(CalcError::MisplacedOperator(biggest_op))
            }
            // Then we merge that operator with its two operands
            let op_a = operations.remove(op_position-1); //.ok_or(CalcError::MisplacedOperator(biggest_op))?;
            let op_b = operations.remove(op_position); // not op_position+1 because the index shifted //.ok_or(CalcError::MisplacedOperator(biggest_op))?;
            let operation = operations.get_mut(op_position-1).ok_or(CalcError::UndefinedError)?;
            operation.child_a = Some(Box::new(op_a));
            operation.child_b = Some(Box::new(op_b));
        };

        let final_operation = operations.first().ok_or(CalcError::UnexpectedParsingError)?.clone();
        return Ok(final_operation)
    }
}


pub fn run_calculation(input: String) -> Result<f64, CalcError>{
    let mut root_group = ExprGroup::new(input);
    let _tokens = root_group.parse()?;
    let optree = root_group.into_operation_tree()?;
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
    use crate::calc::{Operator, parse_number, parse_operator};

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

mod test_matcher {

}