use crate::{
    mathematics::{expression::Expression, rational::Rational},
    representation::{rule::Rule, statement::Statement, term::Term},
};

/// Parser of the textual notation used for statements and rules:
///
/// - statement: `[not] predicate(term, ...)`
/// - rule: `conclusion :- premise, premise, ...`
/// - term: `?variable`, `"text"`, `42`, `3/4`, `0.5`, `v1.2.3`,
///   `[arithmetic expression]` or a bare symbol.
pub(crate) struct NotationParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> NotationParser<'a> {
    const SYMBOL_DELIMITERS: &'static [char] = &[',', '(', ')', '[', ']', '"'];

    pub(crate) fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    pub(crate) fn statement_only(mut self) -> Result<Statement, String> {
        let statement = self.statement()?;
        self.end()?;
        Ok(statement)
    }

    pub(crate) fn rule_only(mut self) -> Result<Rule, String> {
        let conclusion = self.statement()?;
        self.expect(":-")?;

        let mut premises = vec![self.statement()?];

        while self.consume(",") {
            premises.push(self.statement()?);
        }

        self.end()?;

        Rule::new(premises, conclusion)
    }

    fn statement(&mut self) -> Result<Statement, String> {
        self.skip_whitespace();

        let negated = self.keyword("not");
        let predicate = self.identifier()?;

        self.expect("(")?;

        let mut arguments = Vec::new();

        if !self.consume(")") {
            loop {
                arguments.push(self.term()?);

                if self.consume(")") {
                    break;
                }

                self.expect(",")?;
            }
        }

        Ok(if negated {
            Statement::negative(predicate, arguments)
        } else {
            Statement::new(predicate, arguments)
        })
    }

    fn term(&mut self) -> Result<Term, String> {
        self.skip_whitespace();

        match self.peek() {
            Some('?') => {
                self.position += 1;
                Ok(Term::Variable(self.identifier()?))
            }
            Some('"') => self.text().map(Term::Text),
            Some('[') => {
                self.position += 1;
                let expression = self.sum()?;
                self.expect("]")?;
                Ok(Term::Expression(expression))
            }
            Some(_) => {
                let token = self.symbol_token()?;
                Ok(Self::classify(token))
            }
            None => Err(self.error("term expected")),
        }
    }

    fn classify(token: &str) -> Term {
        if let Some(version) = token.strip_prefix('v')
            && version.starts_with(|character: char| character.is_ascii_digit())
            && let Ok(version) = semver::Version::parse(version)
        {
            return Term::Version(version);
        }

        if token.starts_with(|character: char| character.is_ascii_digit() || character == '-')
            && let Ok(number) = token.parse::<Rational>()
        {
            return Term::Number(number);
        }

        Term::symbol(token)
    }

    fn sum(&mut self) -> Result<Expression, String> {
        let mut expression = self.product()?;

        loop {
            if self.consume("+") {
                expression = Expression::Add(Box::new(expression), Box::new(self.product()?));
            } else if self.consume("-") {
                expression = Expression::Subtract(Box::new(expression), Box::new(self.product()?));
            } else {
                return Ok(expression);
            }
        }
    }

    fn product(&mut self) -> Result<Expression, String> {
        let mut expression = self.unary()?;

        loop {
            if self.consume("*") {
                expression = Expression::Multiply(Box::new(expression), Box::new(self.unary()?));
            } else if self.consume("/") {
                expression = Expression::Divide(Box::new(expression), Box::new(self.unary()?));
            } else {
                return Ok(expression);
            }
        }
    }

    fn unary(&mut self) -> Result<Expression, String> {
        if self.consume("-") {
            return Ok(Expression::Negate(Box::new(self.unary()?)));
        }

        if self.consume("(") {
            let expression = self.sum()?;
            self.expect(")")?;
            return Ok(expression);
        }

        self.skip_whitespace();

        let start = self.position;

        while let Some(character) = self.peek() {
            if character.is_alphanumeric() || matches!(character, '_' | '.') {
                self.position += character.len_utf8();
            } else {
                break;
            }
        }

        let token = &self.input[start..self.position];

        if token.is_empty() {
            return Err(self.error("expression operand expected"));
        }

        if token.starts_with(|character: char| character.is_ascii_digit()) {
            return token.parse().map(Expression::Number);
        }

        Ok(Expression::Variable(token.to_owned()))
    }

    fn text(&mut self) -> Result<String, String> {
        self.position += 1;

        let mut value = String::new();
        let mut characters = self.input[self.position..].char_indices();

        while let Some((offset, character)) = characters.next() {
            match character {
                '"' => {
                    self.position += offset + 1;
                    return Ok(value);
                }
                '\\' => {
                    let Some((_, escaped)) = characters.next() else {
                        break;
                    };

                    match escaped {
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        '0' => value.push('\0'),
                        'u' => {
                            let mut code = String::new();

                            for (_, character) in characters.by_ref() {
                                match character {
                                    '{' => {}
                                    '}' => break,
                                    character => code.push(character),
                                }
                            }

                            let character = u32::from_str_radix(&code, 16)
                                .ok()
                                .and_then(char::from_u32)
                                .ok_or_else(|| self.error("invalid unicode escape"))?;

                            value.push(character);
                        }
                        other => value.push(other),
                    }
                }
                character => value.push(character),
            }
        }

        Err(self.error("unterminated text"))
    }

    fn identifier(&mut self) -> Result<String, String> {
        self.skip_whitespace();

        let start = self.position;

        while let Some(character) = self.peek() {
            if character.is_alphanumeric() || matches!(character, '_' | '-' | '.' | ':') {
                self.position += character.len_utf8();
            } else {
                break;
            }
        }

        if start == self.position {
            return Err(self.error("identifier expected"));
        }

        Ok(self.input[start..self.position].to_owned())
    }

    fn symbol_token(&mut self) -> Result<&'a str, String> {
        let start = self.position;

        while let Some(character) = self.peek() {
            if character.is_whitespace() || Self::SYMBOL_DELIMITERS.contains(&character) {
                break;
            }

            self.position += character.len_utf8();
        }

        if start == self.position {
            return Err(self.error("symbol expected"));
        }

        Ok(&self.input[start..self.position])
    }

    fn keyword(&mut self, keyword: &str) -> bool {
        let rest = &self.input[self.position..];

        if rest.starts_with(keyword) && rest[keyword.len()..].starts_with(char::is_whitespace) {
            self.position += keyword.len();
            self.skip_whitespace();
            return true;
        }

        false
    }

    fn consume(&mut self, token: &str) -> bool {
        self.skip_whitespace();

        if self.input[self.position..].starts_with(token) {
            self.position += token.len();
            return true;
        }

        false
    }

    fn expect(&mut self, token: &str) -> Result<(), String> {
        if self.consume(token) {
            Ok(())
        } else {
            Err(self.error(&format!("`{token}` expected")))
        }
    }

    fn end(&mut self) -> Result<(), String> {
        self.skip_whitespace();
        self.consume(".");
        self.skip_whitespace();

        if self.position == self.input.len() {
            Ok(())
        } else {
            Err(self.error("unexpected trailing input"))
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn skip_whitespace(&mut self) {
        while let Some(character) = self.peek() {
            if !character.is_whitespace() {
                break;
            }

            self.position += character.len_utf8();
        }
    }

    fn error(&self, message: &str) -> String {
        format!(
            "Invalid notation at offset {} in `{}`: {message}",
            self.position, self.input
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::representation::{rule::Rule, statement::Statement, term::Term};

    #[test]
    fn round_trips_statements_and_rules() {
        let statement: Statement =
            r#"not uses(owner/repo, "a \"b\"", v1.2.3, 3/4, [2 * (x + 1)], ?who)"#
                .parse()
                .unwrap();

        assert!(statement.is_negated());
        assert_eq!(statement.arguments().len(), 6);
        assert!(matches!(statement.arguments()[2], Term::Version(_)));
        assert!(matches!(statement.arguments()[4], Term::Expression(_)));
        assert_eq!(
            statement.to_string().parse::<Statement>().unwrap(),
            statement
        );

        let rule: Rule = "mortal(?x) :- human(?x).".parse().unwrap();

        assert_eq!(rule.to_string().parse::<Rule>().unwrap(), rule);
        assert!("mortal(?y) :- human(?x)".parse::<Rule>().is_err());
    }
}
