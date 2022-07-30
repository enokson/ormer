pub mod config;
pub mod directives;
pub mod error;
pub mod validate;

use directives::{Directives, RelationDirectiveInfo};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};
use serde_json::{from_value, Value};
use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    marker::{Copy, PhantomData},
    rc::Rc,
};

// #[derive(Debug)]
// pub struct ConfigError {
//     pub ln: i32,
//     pub col: i32,
//     pub msg: String,
//     pub child: Option<>
// }
// impl Display for ConfigError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[Ormer Configuration Error: ln]")
//     }
// }

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
enum Datatype {
    String,
    Boolean,
    Int,
    BigInt,
    Float,
    Decimal,
    DateTime,
    Json,
    Bytes,
}

const SCALAR_TYPES: &'static [&str] = &[
    "String", "Boolean", "Int", "BigInt", "Float", "Decimal", "DateTime", "Json", "Bytes",
];

// autoincrement()
// sequence() (CockroachDB only)
// dbgenerated()
// cuid()
// uuid()
// now()

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum DefaultValue {
    AutoInc,
    Uuid,
    Now,
}

impl Copy for DefaultValue {}

mod helpers {
    use serde::{
        de::{Deserializer, MapAccess, Visitor},
        Deserialize,
    };
    use std::{
        collections::{BTreeMap, BinaryHeap},
        fmt,
        marker::PhantomData,
        rc::Rc,
    };

    pub struct RefKeyMap<K: Ord, V>(pub BTreeMap<K, V>);
    // A Visitor is a type that holds methods that a Deserializer can drive
    // depending on what is contained in the input data.
    //
    // In the case of a map we need generic type parameters K and V to be
    // able to set the output type correctly, but don't require any state.
    // This is an example of a "zero sized type" in Rust. The PhantomData
    // keeps the compiler from complaining about unused generic type
    // parameters.
    struct MyMapVisitor<K: Ord, V> {
        marker: PhantomData<fn() -> RefKeyMap<K, V>>,
    }

    impl<K: Ord, V> MyMapVisitor<K, V> {
        fn new() -> Self {
            MyMapVisitor {
                marker: PhantomData,
            }
        }
    }

    // This is the trait that Deserializers are going to be driving. There
    // is one method for each type of data that our type knows how to
    // deserialize from. There are many other methods that are not
    // implemented here, for example deserializing from integers or strings.
    // By default those methods will return an error, which makes sense
    // because we cannot deserialize a MyMap from an integer or string.
    impl<'de, K: Ord, V> Visitor<'de> for MyMapVisitor<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        // The type that our Visitor is going to produce.
        type Value = RefKeyMap<Rc<K>, V>;

        // Format a message stating what data this Visitor expects to receive.
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a very special map")
        }

        // Deserialize MyMap from an abstract "map" provided by the
        // Deserializer. The MapAccess input is a callback provided by
        // the Deserializer to let us see each entry in the map.
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            // let mut map = MyMap::with_capacity(access.size_hint().unwrap_or(0));
            let mut map = RefKeyMap(BTreeMap::new());

            // While there are entries remaining in the input, add them
            // into our map.
            while let Some((key, value)) = access.next_entry()? {
                map.0.insert(Rc::new(key), value);
            }

            Ok(map)
        }
    }

    // This is the trait that informs Serde how to deserialize MyMap.
    impl<'de, K: Ord, V> Deserialize<'de> for RefKeyMap<Rc<K>, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Instantiate our Visitor and ask the Deserializer to drive
            // it over the input data, resulting in an instance of MyMap.
            deserializer.deserialize_map(MyMapVisitor::new())
        }
    }
}

enum RelationType {
    OneToOne,   // both sides are one to one
    OneToMany,  // one of this model maps to many of the reference model
    ManyToMany, // both are many to many
    None,
}

struct ParsedRelationDirective {
    pub relation_name: Rc<String>,
    pub model_1: Rc<String>,
    pub model_1_member: Rc<String>,
    pub model_1_type: RelationType,
    pub model_2: Rc<String>,
    pub model_2_member: Rc<String>,
    pub model_2_type: RelationType,
}

struct ParsedRelationDirective2 {}

