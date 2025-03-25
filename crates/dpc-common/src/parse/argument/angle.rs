use super::{Float, ParseArgContext, primitives::parse_float};

#[derive(Debug)]
pub struct Angle {
    pub value: Float,
    pub relative: bool,
}

pub fn parse(ctx: &mut ParseArgContext<'_, '_>) -> Angle {
    let relative = ctx.reader.peek() == Some('~');
    if relative {
        ctx.reader.advance();
    }
    let mut value = Float::ZERO;
    if ctx.reader.peek().is_some_and(|chr| !chr.is_whitespace()) {
        value = parse_float(ctx, f32::MIN, f32::MAX);
    }
    Angle { value, relative }
}
