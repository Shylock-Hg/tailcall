use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

use crate::core::config::{Config, Type, Enum};
use crate::core::valid::{Valid, Validator};
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;

/// The @lint directive allows you to configure linting.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema, MergeRight)]
pub struct Lint {
    ///
    /// To autoFix the lint.
    /// Example Usage lint:{autoFix:true}
    #[serde(rename = "autoFix")]
    pub auto_fix: Option<bool>,
    ///
    ///
    /// This enum is provided with appropriate TextCase.
    /// Example Usage: lint:{enum:Pascal}
    #[serde(rename = "enum")]
    pub enum_lint: Option<bool>,
    ///
    ///
    /// This enumValue is provided with appropriate TextCase.
    /// Example Usage: lint:{enumValue:ScreamingSnake}
    #[serde(rename = "enumValue")]
    pub enum_value_lint: Option<bool>,
    ///
    ///
    /// This field is provided with appropriate TextCase.
    /// Example Usage: lint:{field:Camel}
    #[serde(rename = "field")]
    pub field_lint: Option<bool>,
    ///
    ///
    /// This type is provided with appropriate TextCase.
    /// Example Usage: lint:{type:Pascal}
    #[serde(rename = "type")]
    pub type_lint: Option<bool>,
}

impl Default for Lint {
    fn default() -> Self {
        Self {
            auto_fix: Some(true),
            enum_lint: Some(true),
            enum_value_lint: Some(true),
            field_lint: Some(true),
            type_lint: Some(true),
        }
    }
}

const TYPE_NAME_CASE: Case = Case::Pascal;
const FIELD_NAME_CASE: Case = Case::Camel;
const ENUM_NAME_CASE: Case = Case::Pascal;
const ENUM_VALUE_CASE: Case = Case::UpperSnake;

fn is_type(type_name: &str, types: &BTreeMap<String, Type>) -> bool {
    types.contains_key(type_name)
}

fn is_enum(enum_name: &str, enums: &BTreeMap<String, Enum>) -> bool {
    enums.contains_key(enum_name)
}

impl Lint {
    pub fn lint(&self, config: Config) -> Valid<Config, String> {
        let auto_fix = self.auto_fix.map_or(false, |lint| lint);
        let lint_type_name_case = self.type_lint.map_or(true, |l| l);
        let lint_field_name_case = self.field_lint.map_or(true, |l|l);
        let lint_enum_name_case = self.enum_lint.map_or(true, |l| l);
        let lint_enum_value_case = self.enum_value_lint.map_or(true, |l| l);
        let mut new_config = config.clone();
        // types
        let types = Valid::from_iter(config.types, |(type_name, mut ty)| {
            let type_valid = if lint_type_name_case && !type_name.is_case(TYPE_NAME_CASE) {
                if auto_fix {
                    Valid::succeed(type_name.to_case(TYPE_NAME_CASE))
                } else {
                    Valid::fail("Type name {type_name} should be PascalCase, you can use @modify to correct it.".to_owned())
                }
            } else {
                Valid::succeed(type_name)
            };
            let field_valid = Valid::from_iter(ty.fields.iter(), |(field_name, field)| {
                if lint_field_name_case && !field_name.is_case(FIELD_NAME_CASE) {
                    if auto_fix {
                        Valid::succeed((field_name.to_case(FIELD_NAME_CASE), field.clone()))
                    } else {
                        Valid::fail("Field name {field_name} of {type_name} should be camelCase, you can use @modify to correct it.".to_owned())
                    }
                } else {
                    Valid::succeed((field_name.to_owned(), field.clone()))
                }
            }).map(|fields| {
                ty.fields.clear();
                ty.fields.extend(fields);
                ty
            });
            type_valid.fuse(field_valid).into()
        });

        let enums = Valid::from_iter(config.enums, |(enum_name, mut eu)| {
            let enum_valid = if lint_enum_name_case && !enum_name.is_case(ENUM_NAME_CASE) {
                if auto_fix {
                    Valid::succeed(enum_name.to_case(ENUM_NAME_CASE))
                } else {
                    Valid::fail("Enum name {enum_name} should be PascalCase, you can use @modify to correct it.".to_owned())
                }
            } else {
                Valid::succeed(enum_name)
            };
            let enum_value_valid = Valid::from_iter(eu.variants.iter(), |variant| {
                if lint_enum_value_case && !variant.is_case(ENUM_VALUE_CASE) {
                    if auto_fix {
                        Valid::succeed(variant.to_case(ENUM_VALUE_CASE))
                    } else {
                        Valid::fail("Enum value {variant} of {enum_name} should be ALL_CAPS, you can use @modify to correct it.".to_owned())
                    }
                } else {
                    Valid::succeed(variant.clone())
                }
            }).map(|enum_values| {
                eu.variants.clear();
                eu.variants.extend(enum_values);
                eu
            });
            enum_valid.fuse(enum_value_valid).into()
        });
        types.map(|types| {
            new_config.types.clear();
            new_config.types.extend(types);
            new_config
        }).fuse(enums).map(|(mut config, enums)| {
            config.enums.clear();
            config.enums.extend(enums);
            config
        }).into()
    }
}
