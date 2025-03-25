use super::{
    Double, ParseArgContext,
    primitives::{parse_double, parse_integer},
};
use crate::{
    parse::errors::{
        ExpectedLocalCoordinateError, IncompleteLocalCoordinatesError, MixedCoordiantesError,
        ParseError,
    },
    span::Span,
};

#[derive(Debug, Clone, Copy)]
pub struct WorldCoordinate {
    pub value: Double,
    pub relative: bool,
}

#[derive(Debug)]
pub enum Coordinates<const N: usize> {
    World([WorldCoordinate; N]),
    Local([Double; N]),
}

fn parse_local_coordinates<const N: usize>(ctx: &mut ParseArgContext<'_, '_>) -> Coordinates<N> {
    let start = ctx.reader.get_pos();

    let mut coords = [Double::ZERO; N];

    for coord in &mut coords {
        ctx.reader.skip_whitespace();

        if !ctx.reader.has_more() {
            ctx.error(ParseError::IncompleteLocalCoordinates(
                IncompleteLocalCoordinatesError {
                    span: Span::new(start, ctx.reader.get_pos()),
                },
            ));
            break;
        }

        if ctx.reader.peek() == Some('^') {
            ctx.reader.advance();
        } else if ctx.reader.peek() == Some('~') {
            ctx.error(ParseError::MixedCoordinates(MixedCoordiantesError {
                span: Span::new(ctx.reader.get_pos(), ctx.reader.get_next_pos()),
            }));
            ctx.reader.advance();
        } else {
            ctx.error(ParseError::ExpectedLocalCoordinate(
                ExpectedLocalCoordinateError {
                    span: Span::new(ctx.reader.get_pos(), ctx.reader.get_next_pos()),
                },
            ));
        }

        if ctx.reader.peek().is_some_and(|chr| !chr.is_whitespace()) {
            *coord = parse_double(ctx, f64::MIN, f64::MAX);
        }
    }

    Coordinates::Local(coords)
}

fn parse_world_coordinates<const N: usize>(
    ctx: &mut ParseArgContext<'_, '_>,
    mut number_parser: impl FnMut(&mut ParseArgContext<'_, '_>, bool) -> Double,
) -> Coordinates<N> {
    let start = ctx.reader.get_pos();

    let mut coords = [WorldCoordinate {
        value: Double::ZERO,
        relative: false,
    }; N];

    for coord in &mut coords {
        ctx.reader.skip_whitespace();

        if !ctx.reader.has_more() {
            ctx.error(ParseError::IncompleteLocalCoordinates(
                IncompleteLocalCoordinatesError {
                    span: Span::new(start, ctx.reader.get_pos()),
                },
            ));
            break;
        }

        if ctx.reader.peek() == Some('~') {
            coord.relative = true;
            ctx.reader.advance();
        } else if ctx.reader.peek() == Some('^') {
            ctx.error(ParseError::MixedCoordinates(MixedCoordiantesError {
                span: Span::new(ctx.reader.get_pos(), ctx.reader.get_next_pos()),
            }));
            ctx.reader.advance();
        }

        if ctx.reader.peek().is_some_and(|chr| !chr.is_whitespace()) || !coord.relative {
            coord.value = number_parser(ctx, coord.relative);
        }
    }

    Coordinates::World(coords)
}

pub fn parse_block_pos(ctx: &mut ParseArgContext<'_, '_>) -> Coordinates<3> {
    match ctx.reader.peek() {
        Some('^') => parse_local_coordinates(ctx),
        _ => parse_world_coordinates(ctx, |ctx, relative| match relative {
            true => parse_double(ctx, f64::MIN, f64::MAX),
            false => {
                let integer = parse_integer(ctx, i32::MIN, i32::MAX);
                Double {
                    value: integer.value.map(|value| value as f64),
                }
            }
        }),
    }
}

pub fn parse_vec3(ctx: &mut ParseArgContext<'_, '_>) -> Coordinates<3> {
    match ctx.reader.peek() {
        Some('^') => parse_local_coordinates(ctx),
        _ => parse_world_coordinates(ctx, |ctx, _| parse_double(ctx, f64::MIN, f64::MAX)),
    }
}

pub fn parse_vec2(ctx: &mut ParseArgContext<'_, '_>) -> Coordinates<2> {
    match ctx.reader.peek() {
        Some('^') => parse_local_coordinates(ctx),
        _ => parse_world_coordinates(ctx, |ctx, _| parse_double(ctx, f64::MIN, f64::MAX)),
    }
}

pub fn parse_column_pos(ctx: &mut ParseArgContext<'_, '_>) -> Coordinates<2> {
    parse_world_coordinates(ctx, |ctx, _| parse_double(ctx, f64::MIN, f64::MAX))
}
