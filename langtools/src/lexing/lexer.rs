use crate::{
    lexing::lexing_error::LexingError,
    messaging::message_context::MessageContext,
    sourcing::{read_source::ReadSource, source_reader::SourceReader},
};

use super::{
    dfsa::DFSA,
    lexer_context::LexerContext,
    lexer_trigger_action::{LexerTriggerAction, LexerTriggerActionCallback},
    lexing_error::Result,
};

pub type LexerErrorHandler = fn(&mut dyn ReadSource);

#[readonly::make]
pub struct Lexer<TTokenKind> {
    pub trigger_dfsa: DFSA<char, LexerTriggerAction<TTokenKind>>,
    error_handler: Option<LexerErrorHandler>,
}

impl<TTokenKind> Lexer<TTokenKind> {
    pub fn new() -> Self {
        let mut trigger_dfsa = DFSA::new();
        let start_id = trigger_dfsa.add_state();
        trigger_dfsa
            .set_start_id(start_id)
            .expect("setting a start id here should be fine");

        Self {
            trigger_dfsa,
            error_handler: None,
        }
    }

    pub fn add_trigger(
        &mut self,
        prefix: &str,
        callback: LexerTriggerActionCallback<TTokenKind>,
    ) -> Result<()> {
        assert!(!prefix.is_empty(), "cannot add trigger with empty prefix");

        let mut current_id = self.trigger_dfsa.try_get_start_id()?;

        for element in prefix.chars() {
            if let Ok(next_id) = self.trigger_dfsa.try_get_transition(current_id, element) {
                current_id = next_id;
            } else {
                let next_id = self.trigger_dfsa.add_state();
                self.trigger_dfsa
                    .add_transition(current_id, element, next_id)?;
                current_id = next_id;
            }
        }

        if self.trigger_dfsa.try_get_state_action(current_id).is_ok() {
            Err(LexingError::DuplicateTrigger(prefix.to_owned()))
        } else {
            self.trigger_dfsa
                .set_state_action(current_id, Some(LexerTriggerAction::new(callback)))?;

            Ok(())
        }
    }

    pub fn lex<'self_>(
        &'self_ self,
        source_reader: &'self_ mut SourceReader<'self_>,
        message_context: &'self_ mut MessageContext,
    ) -> LexerContext<'self_, TTokenKind> {
        LexerContext::new(self, source_reader, message_context)
    }

    pub fn set_error_handler(&mut self, error_handler: LexerErrorHandler) {
        self.error_handler = Some(error_handler)
    }

    pub fn get_error_handler(&self) -> LexerErrorHandler {
        self.error_handler.unwrap_or(|read_source| {
            let _ = read_source.eat_next();
        })
    }
}

impl<TTokenKind> Default for Lexer<TTokenKind> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::lexing::lexing_error::LexingError;

    use super::*;

    #[test]
    fn test_lexer_add_trigger() {
        let mut lexer = Lexer::<&str>::new();

        assert_eq!(lexer.add_trigger("ab", |_| Some("ab")), Ok(()));
        assert_eq!(lexer.add_trigger("ac", |_| Some("ac")), Ok(()));
        assert_eq!(lexer.add_trigger("abc", |_| Some("abc")), Ok(()));
        assert_eq!(
            lexer.add_trigger("ab", |_| Some("ab")),
            Err(LexingError::DuplicateTrigger(String::from("ab")))
        );
    }
}
