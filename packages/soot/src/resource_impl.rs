use crate::resource::ResourceDef;

/// A Rust type that knows its own resource declaration.
///
/// This is the one place soot asks for a concrete type. Implementing it lets a
/// caller write `domain.add_resource::<Post>()` instead of passing a
/// [`ResourceDef`] around, which keeps a declaration next to the data it
/// describes. Nothing in the engine requires it — a domain full of bare
/// `ResourceDef`s is fully functional — so this is a convenience, not a
/// requirement.
pub trait Resource: Sized {
    /// The resource's name.
    const NAME: &'static str;

    /// The resource's declaration.
    fn definition() -> ResourceDef;
}
