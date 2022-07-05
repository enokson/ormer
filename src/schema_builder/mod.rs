use serde::Deserialize;
use serde_json::{
    Value,
    from_value
};
use std::{
    collections::{
        BTreeMap,
        BTreeSet
    }
};

// String
// Boolean
// Int
// BigInt
// Float
// Decimal
// DateTime
// Json
// Bytes
// Unsupported

#[derive(Debug, Clone, Deserialize)]
pub enum Datatype {
    String,
    Boolean,
    Int,
    BigInt,
    Float,
    Decimal,
    DateTime,
    Json,
    Bytes
}

#[derive(Debug, Clone, Deserialize)]
pub enum AutoInc {
    Int,
    BigInt,
    None
}

// autoincrement()
// sequence() (CockroachDB only)
// dbgenerated()
// cuid()
// uuid()
// now()

#[derive(Debug, Clone, Deserialize)]
pub enum DefaultValue {
    AutoInc,
    Uuid,
    Now
}

#[derive(Debug, Clone, Deserialize)]
pub struct Reference {
    pub name: Option<String>,
    pub fields: Vec<String>,
    pub referencs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum AttribeType {
    List(Vec<String>),
    String(String),
    Obj {
        value_type: String,
        primary_key: Option<DefaultValue>,
        nullable: Option<bool>,
        reference: Option<Reference>
    }
}

pub mod attribute {
    pub const PRIMARY_KEY: &'static str = "PrimaryKey";
    pub const UNIQUE: &'static str = "Unique";
    pub const OPTIONAL: &'static str = "?";
}

#[derive(Debug, Clone, Deserialize)]
pub enum Attribute {
    PrimaryKey(DefaultValue),
    Unique,
    Relation {
        name: Option<String>,
        fields: Vec<String>,
        referencs: Vec<String>,
    },
    Optional
}

pub struct ReferenceInfo {
    models: [String; 2],
    pairs: Vec<(String, String)>, // vec![ (User.uuid, Post.user_uuid) ]
}

pub fn validate_type_names(config: &Value) {
    let models: BTreeMap<String, Value> = from_value(config["database"]["models"].clone()).expect("Could not parse schema.database.models");
    let mut existing_models: BTreeSet<String> = BTreeSet::new();

    // the key is the model.member. the value are pending failures: MissingReference, ReferenceRequiresName, CannotFindType,
    let mut member_types: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut many_to_many_tables: Vec<String> = Vec::new(); // "User.uuid Post.user_id" 
    for model_name in models.keys() {

        // check if model already exists
        {
            if existing_models.contains(model_name) {
                panic!("multiples of model `{}` found", model_name);
            }
            existing_models.insert(model_name.clone());
        }
        
    }

    for (model_name, members_as_value) in &models {
        let members: BTreeMap<String, String> = from_value(members_as_value.clone())
            .expect(&format!("Could not parse schema.database.models.{}.members", model_name));

        for (member_name, attributes) in members {
            let seperated_types_with_atributes = attributes.split(" ").map(|v| v.to_string()).collect::<Vec<String>>();
            let mut attributes_to_search_for: [&'static str; 2] = ["[]", "?"];
            let mut is_optional = false;
            let mut is_list = false;
            let data_type = &seperated_types_with_atributes[0];
            for i in 1..seperated_types_with_atributes.len() {
                if seperated_types_with_atributes[i] == "[]" {
                    is_list = true;
                } else if seperated_types_with_atributes[i] == "?" {
                    is_optional = true;
                } else if seperated_types_with_atributes[i].contains("Ref") {
                    let options = seperated_types_with_atributes[i]
                        .replace("Ref(", "")
                        .replace(")", "")
                        .split(" ")
                        .map(|v| v.to_string());
                    // regex: /^\w+  => get first word
                }
            }


        }
        
        // validate memebers
        {

        }
    }
    
}