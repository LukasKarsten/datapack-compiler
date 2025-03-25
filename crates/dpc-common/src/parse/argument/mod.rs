use std::fmt;

pub use angle::Angle;
pub use color::{ChatColor, Color};
pub use coords::{Coordinates, WorldCoordinate};
pub use primitives::{Boolean, Double, Float, Integer, Text};
use smallvec::SmallVec;

use super::{Reader, cst, errors::ParseError};
use crate::intern::StaticInterner;

mod angle;
mod color;
mod coords;
mod primitives;

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
            Self::Bool => Ok(cst::ArgumentValue::Boolean(primitives::parse_bool(ctx))),
            Self::Integer { .. } => Ok(cst::ArgumentValue::Integer(primitives::parse_integer(ctx))),
            Self::Float { .. } => Ok(cst::ArgumentValue::Float(primitives::parse_float(ctx))),
            Self::Double { .. } => Ok(cst::ArgumentValue::Double(primitives::parse_double(ctx))),
            Self::String(kind) => {
                primitives::parse_text(ctx, *kind).map(cst::ArgumentValue::String)
            }
            Self::Angle => Ok(cst::ArgumentValue::Angle(angle::parse(ctx))),
            Self::BlockPos => Ok(cst::ArgumentValue::Coordinates3(coords::parse_block_pos(
                ctx,
            ))),
            Self::BlockPredicate => todo!(),
            Self::BlockState => todo!(),
            Self::Color => Ok(cst::ArgumentValue::Color(color::parse(ctx))),
            Self::ColumnPos => Ok(cst::ArgumentValue::Coordinates2(coords::parse_column_pos(
                ctx,
            ))),
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
            Self::Vec2 => Ok(cst::ArgumentValue::Coordinates2(coords::parse_vec2(ctx))),
            Self::Vec3 => Ok(cst::ArgumentValue::Coordinates3(coords::parse_vec3(ctx))),
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
