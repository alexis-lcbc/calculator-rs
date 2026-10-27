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
                    println!("malformed number in {} near {} ('{}')", input, i, c);
                    return Err(CalcError::MalformedNumber(input.to_string()))
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


fn parse_number(input: &String) -> Result<Option<f64>, CalcError> {
    // Greedily match from the beginning up to the first non-number token
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

fn parse_operator(input: &String) -> Result<Operator, CalcError> {
    Ok(    
        match input.as_str() {
            "+" => Operator::Add,
            "-" => Operator::Sub,
            "*" => Operator::Mul,
            "/" => Operator::Div,
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

const OPERATOR_CHARS: [char; 4] = [
    '/',
    '*',
    '-',
    '+',
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

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub enum Operator {
    NoOp,
    Add,
    Sub,
    Mul,
    Div
}

impl core::cmp::Ord for Operator {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self {
            Operator::Add | Operator::Sub => {
                match other {
                    Operator::NoOp => core::cmp::Ordering::Less,
                    Operator::Add | Operator::Sub => core::cmp::Ordering::Equal,
                    Operator::Mul | Operator::Div => core::cmp::Ordering::Less,
                }
            },
            Operator::Mul | Operator::Div => {
                match other {
                    Operator::NoOp => core::cmp::Ordering::Less,
                    Operator::Add | Operator::Sub => core::cmp::Ordering::Greater,
                    Operator::Mul | Operator::Div => core::cmp::Ordering::Equal,
                }
            },
            Operator::NoOp => core::cmp::Ordering::Less
        }
    }
}


impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::NoOp => write!(f, " ")
        }
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
            OperationContent::Value(v) => Ok(v.clone()),
            OperationContent::Operator(op) => {
                    let v_a = self.child_a.as_ref().ok_or(CalcError::MissingOperationChild(op.clone()))?.evaluate()?;
                    let v_b = self.child_b.as_ref().ok_or(CalcError::MissingOperationChild(op.clone()))?.evaluate()?;
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
                        Operator::NoOp => Err(CalcError::EvaluationError)
                    }
            }
        }
    }
}

impl ExprGroup {
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


    pub fn parse(&mut self) -> Result<Token, CalcError> {
        let mut last_token_type_was_num = false;
        let mut last_token_type_was_grp = false;
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
                        let mut raw_num = self.contents_consumed.clone();
                        self.contents_consumed = raw_num.split_off(num_end);
                        self.parsed_content.push(Token::Value(parse_number(&raw_num)?.ok_or(CalcError::UndefinedError)?));
                        last_token_type_was_num = true;
                        last_token_type_was_grp = false;
                        continue;
                    }
                },
                None => {}
            }

            if match_operator(&self.contents_consumed)? {
                let mut raw_op = self.contents_consumed.clone();
                self.contents_consumed = raw_op.split_off(1);
                self.parsed_content.push(Token::Operator(parse_operator(&raw_op)?));
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
        for (i, t) in insertions_to_perform {
            self.parsed_content.insert(i, t);
        }

        Ok(Token::Group(self.parsed_content.clone()))
    }

    pub fn into_operation_tree(&self) -> Result<OperationTree, CalcError> {
        let mut operations: Vec<OperationTree> = Vec::new();
        // First we convert all tokens to OperationTree singleton elements
        for t in &self.parsed_content {
            operations.push(match t {
                Token::Operator(op) => OperationTree { content: OperationContent::Operator(op.clone()), child_a: None, child_b: None },
                Token::Value(v) => OperationTree { content: OperationContent::Value(v.clone()), child_a: None, child_b: None },
                Token::Group(grp) =>   ExprGroup::with_parsed_content(grp.to_vec()).into_operation_tree()?
            });
        };

        // Then, until we have merged everything into a single tree
        while operations.len() > 1 {
            // First we find the biggest operator yet
            let mut biggest_op: &Operator = &Operator::NoOp;
            let mut op_position = 0;
            let mut i = 0;
            for operation in &operations {
                if operation.child_a.is_some() && operation.child_b.is_some() { 
                    i+=1;
                    continue;
                 }
                if let OperationContent::Operator(operator) = &operation.content {
                    if operator > biggest_op {
                        biggest_op = operator;
                        op_position = i;
                    }
                }
                i+=1;
            }
            if op_position == 0 || op_position >= operations.len()-1 {
                return Err(CalcError::MisplacedOperator(biggest_op.clone()))
            }
            // Then we merge that operator with its two operands
            let op_a = operations.get(op_position-1).ok_or(CalcError::MisplacedOperator(biggest_op.clone()))?.clone();
            let op_b = operations.get(op_position+1).ok_or(CalcError::MisplacedOperator(biggest_op.clone()))?.clone();
            let operation = operations.get_mut(op_position).ok_or(CalcError::UndefinedError)?;
            operation.child_a = Some(Box::new(op_a.clone()));
            operation.child_b = Some(Box::new(op_b.clone()));
            operations.remove(op_position-1);
            operations.remove(op_position);
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

