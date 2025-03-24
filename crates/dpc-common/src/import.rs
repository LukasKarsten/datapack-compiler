use std::collections::HashMap;

use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    BuildTree, Node,
    build_tree::BuildNodeId,
    parse::argument::{Argument, StringKind},
};

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum JsonNodeKind {
    Root,
    Literal,
    Argument {
        parser: String,
        #[serde(default)]
        properties: HashMap<String, Value>,
    },
}

#[derive(Deserialize)]
struct JsonNode {
    #[serde(flatten)]
    kind: JsonNodeKind,
    #[serde(default)]
    children: FxHashMap<String, JsonNode>,
    #[serde(default)]
    executable: bool,
    #[serde(default)]
    redirect: Vec<String>,
}

pub fn import(json: &str, tree: &mut BuildTree) {
    let node: JsonNode = serde_json::from_str(json).unwrap();

    assert!(matches!(node.kind, JsonNodeKind::Root));
    assert!(!node.executable);

    let mut stack: Vec<(BuildNodeId, &JsonNode)> = Vec::new();
    stack.push((BuildNodeId::ROOT, &node));

    let mut redirects = Vec::new();

    while let Some((parent_id, parent)) = stack.pop() {
        for (child_name, child) in &parent.children {
            let mut node = match &child.kind {
                JsonNodeKind::Root => panic!("encountered root node as child of another node"),
                JsonNodeKind::Literal => Node::literal(child_name.as_str()),
                JsonNodeKind::Argument { parser, properties } => {
                    let param = construct_param(parser.as_str(), properties);
                    Node::argument(child_name.as_str(), param)
                }
            };

            if child.executable {
                node = node.executable();
            }

            let id = tree.insert(parent_id, node);

            if !child.redirect.is_empty() {
                redirects.push((id, child.redirect.as_slice()));
            }

            stack.push((id, child));
        }
    }

    for (source, target_path) in redirects {
        let target = tree
            .find_node_id(target_path)
            .unwrap_or_else(|| panic!("unknown redirect target: {target_path:?}"));
        tree.redirect(source, target);
    }
}

