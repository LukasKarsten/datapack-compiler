use std::fmt;

use smallvec::SmallVec;

use crate::{
    arguments::ChatColor,
    cst,
    intern::{Interner, StaticInterner},
    span::Span,
};

use super::{
    Reader,
    errors::{
        ExpectedLocalCoordinateError, IncompleteLocalCoordinatesError, InvalidColorError,
        InvalidStringCharsError, MixedCoordiantesError, ParseBoolError, ParseDoubleError,
        ParseError, ParseFloatError, ParseIntegerError, QuotedSingleWordError,
        UnterminatedStringError,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StringKind {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

#[derive(Clone)]
pub enum Argument {
    Bool,
    Double { min: f64, max: f64 },
    Float { min: f32, max: f32 },
    Integer { min: i32, max: i32 },
    String(StringKind),
    Angle,
    BlockPos,
    BlockPredicate,
    BlockState,
    Color,
    ColumnPos,
    Component,
    Dimension,
    Entity { single: bool, players_only: bool },
    EntityAnchor,
    Function,
    GameProfile,
    Gamemode,
    Heightmap,
    IntRange,
    ItemPredicate,
    ItemSlot,
    ItemSlots,
    ItemStack,
    LootModifier,
    LootPredicate,
    LootTable,
    Message,
    NbtCompoundTag,
    NbtPath,
    NbtTag,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Resource { registry: Box<str> },
    ResourceKey { registry: Box<str> },
    ResourceLocation,
    ResourceOrTag { registry: Box<str> },
    ResourceOrTagKey { registry: Box<str> },
    Rotation,
    ScoreHolder { single: bool },
    ScoreboardSlot,
    Style,
    Swizzle,
    Team,
    TemplateMirror,
    TemplateRotation,
    Time { min: i32 },
    Vec2,
    Vec3,
}

pub struct ParseArgContext<'a, 'src> {
    pub reader: &'a mut Reader<'src>,
    pub interner: &'a mut StaticInterner,
    pub errors: SmallVec<[ParseError; 1]>,
}

impl ParseArgContext<'_, '_> {
    fn error(&mut self, error: ParseError) {
        self.errors.push(error);
    }
}

impl Argument {
    pub fn parse(
        &self,
        ctx: &mut ParseArgContext<'_, '_>,
    ) -> Result<cst::ArgumentValue, ParseError> {
        match self {
            Self::Bool => Ok(cst::ArgumentValue::Boolean(parse_bool(ctx))),
            Self::Integer { .. } => Ok(cst::ArgumentValue::Integer(parse_integer(ctx))),
            Self::Float { .. } => Ok(cst::ArgumentValue::Float(parse_float(ctx))),
            Self::Double { .. } => Ok(cst::ArgumentValue::Double(parse_double(ctx))),
            Self::String(kind) => parse_string(ctx, *kind).map(cst::ArgumentValue::String),
            Self::Angle => Ok(cst::ArgumentValue::Angle(parse_angle(ctx))),
            Self::BlockPos => Ok(cst::ArgumentValue::Coordinates3(parse_block_pos(ctx))),
            Self::BlockPredicate => todo!(),
            Self::BlockState => todo!(),
            Self::Color => Ok(cst::ArgumentValue::Color(parse_color(ctx))),
            Self::ColumnPos => Ok(cst::ArgumentValue::Coordinates2(parse_column_pos(ctx))),
            Self::Component => todo!(),
            Self::Dimension => todo!(),
            Self::Entity {
                single: _,
                players_only: _,
            } => {
                todo!()
            }
            Self::EntityAnchor => todo!(),
            Self::Function => todo!(),
            Self::GameProfile => todo!(),
            Self::Gamemode => todo!(),
            Self::Heightmap => todo!(),
            Self::IntRange => todo!(),
            Self::ItemPredicate => todo!(),
            Self::ItemSlot => todo!(),
            Self::ItemSlots => todo!(),
            Self::ItemStack => todo!(),
            Self::LootModifier => todo!(),
            Self::LootPredicate => todo!(),
            Self::LootTable => todo!(),
            Self::Message => todo!(),
            Self::NbtCompoundTag => todo!(),
            Self::NbtPath => todo!(),
            Self::NbtTag => todo!(),
            Self::Objective => todo!(),
            Self::ObjectiveCriteria => todo!(),
            Self::Operation => todo!(),
            Self::Particle => todo!(),
            Self::Resource { registry: _ } => todo!(),
            Self::ResourceKey { registry: _ } => todo!(),
            Self::ResourceLocation => todo!(),
            Self::ResourceOrTag { registry: _ } => todo!(),
            Self::ResourceOrTagKey { registry: _ } => todo!(),
            Self::Rotation => todo!(),
            Self::ScoreHolder { single: _ } => todo!(),
            Self::ScoreboardSlot => todo!(),
            Self::Style => todo!(),
            Self::Swizzle => todo!(),
            Self::Team => todo!(),
            Self::TemplateMirror => todo!(),
            Self::TemplateRotation => todo!(),
            Self::Time { min: _ } => todo!(),
            Self::Vec2 => Ok(cst::ArgumentValue::Coordinates2(parse_vec2(ctx))),
            Self::Vec3 => Ok(cst::ArgumentValue::Coordinates3(parse_vec3(ctx))),
        }
    }
}

impl fmt::Debug for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => f.write_str("bool"),
            Self::Double { min, max } => {
                f.write_str("double")?;
                match (*min, *max) {
                    (f64::MIN, f64::MAX) => Ok(()),
                    (_, f64::MAX) => write!(f, "(min={min:?})"),
                    (f64::MIN, _) => write!(f, "(max={max:?})"),
                    (_, _) => write!(f, "(min={min:?} max={max:?})"),
                }
            }
            Self::Float { min, max } => {
                f.write_str("float")?;
                match (*min, *max) {
                    (f32::MIN, f32::MAX) => Ok(()),
                    (_, f32::MAX) => write!(f, "(min={min:?})"),
                    (f32::MIN, _) => write!(f, "(max={max:?})"),
                    (_, _) => write!(f, "(min={min:?} max={max:?})"),
                }
            }
            Self::Integer { min, max } => {
                f.write_str("integer")?;
                match (*min, *max) {
                    (i32::MIN, i32::MAX) => Ok(()),
                    (_, i32::MAX) => write!(f, "(min={min:?})"),
                    (i32::MIN, _) => write!(f, "(max={max:?})"),
                    (_, _) => write!(f, "(min={min:?} max={max:?})"),
                }
            }
            Self::String(StringKind::SingleWord) => f.write_str("string(kind=word)"),
            Self::String(StringKind::QuotablePhrase) => f.write_str("string(kind=phrase)"),
            Self::String(StringKind::GreedyPhrase) => f.write_str("string(kind=greedy)"),
            Self::Angle => f.write_str("angle"),
            Self::BlockPos => f.write_str("block_pos"),
            Self::BlockPredicate => f.write_str("block_predicate"),
            Self::BlockState => f.write_str("block_state"),
            Self::Color => f.write_str("color"),
            Self::ColumnPos => f.write_str("column_pos"),
            Self::Component => f.write_str("component"),
            Self::Dimension => f.write_str("dimension"),
            Self::Entity {
                single,
                players_only,
            } => write!(
                f,
                "entity(amount={}; type={})",
                if *single { "single" } else { "multiple" },
                if *players_only { "players" } else { "entities" },
            ),
            Self::EntityAnchor => f.write_str("entity_anchor"),
            Self::Function => f.write_str("function"),
            Self::GameProfile => f.write_str("game_profile"),
            Self::Gamemode => f.write_str("gamemode"),
            Self::Heightmap => f.write_str("heightmap"),
            Self::IntRange => f.write_str("int_range"),
            Self::ItemPredicate => f.write_str("item_predicate"),
            Self::ItemSlot => f.write_str("item_slot"),
            Self::ItemSlots => f.write_str("item_slots"),
            Self::ItemStack => f.write_str("item_stack"),
            Self::LootModifier => f.write_str("loot_modifier"),
            Self::LootPredicate => f.write_str("loot_predicate"),
            Self::LootTable => f.write_str("loot_table"),
            Self::Message => f.write_str("message"),
            Self::NbtCompoundTag => f.write_str("nbt_compound_tag"),
            Self::NbtPath => f.write_str("nbt_path"),
            Self::NbtTag => f.write_str("nbt_tag"),
            Self::Objective => f.write_str("objective"),
            Self::ObjectiveCriteria => f.write_str("objective_criteria"),
            Self::Operation => f.write_str("operation"),
            Self::Particle => f.write_str("particle"),
            Self::Resource { registry } => write!(f, "resource(registry={registry})"),
            Self::ResourceKey { registry } => write!(f, "resource_key(registry={registry})"),
            Self::ResourceLocation => f.write_str("resource_location"),
            Self::ResourceOrTag { registry } => write!(f, "resource_or_tag(registry={registry})"),
            Self::ResourceOrTagKey { registry } => {
                write!(f, "resource_or_tag_key(registry={registry})")
            }
            Self::Rotation => f.write_str("rotation"),
            Self::ScoreHolder { single: false } => f.write_str("score_holder(amount=multiple)"),
            Self::ScoreHolder { single: true } => f.write_str("score_holder(amount=single)"),
            Self::ScoreboardSlot => f.write_str("scoreboard_slot"),
            Self::Style => f.write_str("style"),
            Self::Swizzle => f.write_str("swizzle"),
            Self::Team => f.write_str("team"),
            Self::TemplateMirror => f.write_str("template_mirror"),
            Self::TemplateRotation => f.write_str("template_rotation"),
            Self::Time { min } => write!(f, "time(min={min})"),
            Self::Vec2 => f.write_str("vec2"),
            Self::Vec3 => f.write_str("vec3"),
        }
    }
}

