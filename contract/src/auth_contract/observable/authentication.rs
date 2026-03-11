use crate::{
    auth_contract::{Actor, GuardResponse},
    prelude::{Context, Observable},
};

/// Dispatches when authentication is successful
#[derive(Debug)]
pub struct AuthSucceeded {
    actor: Actor,
}

impl AuthSucceeded {
    pub fn new(actor: Actor) -> Self {
        Self { actor }
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub fn set_actor(&mut self, actor: Actor) {
        self.actor = actor;
    }

    pub async fn dispatch_response(actor: Actor, ctx: &Context) -> Actor {
        (Self::new(actor)).notify(ctx).await.actor().clone()
    }
}

impl Observable for AuthSucceeded {}

/// Dispatches when authentication is unsuccessful
#[derive(Debug)]
pub struct AuthUnSuccessful {
    resp: GuardResponse,
}

impl AuthUnSuccessful {
    pub fn new(resp: GuardResponse) -> Self {
        Self { resp }
    }

    pub fn set_response(&mut self, resp: GuardResponse) -> &mut Self {
        self.resp = resp;
        self
    }

    pub fn response(&self) -> &GuardResponse {
        &self.resp
    }

    pub fn take_response(self) -> GuardResponse {
        self.resp
    }
}

impl Observable for AuthUnSuccessful {}
