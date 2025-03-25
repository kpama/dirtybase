use crate::GuardResolver;

pub const SESSION_GUARD: &'static str = "session";

pub async fn authenticate(resolver: GuardResolver) -> GuardResolver {
    tracing::info!(">>>> In SESSION Authentication guard");

    resolver
}
