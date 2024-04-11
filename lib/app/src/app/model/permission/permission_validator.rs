use super::permission_service::PermissionService;
use crate::app::token_claim::Claim;

pub struct PermissionValidator {
    claim: Claim,
    service: PermissionService,
}

impl PermissionValidator {
    pub fn new(claim: Claim, service: PermissionService) -> Self {
        Self { claim, service }
    }

    pub async fn can<P: ToString>(&self, _name: P) -> bool {
        false
    }
    pub async fn can_do_any<P: ToString>(&self, _name: &[P]) -> bool {
        false
    }

    pub async fn has<P: ToString>(&self, name: P) -> bool {
        self.can(name).await
    }

    pub async fn has_any<P: ToString>(&self, names: &[P]) -> bool {
        self.can_do_any(names).await
    }

    pub async fn is_sys_admin(&self) -> bool {
        false
    }
}
