use crate::sourcing::read_source::ReadSource;

pub type LexerTriggerActionCallback<TTokenKind> = fn(&mut dyn ReadSource) -> Option<TTokenKind>;

#[readonly::make]
pub struct LexerTriggerAction<TTokenKind> {
    pub callback: LexerTriggerActionCallback<TTokenKind>,
}

impl<TTokenKind> LexerTriggerAction<TTokenKind> {
    pub fn new(callback: LexerTriggerActionCallback<TTokenKind>) -> Self {
        Self { callback }
    }
}
