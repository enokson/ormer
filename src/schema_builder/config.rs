use serde::Deserialize;
use std::{borrow::Cow, collections::BTreeMap};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RelationInput {
    pub name: Option<String>,
    pub fields: Option<Vec<String>>,
    pub references: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManyToManyConfig<'a> {
    pub name: &'a str,
    pub model_one_name: &'a str,
    pub model_one_members: &'a [Cow<'a, String>],
    pub model_two_name: &'a str,
    pub model_two_members: &'a [Cow<'a, String>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationOptions<'a> {
    // create a name if one does not exist
    pub name: Option<Cow<'a, String>>,
    // options that are only needed for the name will not include a fields member
    pub fields: Option<&'a [String]>,
    // options that are only needed for the name will not include a references member
    pub references: Option<&'a [String]>,
    // is used when no more mutations occure
    pub completed: bool,
    // is true if the system had to create a name
    pub generated_name: Option<bool>,
    // is true if the system had to create this options (for many-to-many relationships)
    pub generated_options: bool,
    // is true if the relation is many-to-many
    pub many_to_many_config: Option<ManyToManyConfig<'a>>,
}

impl<'a> RelationOptions<'a> {
    /// Used when there is a many-to-many relation
    pub fn create_sister(&self) -> Self {
        Self {
            name: match &self.name {
                Some(name) => Some(name.clone()),
                None => None
            },
            fields: match &self.fields {
                Some(f) => Some(f),
                None => None
            },
            references: match &self.references {
                Some(r) => Some(r),
                None => None
            },
            completed: self.completed,
            generated_name: self.generated_name,
            generated_options: self.generated_options,
            many_to_many_config: match &self.many_to_many_config {
                Some(c) => Some(c.clone()),
                None => None
            },
        }
    }

    /// from input
    /// This is used to copy the contents of the config file while added
    /// needed parameters to the config.
    fn from_input(i: &'a RelationInput) -> Self {
        Self {
            name: match &i.name {
                  Some(n) => Some(Cow::Borrowed(n)),
                  None => None
            },
            fields: match &i.fields {
                Some(f) => Some(f.as_slice()),
                None => None
            },
            references: match &i.references {
                Some(r) => Some(r.as_slice()),
                None => None
            },
            completed: false,
            generated_name: None,
            generated_options: false,
            many_to_many_config: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum DefaultType {
    Uuid,
    AutoInc,
    Now,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ModelMemberInput {
    String(String),
    Props {
        #[serde(rename(deserialize = "type"))]
        member_type: String,
        is_id: Option<bool>,
        default: Option<DefaultType>,
        is_list: Option<bool>,
        relation: Option<RelationInput>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelMemberOptions<'a> {
    pub member_type: &'a str,
    pub is_id: bool,
    pub default: Option<&'a DefaultType>,
    pub is_list: bool,
    pub relation: Option<RelationOptions<'a>>,
}

impl<'a> Default for ModelMemberOptions<'a> {
    fn default() -> Self {
        Self {
            member_type: "fake",
            is_id: false,
            default: None,
            is_list: false,
            relation: None,
        }
    }
}

impl<'a> ModelMemberOptions<'a> {
    pub fn from_input(i: &'a ModelMemberInput) -> Self {
        match i {
            ModelMemberInput::Props { member_type, is_id, default, is_list, relation } => {
                Self {
                    member_type,
                    is_id: match &is_id {
                        Some(cond) => *cond,
                        None => false
                    },
                    default: match &default {
                        Some(default) => Some(default),
                        None => None
                    },
                    is_list: match &is_list {
                        Some(cond) => *cond,
                        None => false
                    },
                    relation: match &relation {
                        Some(r) => Some(RelationOptions::from_input(r)),
                        None => None
                    },
                }
            },
            ModelMemberInput::String(member_type) => {
                Self {
                    member_type,
                    is_id: false,
                    default: None,
                    is_list: false,
                    relation: None,
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModelInput {
    pub table_name: Option<String>,
    pub model_name: Option<String>,
    pub members: BTreeMap<String, ModelMemberInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOptions<'a> {
    pub table_name: Option<&'a str>,
    pub model_name: Option<&'a str>,
    pub members: BTreeMap<&'a str, ModelMemberOptions<'a>>,
}

impl<'a> ModelOptions<'a> {
    pub fn from_input(i: &'a ModelInput) -> Self {
        let mut members: BTreeMap<&'a str, ModelMemberOptions<'a>> = BTreeMap::new();
        for (n, o) in i.members.iter() {
            members.insert(n, ModelMemberOptions::from_input(o));
        }
        Self {
            table_name: match &i.table_name {
                Some(n) => Some(n),
                None => None
            },
            model_name: match &i.model_name {
                Some(n) => Some(n),
                None => None
            },
            members,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatabaseInput {
    #[serde(rename(deserialize = "type"))]
    pub database_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseOptions<'a> {
    pub database_type: &'a str,
}

impl<'a> DatabaseOptions<'a> {
    pub fn from_input(i: &'a DatabaseInput) -> Self {
        Self {
            database_type: &i.database_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SchemaInput {
    pub database: DatabaseInput,
    pub models: BTreeMap<String, ModelInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaOptions<'a> {
    pub database: DatabaseOptions<'a>,
    pub models: BTreeMap<&'a str, ModelOptions<'a>>,
}

impl<'a> SchemaOptions<'a> {
    pub fn from_input(i: &'a SchemaInput) -> Self {
        let mut models: BTreeMap<&'a str, ModelOptions<'a>> = BTreeMap::new();
        for (n, o) in i.models.iter() {
            models.insert(n, ModelOptions::from_input(o));
        }
        Self {
            database: DatabaseOptions::from_input(&i.database),
            models,
        }
    }
    
    pub fn from_outer_input(i: &'a OuterSchemaInput) -> Self {
        Self::from_input(&i.schema)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OuterSchemaInput {
    pub schema: SchemaInput,
}