struct ParsedDirectives {
    pub member_type: Rc<String>,
    pub is_array: bool,
    pub is_optional: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: Option<DefaultValue>,
    pub relation: Option<Rc<ParsedRelationDirective>>,
}

type Members = BTreeMap<Rc<String>, ParsedDirectives>;
type Models = BTreeMap<Rc<String>, Members>;
type ManyToManyRelationships = BTreeMap<Rc<RefCell<String>>, Rc<ParsedRelationDirective>>;

struct Config {
    pub models: Models,
    pub many_to_many_relationships: ManyToManyRelationships,
}

pub fn parse_config(config: &Value) {
    // get the 'models' object from the configuration file
    let models_str_list: helpers::RefKeyMap<Rc<String>, Value> =
        from_value(config["database"]["models"].clone())
            .expect("Could not parse schema.database.models");
    let models: BTreeMap<Rc<String>, Value> = BTreeMap::new();
    // ensure that the schema has only one of each model defined
    {
        let mut existing_models: BTreeSet<Rc<String>> = BTreeSet::new();
        for model_name in models_str_list.0.keys() {
            if existing_models.contains(model_name) {
                panic!("multiples of model `{}` found", model_name);
            }
            existing_models.insert(model_name.clone());
        }
    }
    // parse and store each model defined
    let mut models_info: BTreeMap<Rc<String>, BTreeMap<Rc<String>, Rc<Directives>>> =
        BTreeMap::new();
    for (model_name, members_as_value) in &models {
        // grab each member and their directives string
        let member_and_directive_strings: BTreeMap<String, String> =
            from_value(members_as_value.clone()).expect(&format!(
                "Could not parse schema.database.models.{}.members",
                model_name
            ));
        // check if any members exist more than once
        {
            let mut member_names: BTreeSet<Rc<String>> = BTreeSet::new();
            for name in member_and_directive_strings.keys() {
                if member_names.contains(name) {
                    panic!("member {} is listed more than once.", name);
                }
                member_names.insert(Rc::new(name.clone()));
            }
        }
        // parse and store directives
        let mut members_info: BTreeMap<Rc<String>, Rc<Directives>> = BTreeMap::new();
        // ensure each model has one and only one primary key
        {
            let mut primary_key_count = 0;
            for (member_name, directives) in member_and_directive_strings {
                // parse directives
                let directives_info: Directives = directives::parse_directives_str(&directives);
                members_info.insert(Rc::new(member_name), Rc::new(directives_info.clone()));
                if directives_info.is_primary_key {
                    primary_key_count += 1;
                }
            }
            if primary_key_count != 1 {
                panic!("model {} must have only one primary key.", model_name);
            }
        }
        models_info.insert(model_name.clone(), members_info);
    }
    {
        // store the system generated names: key: (<model name>, <member name>), value: <system generated name>
        let mut relations_names_to_change: BTreeMap<(Rc<String>, Rc<String>), Rc<RefCell<String>>> =
            BTreeMap::new();
        // store the many-to-many relationships to be added. key: (<model name>, <member name>), value: <RelationDirective>
        let mut relation_directives_to_add: BTreeMap<
            (Rc<String>, Rc<String>),
            Rc<RefCell<RelationDirectiveInfo>>,
        > = BTreeMap::new();
        // TODO: store parsed relationships
        let mut parsed_relation_directives: BTreeMap<
            Rc<String>,
            BTreeMap<Rc<String>, ParsedRelationDirective>,
        > = BTreeMap::new();

        for (model_name, members_info) in &models_info {
            let unnamed_model_relation_count = members_info
                .values()
                .filter(|directives| {
                    if let Some(relation_directive) = &directives.relation {
                        relation_directive.as_ref().borrow().name.is_none()
                    } else {
                        false
                    }
                })
                .count();
            if unnamed_model_relation_count > 1 {
                panic!("There are ambiguous relations in {}", model_name);
            }
            // ensure no other member has the same relation name
            {
                let mut list = members_info
                    .values()
                    .filter_map(|directives| {
                        if let Some(relation_directive) = &directives.relation {
                            if let Some(name) = &relation_directive.as_ref().borrow().name {
                                Some(name.as_ref().borrow().as_str().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>();
                let starting_count = list.len();
                list.dedup();
                let ending_count = list.len();
                if starting_count > ending_count {
                    panic!(
                        "model {} has multiple relation directives with the same name",
                        model_name
                    );
                }
            }

            for (member_name, member) in members_info {
                // validate the types as scalar or user defined
                if !models_info.contains_key(&member.member_type)
                    && !SCALAR_TYPES.contains(&member.member_type.as_str())
                {
                    panic!(
                        "type not found: {}/{}/{}",
                        model_name, member_name, member.member_type
                    )
                }
                // list system generated names to know if one has been used before
                // if one has been used before that means that two models share more than one relation,
                // the user hasn't named them, and they need to be disambiguated.
                let mut system_generated_relation_names: BTreeSet<Rc<RefCell<String>>> =
                    BTreeSet::new();
                if let Some(member_relation) = &member.relation {
                    let relation_info: Ref<RelationDirectiveInfo> =
                        member_relation.as_ref().borrow();
                    // ensure that the type being referenced is not a scalar
                    // this is technically redundant as all relations are checked against its partner model,
                    // but this cause the app to fail earlier and have a different error message
                    if SCALAR_TYPES.contains(&member.member_type.as_str()) {
                        panic!(
                            "{}/{} contains a reference to a scalar",
                            model_name, member_name
                        );
                    }

                    let _relation_name = match &relation_info.name {
                        Some(name) => name.clone(),
                        None => {
                            // if the config does not name the relation, create a name
                            let system_generated_name = {
                                let mut list = [member_name.clone(), member.member_type.clone()];
                                list.sort();
                                Rc::new(RefCell::new(format!("ormer#{}{}", list[0], list[1])))
                            };
                            // check if the name exists already
                            if system_generated_relation_names
                                .contains(system_generated_name.as_ref())
                            {
                                panic!(
                                    "relations between {} and {} needs to be disambiguated",
                                    member_name, member.member_type
                                );
                            }
                            system_generated_relation_names.insert(system_generated_name.clone());
                            // store the new generated name
                            relations_names_to_change.insert(
                                (model_name.clone(), member_name.clone()),
                                system_generated_name.clone(),
                            );
                            system_generated_name
                        }
                    };
                    // ensure that the model has the member that is listed as a field
                    for field in relation_info.fields.as_ref() {
                        if !members_info.contains_key(field) {
                            panic!("model '{}' does not contain member '{}'", model_name, field)
                        }
                    }
                    // ensure that the other model being referenced has the member being referenced
                    for member_relation_directive in relation_info.references.as_ref() {
                        // get the referenced model
                        let reference_model = match models_info.get(&member.member_type) {
                            Some(reference_model) => reference_model,
                            None => {
                                panic!(
                                    "cannot find model {} as referenced by {}/{}",
                                    &member.member_type, model_name, member_name
                                )
                            }
                        };
                        // get the referenced member
                        let _reference_member_directives =
                            match reference_model.get(&member.member_type) {
                                Some(reference_member_directives) => reference_member_directives,
                                None => panic!(
                                    "cannot find model {}/{} as referenced by {}/{}",
                                    &member.member_type,
                                    member_relation_directive,
                                    model_name,
                                    member_name
                                ),
                            };
                    }
                }

                // check if member is an array
                if member.is_array {
                    // it is a many-to-many-relationship or a many-to-one
                    // get the reference model being referred to
                    let reference_model = models_info.get(&member.member_type).unwrap();
                    // find the member of the model that also has the inverse + array (if there is one)
                    let mut many_to_many_relationship: Option<(Rc<String>, Rc<Directives>)> = None;
                    'many_to_many_check: for (reference_member_name, reference_member) in
                        reference_model
                    {
                        // if the names don't match then its not the same relation
                        if reference_member_name.as_str() != member_name.as_str() {
                            continue 'many_to_many_check;
                        }
                        // if the reference member is not an array then its not a many-to-many relation
                        if reference_member.is_array == member.is_array {
                            many_to_many_relationship =
                                Some((reference_member_name.clone(), reference_member.clone()));
                            break 'many_to_many_check;
                        }
                    }
                    // check again if the relationship is many-to-many
                    if let Some((_referenced_member_name, referenced_member)) =
                        many_to_many_relationship
                    {
                        // is many-to-many
                        // TODO: validate that if the reference has a reference directive, it is the same name
                        if member.relation.is_some() && referenced_member.relation.is_some() {
                            let member_relation =
                                member.relation.as_ref().unwrap().as_ref().borrow();
                            let reference_relation = referenced_member
                                .relation
                                .as_ref()
                                .unwrap()
                                .as_ref()
                                .borrow();
                            if member_relation.name != reference_relation.name {
                                // TODO: return error, the relation name do not match
                            }
                        } else if (!member.relation.is_some()
                            && referenced_member.relation.is_none())
                            || (!member.relation.is_none() && referenced_member.relation.is_some())
                        {
                            // TODO return error, the relation directives are not matches
                        }
                        // TODO: add an entry into the many-to-many map
                        // TODO: add a relation entry for this member (not the other, while this is less performant, I just want something that works for now)
                    } else {
                        // is one-to-many
                        // TODO: add a relation entry for this member
                    }
                } else {
                    // it is either a one-to-many relationship or a one-to-one or none
                    if let Some(member_relation) = member.relation.as_ref() {
                        // TODO: add a relation entry for this member
                    } // else is not a relation
                }

                // TODO: ensure that each one-to-one or one-to-many relationship has a relation directive
                // this is done by checking if the type is an array or if the member in the other model is an array

                // TODO: ensure that each member of each model that has a many-to-many relationship also has a relation directive
                // this needed to create the query objects
            }
        }

        {
            // apply system generated names to relations not named by configuration
            for ((model_name, member_name), new_relation_name) in &relations_names_to_change {
                let model = models_info.get_mut(model_name).unwrap();
                let member = model.get_mut(member_name).unwrap();
                let mut relation = member.relation.as_ref().unwrap().borrow_mut();
                // let relation = relation_ref.as_ref().borrow_mut();
                relation.name = Some(new_relation_name.clone());
                /*              let relation = member.relations.get_mut((*iteration) as usize).unwrap();
                relation.name = Some(new_relation_name.clone()); */
            }
            // apply inverse relation directives
            for ((model_name, member_name), new_directive) in &relation_directives_to_add {
                let model = models_info.get_mut(model_name).unwrap();
                let mut member_directives = model.get_mut(member_name).unwrap();
                // ! TODO: FIX and uncomment
                // ? can this be removed now?
                // member_directives.relation = Some(new_directive.clone());
            }
        }
    }
    // list tables that will need to be created. key: <relation name>, value: ((<first model>, <member name>),(second model, member name))
    let mut many_to_many_tables: BTreeMap<
        Rc<RefCell<String>>,
        ((Rc<String>, Rc<String>), (Rc<String>, Rc<String>)),
    > = BTreeMap::new();
    {
        // validate relations
        let mut completed_relations: BTreeSet<Rc<RefCell<String>>> = BTreeSet::new();
        for (model_name, members_info) in &models_info {
            for (_member_name, directives_info) in members_info {
                // let mut system_generated_relation_names: BTreeSet<String> = BTreeSet::new();
                if let Some(relation_info_ref) = directives_info.relation.as_ref() {
                    let relation_info: Ref<RelationDirectiveInfo> =
                        relation_info_ref.as_ref().borrow();
                    // ensure that relation being validated wasn't already completed,
                    // which, it might because the loop here touch both models
                    // and will replicate the entry
                    if !completed_relations.contains(relation_info.name.as_ref().unwrap()) {
                        let other_model = models_info.get(&directives_info.member_type).unwrap();
                        let other_member = other_model.get(&directives_info.member_type).unwrap();
                        // if both models are arrays then a many-to-many table needs to be created
                        if directives_info.is_array && other_member.is_array {
                            // ignore fields and references for now, just use the primary key
                            let relation_name = relation_info.name.as_ref().unwrap();
                            // get the first model's primary key
                            let model_primary_key = {
                                let mut primary_key: Option<Rc<String>> = None;
                                for (model_member_name, model_directive) in members_info {
                                    if model_directive.is_primary_key {
                                        primary_key = Some(model_member_name.clone());
                                    }
                                }
                                primary_key.unwrap()
                            };
                            // get the second model's primary key
                            let other_model_primary_key = {
                                let mut primary_key: Option<Rc<String>> = None;
                                for (model_member_name, model_directive) in other_model {
                                    if model_directive.is_primary_key {
                                        primary_key = Some(model_member_name.clone());
                                    }
                                }
                                primary_key.unwrap()
                            };
                            // only place in list if it doesn't exist already,
                            // which, it might because the loop here touch both models
                            // and will replicate the entry
                            if !many_to_many_tables.contains_key(relation_name) {
                                let relationship = (
                                    (model_name.clone(), model_primary_key),
                                    (directives_info.member_type.clone(), other_model_primary_key),
                                );
                                many_to_many_tables.insert(relation_name.clone(), relationship);
                            }
                        }
                        completed_relations.insert(relation_info.name.as_ref().unwrap().clone());
                    }
                }
            }
        }
    }
}

pub mod parse {
    use super::{
        directives::{parse_directives, pipe, Directives2, RelationDirective2},
        error::*,
        helpers::RefKeyMap,
    };
    use serde_json::{from_str, from_value, Value};
    use std::{
        collections::{btree_map::Keys, BTreeMap},
        mem,
        rc::Rc,
    };

    struct PartialConfig {
        models: BTreeMap<Rc<String>, BTreeMap<Rc<String>, Rc<Directives2>>>,
    }

    fn database_obj_check(possible_obj: &Value) -> Result<(), StackError> {
        if !possible_obj.is_object() {
            return Err(StackError::new_wo_error(
                OrmerError::UserConfigError,
                file!(),
                line!(),
                Some("Expect schema/database to be an object"),
            ));
        }
        return Ok(());
    }

    fn models_obj_check(possible_obj: &Value) -> Result<(), StackError> {
        if !possible_obj.is_object() {
            return Err(StackError::new_wo_error(
                OrmerError::UserConfigError,
                file!(),
                line!(),
                Some("Expect schema/database/models to be an object"),
            ));
        }
        return Ok(());
    }

    fn parse_model_obj_list(
        mut models_obj_value: Value,
    ) -> Result<RefKeyMap<Rc<String>, String>, StackError> {
        match from_value(mem::take(&mut models_obj_value)) {
            Ok(list) => Ok(list),
            Err(err) => {
                return Err(StackError::new(
                    OrmerError::UserConfigError,
                    file!(),
                    line!(),
                    Some("Could not parse schema/database/models"),
                    Some(err),
                ))
            }
        }
    }

    fn parse_model_obj(
        mut model_obj_list: RefKeyMap<Rc<String>, String>,
    ) -> Result<BTreeMap<Rc<String>, Directives2>, StackError> {
        let model_names: Vec<Rc<String>> = model_obj_list
            .0
            .keys()
            .map(|key| key.clone())
            .collect::<Vec<Rc<String>>>();
        let mut models_list: BTreeMap<Rc<String>, Directives2> = BTreeMap::new();
        for model_name in model_names {
            let directives_string = model_obj_list.0.remove(&model_name).unwrap();
            let directives = match parse_directives(directives_string) {
                Ok(directives) => directives,
                Err(err) => {
                    return Err(StackError::new(
                        OrmerError::UserConfigError,
                        file!(),
                        line!(),
                        Some(format!("Could not parse /schema/models/{}", model_name)),
                        Some(err),
                    ))
                }
            };
            models_list.insert(model_name.clone(), directives);
        }
        Ok(models_list)
    }

    fn parse_config(config_string: String) -> Result<(), StackError> {
        let mut config_as_value: Value = match from_str(&config_string) {
            Ok(partial_config) => partial_config,
            Err(err) => {
                return Err(StackError::new(
                    OrmerError::ParsingError,
                    file!(),
                    line!(),
                    Some("Could not parse the schema config file."),
                    Some(err),
                ))
            }
        };
        let mut database_obj_value = mem::replace(&mut config_as_value["database"], Value::Null);
        database_obj_check(&database_obj_value)?;
        let mut models_obj_value = mem::replace(&mut database_obj_value["models"], Value::Null);
        models_obj_check(&models_obj_value)?;
        let model_obj_value_list = parse_model_obj_list(mem::take(&mut models_obj_value))?;
        // let models_list =
        Ok(())
    }
}
