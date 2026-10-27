
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
