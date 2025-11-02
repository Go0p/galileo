pub mod bundle;
pub mod decorators;

pub use bundle::InstructionBundle;
pub use decorators::{AssemblyContext, DecoratorChain};

pub(super) fn attach_lighthouse<'a>(
    ctx: &mut AssemblyContext<'a>,
    value: &'a mut super::LighthouseRuntime,
) {
    decorators::attach_lighthouse_internal(ctx, value);
}