fn parse_bool(ctx: &mut ParseArgContext<'_, '_>) -> cst::Boolean {
    let range = ctx.reader.read_range_until(char::is_whitespace);
    let value = match &ctx.reader.get_src()[range.clone()] {
        "true" => Some(true),
        "false" => Some(false),
        _ => {
            ctx.errors
                .push(ParseError::ParseBool(ParseBoolError { span: range.into() }));
            None
        }
    };
    cst::Boolean { value }
}

fn read_number_string<'src>(reader: &mut Reader<'src>) -> Result<(&'src str, Span), ParseError> {
    fn is_number_char(chr: char) -> bool {
        matches!(chr, '0'..='9' | '.' | '-')
    }

    let range = reader.read_range_until(char::is_whitespace);
    let span = range.clone().into();
    let string = &reader.get_src()[range.clone()];
    if !string.chars().all(is_number_char) {
        Err(ParseError::ParseInteger(ParseIntegerError { span }))
    } else {
        Ok((string, span))
    }
}

fn parse_integer(ctx: &mut ParseArgContext<'_, '_>) -> cst::Integer {
    let mut value = None;
    match read_number_string(ctx.reader) {
        Ok((string, span)) => match string.parse() {
            Ok(number) => value = Some(number),
            Err(_) => ctx.error(ParseError::ParseInteger(ParseIntegerError { span })),
        },
        Err(err) => ctx.error(err),
    }
    cst::Integer { value }
}

