#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanValidationError {
    MixedSendAndInteraction,
    InvalidFirstStep,
    ResponseNotFirst,
    DeferNotFirst,
    DeferAndResponse,
    OpenModalNotExclusive,
}

impl std::fmt::Display for PlanValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            PlanValidationError::MixedSendAndInteraction => {
                "Send cannot be mixed with interaction steps"
            }
            PlanValidationError::InvalidFirstStep => {
                "interaction plan must start with Defer, Response, or OpenModal"
            }
            PlanValidationError::ResponseNotFirst => "Response must be the first step",
            PlanValidationError::DeferNotFirst => "Defer must be the first step",
            PlanValidationError::DeferAndResponse => "Defer and Response cannot both appear",
            PlanValidationError::OpenModalNotExclusive => "OpenModal must be the only step",
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for PlanValidationError {}
