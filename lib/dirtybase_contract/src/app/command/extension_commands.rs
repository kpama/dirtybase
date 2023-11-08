use crate::ExtensionSetup;

/// Command that should be handled by the core application.
/// By default this will be the dirtybase framework
pub struct RegisterExtensionCommand {
    extension: Box<dyn ExtensionSetup>,
}

impl RegisterExtensionCommand {
    pub fn new(extension: impl ExtensionSetup + 'static) -> Self {
        Self {
            extension: Box::new(extension),
        }
    }
}

#[async_trait::async_trait]
impl busstop::DispatchableCommand for RegisterExtensionCommand {}