fn construct_param(parser: &str, properties: &HashMap<String, Value>) -> Argument {
    fn get_min_max<T>(
        properties: &HashMap<String, Value>,
        f: fn(&Value) -> Option<T>,
        default_min: T,
        default_max: T,
    ) -> (T, T) {
        let min = properties
            .get("min")
            .map(|v| f(v).unwrap())
            .unwrap_or(default_min);
        let max = properties
            .get("max")
            .map(|v| f(v).unwrap())
            .unwrap_or(default_max);
        (min, max)
    }

    fn get_registry(properties: &HashMap<String, Value>) -> Box<str> {
        properties
            .get("registry")
            .expect("missing registry property")
            .as_str()
            .expect("registry must be a string")
            .into()
    }

    match parser {
        "brigadier:bool" => Argument::Bool,
        "brigadier:double" => {
            let (min, max) = get_min_max(properties, Value::as_f64, f64::MIN, f64::MAX);
            Argument::Double { min, max }
        }
        "brigadier:float" => {
            let mapper = |v: &Value| v.as_f64().map(|num| num as f32);
            let (min, max) = get_min_max(properties, mapper, f32::MIN, f32::MAX);
            Argument::Float { min, max }
        }
        "brigadier:integer" => {
            let mapper = |v: &Value| v.as_i64().map(|num| num as i32);
            let (min, max) = get_min_max(properties, mapper, i32::MIN, i32::MAX);
            Argument::Integer { min, max }
        }
        "brigadier:string" => {
            let kind = match properties.get("type").map(|v| v.as_str().unwrap()) {
                None | Some("word") => StringKind::SingleWord,
                Some("phrase") => StringKind::QuotablePhrase,
                Some("greedy") => StringKind::GreedyPhrase,
                invalid => panic!("invalid type for minecraft:string parser: {invalid:?}"),
            };
            Argument::String(kind)
        }
        "minecraft:angle" => Argument::Angle,
        "minecraft:block_pos" => Argument::BlockPos,
        "minecraft:block_predicate" => Argument::BlockPredicate,
        "minecraft:block_state" => Argument::BlockState,
        "minecraft:color" => Argument::Color,
        "minecraft:column_pos" => Argument::ColumnPos,
        "minecraft:component" => Argument::Component,
        "minecraft:dimension" => Argument::Dimension,
        "minecraft:entity" => {
            let single = match properties.get("amount").map(|v| v.as_str().unwrap()) {
                None | Some("multiple") => false,
                Some("single") => true,
                invalid => panic!("invalid amount for minecraft:entity parser: {invalid:?}"),
            };
            let players_only = match properties.get("type").map(|v| v.as_str().unwrap()) {
                None | Some("entities") => false,
                Some("players") => true,
                invalid => panic!("invalid type for minecraft:entity parser: {invalid:?}"),
            };
            Argument::Entity {
                single,
                players_only,
            }
        }
        "minecraft:entity_anchor" => Argument::EntityAnchor,
        "minecraft:function" => Argument::Function,
        "minecraft:game_profile" => Argument::GameProfile,
        "minecraft:gamemode" => Argument::Gamemode,
        "minecraft:heightmap" => Argument::Heightmap,
        "minecraft:int_range" => Argument::IntRange,
        "minecraft:item_predicate" => Argument::ItemPredicate,
        "minecraft:item_slot" => Argument::ItemSlot,
        "minecraft:item_slots" => Argument::ItemSlots,
        "minecraft:item_stack" => Argument::ItemStack,
        "minecraft:loot_modifier" => Argument::LootModifier,
        "minecraft:loot_predicate" => Argument::LootPredicate,
        "minecraft:loot_table" => Argument::LootTable,
        "minecraft:message" => Argument::Message,
        "minecraft:nbt_compound_tag" => Argument::NbtCompoundTag,
        "minecraft:nbt_path" => Argument::NbtPath,
        "minecraft:nbt_tag" => Argument::NbtTag,
        "minecraft:objective" => Argument::Objective,
        "minecraft:objective_criteria" => Argument::ObjectiveCriteria,
        "minecraft:operation" => Argument::Operation,
        "minecraft:particle" => Argument::Particle,
        "minecraft:resource" => Argument::Resource {
            registry: get_registry(properties),
        },
        "minecraft:resource_key" => Argument::ResourceKey {
            registry: get_registry(properties),
        },
        "minecraft:resource_location" => Argument::ResourceLocation,
        "minecraft:resource_or_tag" => Argument::ResourceOrTag {
            registry: get_registry(properties),
        },
        "minecraft:resource_or_tag_key" => Argument::ResourceOrTagKey {
            registry: get_registry(properties),
        },
        "minecraft:rotation" => Argument::Rotation,
        "minecraft:score_holder" => {
            let single = match properties.get("amount").map(|v| v.as_str().unwrap()) {
                None | Some("multiple") => false,
                Some("single") => true,
                amount => panic!("invalid amount for minecraft:score_holder parser: {amount:?}"),
            };
            Argument::ScoreHolder { single }
        }
        "minecraft:scoreboard_slot" => Argument::ScoreboardSlot,
        "minecraft:style" => Argument::Style,
        "minecraft:swizzle" => Argument::Swizzle,
        "minecraft:team" => Argument::Team,
        "minecraft:template_mirror" => Argument::TemplateMirror,
        "minecraft:template_rotation" => Argument::TemplateRotation,
        "minecraft:time" => {
            let min = properties
                .get("min")
                .map(|v| v.as_i64().unwrap() as i32)
                .unwrap_or(0);
            Argument::Time { min }
        }
        "minecraft:vec2" => Argument::Vec2,
        "minecraft:vec3" => Argument::Vec3,
        _ => panic!("unknown parser: {parser}"),
    }
}
