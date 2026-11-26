use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::{TokenStream, TokenTree};

#[derive(Default)]
pub(crate) struct SourceTokens {
    references: BTreeMap<String, BTreeSet<String>>,
}

impl SourceTokens {
    pub(crate) fn parse(source: &str) -> Result<Self, String> {
        let stream = source
            .parse::<TokenStream>()
            .map_err(|error| error.to_string())?;

        let mut tokens = Self::default();

        tokens.collect(stream, &[]);

        Ok(tokens)
    }

    pub(crate) fn references(&self, name: &str) -> bool {
        self.references.contains_key(name)
    }

    pub(crate) fn identifiers(&self, name: &str) -> BTreeSet<String> {
        self.references.get(name).cloned().unwrap_or_default()
    }

    fn record(&mut self, path: &[String]) {
        if path.len() < 2 {
            return;
        }

        let Some(root) = path.first() else {
            return;
        };

        let Some(symbol) = path.last() else {
            return;
        };

        self.references
            .entry(root.clone())
            .or_default()
            .insert(symbol.clone());
    }

    fn collect(&mut self, stream: TokenStream, prefix: &[String]) {
        let tokens: Vec<_> = stream.into_iter().collect();
        let mut index = 0;

        while index < tokens.len() {
            match &tokens[index] {
                TokenTree::Ident(identifier) => {
                    let name = identifier.to_string();

                    let mut path = prefix.to_vec();

                    path.push(name.strip_prefix("r#").unwrap_or(&name).to_owned());

                    let mut cursor = index + 1;

                    loop {
                        if !Self::punctuation(tokens.get(cursor), ':')
                            || !Self::punctuation(tokens.get(cursor + 1), ':')
                        {
                            break;
                        }

                        match tokens.get(cursor + 2) {
                            Some(TokenTree::Ident(identifier)) => {
                                let name = identifier.to_string();

                                path.push(name.strip_prefix("r#").unwrap_or(&name).to_owned());

                                cursor += 3;
                            }

                            Some(TokenTree::Group(group)) => {
                                self.collect(group.stream(), &path);
                                cursor += 3;
                                break;
                            }

                            Some(TokenTree::Punct(value)) if value.as_char() == '*' => {
                                self.references.entry(path[0].clone()).or_default();

                                cursor += 3;
                                break;
                            }

                            _ => break,
                        }
                    }

                    self.record(&path);

                    index = cursor;
                }

                TokenTree::Group(group) => {
                    self.collect(group.stream(), prefix);

                    index += 1;
                }

                _ => {
                    index += 1;
                }
            }
        }
    }

    fn punctuation(token: Option<&TokenTree>, expected: char) -> bool {
        matches!(
            token,
            Some(TokenTree::Punct(value))
                if value.as_char() == expected
        )
    }
}
