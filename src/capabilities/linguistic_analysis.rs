use std::collections::BTreeSet;

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    mathematics::rational::Rational,
    reasoning::{epistemic_kind::EpistemicKind, node_id::NodeId},
    representation::{
        content::Content, content_kind::ContentKind, statement::Statement, term::Term,
    },
    resources::lexicon::Lexicon,
};

/// Linguistic analysis of documents.
///
/// Documents are segmented into sentences and tokens, then every token is
/// looked up in the lexicon. All readings of an ambiguous form are kept,
/// together with their morphological features and senses, so that the
/// reasoning can disambiguate from context. Recognising negation, modality
/// or conditions is done by reasoning over these structures with the
/// linguistic knowledge of the resources, not by fixed word lists.
///
/// Produced relations:
/// `sentence(doc, s, text)`, `token(doc, s, i, form)`,
/// `reading(doc, s, i, lemma, category)`,
/// `morphology(doc, s, i, feature, value)`, `sense(lemma, sense, definition)`,
/// `sense_relation(sense, relation, target)`.
pub(crate) struct LinguisticAnalysis {
    descriptor: ModuleDescriptor,
}

impl LinguisticAnalysis {
    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "linguistic_analysis",
                "Segments documents and relates their words to lexical knowledge, keeping ambiguities.",
            )
            .consuming(&[ContentKind::Document])
            .producing(&[EpistemicKind::Deduction]),
        }
    }

    pub(crate) fn sentences(text: &str) -> Vec<&str> {
        let mut sentences = Vec::new();
        let mut start = 0;
        let mut characters = text.char_indices().peekable();

        while let Some((index, character)) = characters.next() {
            let boundary = matches!(character, '.' | '!' | '?' | '\n')
                && characters
                    .peek()
                    .is_none_or(|(_, next)| next.is_whitespace());

            if boundary {
                let end = index + character.len_utf8();
                let sentence = text[start..end].trim();

                if !sentence.is_empty() && sentence.chars().any(char::is_alphanumeric) {
                    sentences.push(sentence);
                }

                start = end;
            }
        }

        let rest = text[start..].trim();

        if !rest.is_empty() && rest.chars().any(char::is_alphanumeric) {
            sentences.push(rest);
        }

        sentences
    }

    pub(crate) fn tokens(sentence: &str) -> Vec<&str> {
        let mut tokens = Vec::new();
        let mut start: Option<usize> = None;
        let characters: Vec<(usize, char)> = sentence.char_indices().collect();

        for (position, (index, character)) in characters.iter().enumerate() {
            let joining = matches!(character, '\'' | '’' | '-')
                && start.is_some()
                && characters
                    .get(position + 1)
                    .is_some_and(|(_, next)| next.is_alphanumeric());

            if character.is_alphanumeric() || character == &'_' || joining {
                start.get_or_insert(*index);
                continue;
            }

            if let Some(begin) = start.take() {
                tokens.push(&sentence[begin..*index]);
            }

            if !character.is_whitespace() {
                tokens.push(&sentence[*index..*index + character.len_utf8()]);
            }
        }

        if let Some(begin) = start {
            tokens.push(&sentence[begin..]);
        }

        tokens
    }

    fn number(value: usize) -> Term {
        Term::Number(Rational::integer(value as i128))
    }

    fn analyse(
        lexicon: &Lexicon,
        document: NodeId,
        identifier: &str,
        text: &str,
        output: &mut ModuleOutput,
        senses: &mut BTreeSet<String>,
    ) {
        let doc = Term::symbol(identifier);
        let mut propose = |statement: Statement| {
            output.propose(Proposal::new(
                Content::Statement(statement),
                EpistemicKind::Deduction,
                vec![document],
            ));
        };

        for (sentence_index, sentence) in Self::sentences(text).into_iter().enumerate() {
            let position = Self::number(sentence_index);

            propose(Statement::new(
                "sentence",
                vec![doc.clone(), position.clone(), Term::text(sentence)],
            ));

            for (token_index, token) in Self::tokens(sentence).into_iter().enumerate() {
                let place = vec![doc.clone(), position.clone(), Self::number(token_index)];

                propose(Statement::new(
                    "token",
                    [place.clone(), vec![Term::text(token)]].concat(),
                ));

                for entry in lexicon.lookup(token) {
                    propose(Statement::new(
                        "reading",
                        [
                            place.clone(),
                            vec![Term::text(entry.lemma()), Term::symbol(entry.category())],
                        ]
                        .concat(),
                    ));

                    for (feature, value) in entry.features() {
                        propose(Statement::new(
                            "morphology",
                            [
                                place.clone(),
                                vec![Term::symbol(feature), Term::symbol(value)],
                            ]
                            .concat(),
                        ));
                    }

                    for sense in entry.senses() {
                        if !senses.insert(sense.identifier().to_owned()) {
                            continue;
                        }

                        propose(Statement::new(
                            "sense",
                            vec![
                                Term::text(entry.lemma()),
                                Term::symbol(sense.identifier()),
                                Term::text(sense.definition()),
                            ],
                        ));

                        for (relation, target) in sense.relations() {
                            propose(Statement::new(
                                "sense_relation",
                                vec![
                                    Term::symbol(sense.identifier()),
                                    Term::symbol(relation),
                                    Term::symbol(target),
                                ],
                            ));
                        }
                    }
                }
            }
        }
    }
}

impl CognitiveModule for LinguisticAnalysis {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let lexicon = context.resources().lexicon();
        let mut output = ModuleOutput::default();
        let mut senses = BTreeSet::new();

        if lexicon.len() == 0 {
            output.note("no lexicon available: only segmentation is performed");
        } else {
            output.note(format!(
                "lexicon of {} entries from {}",
                lexicon.len(),
                lexicon.sources().join(", ")
            ));
        }

        for node in context.graph().valid_nodes() {
            if node.revision() <= context.seen_revision() {
                continue;
            }

            if let Some(document) = node.content().document() {
                Self::analyse(
                    lexicon,
                    node.id(),
                    document.identifier(),
                    document.text(),
                    &mut output,
                    &mut senses,
                );
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::LinguisticAnalysis;

    #[test]
    fn segments_sentences_and_tokens() {
        let sentences =
            LinguisticAnalysis::sentences("It doesn't work. Version 1.2 is fine!\nDone");

        assert_eq!(
            sentences,
            vec!["It doesn't work.", "Version 1.2 is fine!", "Done"]
        );
        assert_eq!(
            LinguisticAnalysis::tokens("It doesn't work."),
            vec!["It", "doesn't", "work", "."]
        );
    }
}
