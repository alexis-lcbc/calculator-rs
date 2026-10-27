
fn find_groups(input: &mut String) -> Result<Vec<ExprGroup>, Errors> {
    let mut groups: Vec<ExprGroup> = Vec::new();
    let mut i = 0;
    if input.len() == 0 {
        return Err(Errors::EmptyString)
    }

    while i < input.len()-1 {
        let c = input.chars().nth(i).ok_or(Errors::UnexpectedParsingError)?;
        if !ALLOWED_TOKENS.contains(&c) {return Err(Errors::InvalidTokenError)}
        if c == '(' {
            let mut j = i;
            let mut depth = 0;
            while j < input.len() {
                if input.chars().nth(j) == Some('(') {
                    depth+=1;
                }
                if input.chars().nth(j) == Some(')') {
                    depth-=1;
                    if depth == 0 {
                        break;
                    }
                }
                j+=1;
            }

            if depth != 0 {
                println!("Reached end with unfinished parentheses");
                return Err(Errors::MismatchedParentheses);
            }

            
            let exprgroup = ExprGroup::new(
                input.get(i+1..j)
                .ok_or(Errors::UnexpectedParsingError)?
                .to_string()
            );
            groups.push(exprgroup);
            
            // We remove the subgroup's string from the current input
            *input = input.get(..i).ok_or(Errors::UnexpectedParsingError)?.to_string() + input.get(j+1..).ok_or(Errors::UnexpectedParsingError)?;
            i = 0; // We go back to the beginning of the new input
        } else {
            i+=1;
        }
    }


    Ok(groups)
}

fn find_operator() {
    // Obeying order : Division, Product, (Addition & Substraction)
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
    pub fn parse(&mut self) -> Result<Vec<Token>, CalcError> {
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

