use crate::atoms::Atom;
use std::fmt::Display;
use tracing::error;

pub mod finalizers;
pub mod initializers;

pub struct Step {
    pub atom: Box<dyn Atom>,
    pub initializers: Vec<initializers::FlowControl>,
    pub finalizers: Vec<finalizers::FlowControl>,
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Step: {} (Not printing initializers and finalizers yet)",
            self.atom
        )
    }
}

impl Step {
    pub fn do_initializers_allow_us_to_run(&self) -> bool {
        self.initializers
            .iter()
            .fold(true, |_, flow_control| match flow_control {
                initializers::FlowControl::SkipIf(i) => {
                    match i.initialize() {
                        Ok(true) => {
                            // Returning false because we should Skip if true, so false
                            // will filter this out of the atom list
                            return false;
                        }
                        Ok(false) => true,
                        Err(err) => {
                            error!("Failed to run initializer: {}", err.to_string());

                            // On an error, we can't really determine if this Atom should
                            // run; so lets play it safe and filter it out too
                            return false;
                        }
                    }
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::initializers::test::Echo as EchoInitializer;
    use super::initializers::test::Error as ErrorInitializer;
    use super::initializers::FlowControl;
    use crate::atoms::Echo as EchoAtom;

    use super::*;

    #[test]
    fn initializers_can_control_execution() {
        let step = Step {
            atom: Box::new(EchoAtom("hello-world")),
            initializers: vec![FlowControl::SkipIf(Box::new(EchoInitializer(true)))],
            finalizers: vec![],
        };

        assert_eq!(false, step.do_initializers_allow_us_to_run());

        let step = Step {
            atom: Box::new(EchoAtom("hello-world")),
            initializers: vec![FlowControl::SkipIf(Box::new(EchoInitializer(false)))],
            finalizers: vec![],
        };

        assert_eq!(true, step.do_initializers_allow_us_to_run());

        let step = Step {
            atom: Box::new(EchoAtom("hello-world")),
            initializers: vec![
                FlowControl::SkipIf(Box::new(EchoInitializer(false))),
                FlowControl::SkipIf(Box::new(EchoInitializer(true))),
            ],
            finalizers: vec![],
        };

        assert_eq!(false, step.do_initializers_allow_us_to_run());
    }

    #[test]
    fn initializers_that_error_block_execution() {
        let step = Step {
            atom: Box::new(EchoAtom("hello-world")),
            initializers: vec![FlowControl::SkipIf(Box::new(ErrorInitializer()))],
            finalizers: vec![],
        };

        assert_eq!(false, step.do_initializers_allow_us_to_run());

        let step = Step {
            atom: Box::new(EchoAtom("hello-world")),
            initializers: vec![
                FlowControl::SkipIf(Box::new(EchoInitializer(false))),
                FlowControl::SkipIf(Box::new(ErrorInitializer())),
            ],
            finalizers: vec![],
        };

        assert_eq!(false, step.do_initializers_allow_us_to_run());
    }
}