fn parse_float(ctx: &mut ParseArgContext<'_, '_>) -> cst::Float {
    let mut value = None;
    match read_number_string(ctx.reader) {
        Ok((string, span)) => match string.parse() {
            Ok(number) => value = Some(number),
            Err(_) => ctx.error(ParseError::ParseFloat(ParseFloatError { span })),
        },
        Err(err) => ctx.error(err),
    }
    cst::Float { value }
}

fn parse_double(ctx: &mut ParseArgContext<'_, '_>) -> cst::Double {
    let mut value = None;
    match read_number_string(ctx.reader) {
        Ok((string, span)) => match string.parse() {
            Ok(number) => value = Some(number),
            Err(_) => ctx.error(ParseError::ParseDouble(ParseDoubleError { span })),
        },
        Err(err) => ctx.error(err),
    }
    cst::Double { value }
}

fn parse_string(
    ctx: &mut ParseArgContext<'_, '_>,
    kind: StringKind,
) -> Result<cst::String, ParseError> {
    if kind == StringKind::GreedyPhrase {
        return parse_greedy_phrase(ctx);
    }

    let Some(quote @ ('"' | '\'')) = ctx.reader.peek() else {
        let string = parse_unquoted_string(ctx);

        return string;
    };

    let string_start = ctx.reader.get_pos();

    ctx.reader.advance();
    let content_start = ctx.reader.get_pos();

    while let Some(chr) = ctx.reader.peek() {
        if chr == quote {
            let string = &ctx.reader.get_src()[content_start..ctx.reader.get_pos()];
            ctx.reader.advance();

            if kind == StringKind::SingleWord {
                ctx.error(ParseError::QuotedSingleWord(QuotedSingleWordError {
                    span: Span::new(string_start, ctx.reader.get_pos()),
                }));
            }

            return Ok(cst::String {
                value: Some(ctx.interner.intern(string)),
                kind: cst::StringKind::Quotable,
            });
        } else if chr == '\\' {
            ctx.reader.advance();
        }
        ctx.reader.advance();
    }

    let span = string_start..ctx.reader.get_pos();
    Err(ParseError::UnterminatedString(UnterminatedStringError {
        span: span.into(),
    }))
}

