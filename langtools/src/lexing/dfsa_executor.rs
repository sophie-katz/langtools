use super::dfsa::DFSA;
use super::dfsa_error::Result;
use super::dfsa_types::DFSAId;
use std::hash::Hash;

#[derive(Debug)]
pub struct DFSAExecutor<'dfsa, TElement: Eq + Hash, TAction> {
    dfsa: &'dfsa DFSA<TElement, TAction>,
    start_id: DFSAId,
    current_id: DFSAId,
}

impl<'dfsa, TElement: Eq + Hash, TAction> DFSAExecutor<'dfsa, TElement, TAction> {
    pub fn new(dfsa: &'dfsa DFSA<TElement, TAction>) -> Result<Self> {
        let start_id = dfsa.try_get_start_id()?;

        Ok(Self {
            dfsa,
            start_id,
            current_id: start_id,
        })
    }

    pub fn reset(&mut self) {
        self.current_id = self.start_id
    }

    pub fn step(&mut self, element: TElement) -> Result<()> {
        match self.dfsa.try_get_transition(self.current_id, element) {
            Ok(next_id) => {
                self.current_id = next_id;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn current_action(&self) -> Option<&'dfsa TAction> {
        self.dfsa.try_get_state_action(self.current_id).ok()
    }

    pub fn is_at_start(&self) -> bool {
        self.start_id == self.current_id
    }
}

#[cfg(test)]
mod tests {
    use super::super::dfsa_error::DFSAError;
    use super::*;

    #[test]
    fn test_dfsa_executor_good() -> Result<()> {
        let mut dfsa = DFSA::<char, String>::new();
        let start = dfsa.add_state();
        let a = dfsa.add_state();
        let ab = dfsa.add_state_with_action(String::from("ab"));
        let b = dfsa.add_state_with_action(String::from("b"));
        dfsa.add_transition(start, 'a', a)?;
        dfsa.add_transition(a, 'b', ab)?;
        dfsa.add_transition(start, 'b', b)?;
        dfsa.set_start_id(start)?;

        let mut dfsa_executor = DFSAExecutor::new(&dfsa)?;

        assert_eq!(dfsa_executor.step('a'), Ok(()));
        assert_eq!(dfsa_executor.current_action(), None);
        assert_eq!(dfsa_executor.step('b'), Ok(()));
        assert_eq!(
            dfsa_executor.current_action().cloned(),
            Some(String::from("ab"))
        );
        assert_eq!(dfsa_executor.step('a'), Err(DFSAError::NoSuchTransition));

        dfsa_executor.reset();

        assert_eq!(dfsa_executor.step('b'), Ok(()));
        assert_eq!(
            dfsa_executor.current_action().cloned(),
            Some(String::from("b"))
        );
        assert_eq!(dfsa_executor.step('a'), Err(DFSAError::NoSuchTransition));

        Ok(())
    }

    #[test]
    fn test_dfsa_executor_new_no_start_id() {
        let dfsa = DFSA::<char, String>::new();

        assert_eq!(
            DFSAExecutor::new(&dfsa).map(|_| ()),
            Err(DFSAError::NoStartId)
        );
    }
}