fn parse_unquoted_string(ctx: &mut ParseArgContext<'_, '_>) -> Result<cst::String, ParseError> {
    fn is_string_char(chr: char) -> bool {
        matches!(chr, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '+')
    }

    let (range, string) = ctx
        .reader
        .parse_with_span(|reader| reader.read_until(char::is_whitespace));

    let value = if !string.chars().all(is_string_char) {
        ctx.error(ParseError::InvalidStringChars(InvalidStringCharsError {
            span: range.into(),
        }));
        None
    } else {
        Some(ctx.interner.intern(string))
    };

    Ok(cst::String {
        value,
        kind: cst::StringKind::Bare,
    })
}

fn parse_greedy_phrase(ctx: &mut ParseArgContext<'_, '_>) -> Result<cst::String, ParseError> {
    let symbol = ctx.interner.intern(ctx.reader.remaining_src().trim_end());
    ctx.reader.set_pos(ctx.reader.get_src().len());
    Ok(cst::String {
        value: Some(symbol),
        kind: cst::StringKind::Bare,
    })
}

fn parse_angle(ctx: &mut ParseArgContext<'_, '_>) -> cst::Angle {
    let relative = ctx.reader.peek() == Some('~');
    if relative {
        ctx.reader.advance();
    }
    let mut value = cst::Float::ZERO;
    if ctx.reader.peek().is_some_and(|chr| !chr.is_whitespace()) {
        value = parse_float(ctx);
    }
    cst::Angle { value, relative }
}

fn parse_local_coordinates<const N: usize>(
    ctx: &mut ParseArgContext<'_, '_>,
) -> cst::Coordinates<N> {
    let start = ctx.reader.get_pos();

    let mut coords = [cst::Double::ZERO; N];

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
            *coord = parse_double(ctx);
        }
    }

    cst::Coordinates::Local(coords)
}

fn parse_world_coordinates<const N: usize>(
    ctx: &mut ParseArgContext<'_, '_>,
    mut number_parser: impl FnMut(&mut ParseArgContext<'_, '_>, bool) -> cst::Double,
) -> cst::Coordinates<N> {
    let start = ctx.reader.get_pos();

    let mut coords = [cst::WorldCoordinate {
        value: cst::Double::ZERO,
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

    cst::Coordinates::World(coords)
}

fn parse_block_pos(ctx: &mut ParseArgContext<'_, '_>) -> cst::Coordinates<3> {
    match ctx.reader.peek() {
        Some('^') => parse_local_coordinates(ctx),
        _ => parse_world_coordinates(ctx, |ctx, relative| match relative {
            true => parse_double(ctx),
            false => {
                let integer = parse_integer(ctx);
                cst::Double {
                    value: integer.value.map(|value| value as f64),
                }
            }
        }),
    }
}

fn parse_vec3(ctx: &mut ParseArgContext<'_, '_>) -> cst::Coordinates<3> {
    match ctx.reader.peek() {
        Some('^') => parse_local_coordinates(ctx),
        _ => parse_world_coordinates(ctx, |ctx, _| parse_double(ctx)),
    }
}

fn parse_vec2(ctx: &mut ParseArgContext<'_, '_>) -> cst::Coordinates<2> {
    match ctx.reader.peek() {
        Some('^') => parse_local_coordinates(ctx),
        _ => parse_world_coordinates(ctx, |ctx, _| parse_double(ctx)),
    }
}

fn parse_column_pos(ctx: &mut ParseArgContext<'_, '_>) -> cst::Coordinates<2> {
    parse_world_coordinates(ctx, |ctx, _| parse_double(ctx))
}

fn parse_color(ctx: &mut ParseArgContext<'_, '_>) -> cst::Color {
    let (span, name) = ctx
        .reader
        .parse_with_span(|reader| reader.read_until(char::is_whitespace));

    let color = ChatColor::from_string(name);

    if color.is_none() {
        ctx.error(ParseError::InvalidColor(InvalidColorError {
            span: span.into(),
        }));
    }

    cst::Color { color }
}
