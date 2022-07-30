use std::collections::BTreeMap;
use super::{config::*, Config};

const SCALAR_TYPES: &'static [&str] = &[
    "String",
    "Int",
    "Float",
    "Uuid",
    "DateTime",
];

fn get_model_types<'a>(models: &BTreeMap<&'a str, ModelOptions<'a>>) -> Vec<&'a str> {
    let mut model_types: Vec<&'a str> = Vec::with_capacity(models.len());
    for n in models.keys() {
        if !is_scalar(n) {
            model_types.push(n);
        }
    }
    model_types
}

fn is_valid_type(n: &str, models: &[&str]) -> bool {
    let mut is_valid = false;
    for s in SCALAR_TYPES.iter() {
        if &n == s {
            is_valid = true;
            break;
        }
    }
    if !is_valid {
        for m in models.iter() {
            if &n == m {
                is_valid = true;
                break;
            }
        }
    }
    is_valid
} 

fn is_unique(n: &str, list: &[&str]) -> bool {
    let mut count = 0;
    for i in list {
        if &n == i {
            count += 1;
            if count > 1 {
                return false
            }
        }
    }
    true
}

fn model_name_is_unique(n: &str, list: &[&str]) -> bool {
    is_unique(n, list)
}

fn member_name_is_unique(n: &str, list: &[&str]) -> bool {
    is_unique(n, list)
}

fn has_unique_id(m: &BTreeMap<&str, ModelMemberOptions>) -> bool {
    let mut count = 0;
    for o in m.values() {
        if o.is_id {
            count += 1;
            if count > 1 {
                return false;
            }
        }
    }
    if count < 1 {
        return false;
    }
    true
}

fn relation_field_exists<'a>(f: &'a str, members: &[&'a str]) -> bool {
    members.contains(&f)
}

fn missing_relation_field<'a>(fields: &'a [String], members: &[&'a str]) -> Option<&'a str> {
    for f in fields.iter() {
        if !relation_field_exists(f, members) {
            return Some(f);
        }
    }
    None
}

fn get_member_list<'a>(list: &'a BTreeMap<&'a str,  ModelMemberOptions<'a>>) -> Vec<&'a str> {
    let mut keys: Vec<&'a str> = Vec::with_capacity(list.len());
    for (k, _v) in list {
        keys.push(k);
    }
    keys
}

fn get_member_list_from_model<'a>(m: &'a str, models: &'a BTreeMap<&'a str, ModelOptions<'a>>) -> Vec<&'a str> {
    get_member_list(&models.get(m).unwrap().members)
}

fn missing_releation_fields<'a>(m: &'a str, fields: &'a [String], models: &'a BTreeMap<&'a str, ModelOptions<'a>>) -> Option<&'a str> {
    let members = &get_member_list_from_model(m, models);
    missing_relation_field(fields, members)
}

fn missing_relation_reference<'a>(m: &'a str, references: &'a [String], models: &'a BTreeMap<&'a str, ModelOptions<'a>>) -> Option<&'a str> {
    let members = &get_member_list_from_model(m, models);
    missing_relation_field(references, members)
}

fn is_scalar<'a>(t: &'a str) -> bool {
    SCALAR_TYPES.contains(&t)
}

fn non_scalars<'a>(models: &'a BTreeMap<&'a str, ModelMemberOptions<'a>>) -> Vec<&'a str> {
    let mut non_scalars: Vec<&'a str> = vec![];
    for k in models.keys() {
        if !is_scalar(k) {
            non_scalars.push(k);
        }
    }
    non_scalars
}

fn has_relation<'a>(m: &ModelMemberOptions<'a>) -> bool {
    m.relation.is_some()
}

fn has_name<'a>(r: &RelationOptions<'a>) -> bool {
    r.name.is_some()
}

fn relation_counts<'a>(members: &'a BTreeMap<&'a str, ModelMemberOptions<'a>>) -> BTreeMap<&'a str, i32> {
    let mut relationships: BTreeMap<&'a str, i32> = BTreeMap::new(); 
    for (n, m) in members.iter() {
        if is_scalar(n) {
            continue;
        }
        let relation = if let Some(r) = &m.relation { r } else { continue; };
        if has_name(relation) {
            continue;
        }
        let count = if let Some(c) = relationships.remove(n) { c } else { 0 };
        relationships.insert(n, count + 1);
    }
    relationships
}

fn has_ambiguous_relations<'a>(counts: BTreeMap<&'a str, i32>) -> bool {
    for c in counts.values() {
        if c > &1 {
            return true;
        }
    }
    false
}

fn model_has_ambiguous_relations<'a>(m: &'a BTreeMap<&'a str, ModelMemberOptions<'a>>) -> bool {
    has_ambiguous_relations(relation_counts(m))
}

fn model_has_issue<'a>(
    n: &'a str, 
    model: &ModelOptions<'a>, 
    model_types: &[&'a str], 
    models: &BTreeMap<&'a str, ModelOptions<'a>>
) -> Option<String> {
    // The model name must be unique
    if !is_unique(n, model_types) {
        return Some(format!("Model name {} is not unique", n));
    }
    let member_list = get_member_list(&model.members);
    for (member_n, o) in model.members.iter() {
        // the member name must be unique
        if !is_unique(member_n, &member_list) {
            return Some(format!("Model {} has {} listed more than once", n, member_n));
        }
        if !has_unique_id(&model.members) {
            return Some(format!("model {} does not have a unique id", n));
        }
        if let Some(r) = &o.relation { 
            if let Some(fields) = r.fields {
                if let Some(f) = missing_releation_fields(n, &fields, models) {
                    return Some(format!("Cannot find field {}/{} referenced by {}/{}", n, f, n, member_n));
                }
            } else {
                if r.references.is_some() {
                    return Some(format!("fields section is missing on {}/{}", n, member_n));
                }
            }
            if let Some(references) = r.references {
                if let Some(r) = missing_relation_reference(o.member_type, references, models) {
                    return Some(format!("Cannot find reference {} referenced by {}/{}", r, n, member_n));
                }
            } else {
                if r.fields.is_some() {
                    return Some(format!("references section is missing on {}/{}", n, member_n));
                }
            }
        }
    }
    if model_has_ambiguous_relations(&model.members) {
        return Some(format!("Model {} has ambiguous relations", n));
    }
    return None
}

fn models_have_issues<'a>(models: &BTreeMap<&'a str, ModelOptions<'a>>) -> Option<String> {
    let model_types = get_model_types(models);
    for (n, m) in models.iter() {
        if let Some(issue) = model_has_issue(n, m, &model_types, models) {
            return Some(issue)
        }
    }
    None
}

fn schema_has_issues<'a>(schema: &SchemaOptions) -> Option<String> {
    if let Some(msg) = models_have_issues(&schema.models) {
        return Some(msg);
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_model_type_test() {
        assert!(is_valid_type("User", &[ "User" ]));
        assert!(!is_valid_type("Users", &[ "User" ]))
    }

    #[test]
    fn model_name_is_unique_test()  {
        assert!(model_name_is_unique("User", &["User", "Post"]));
        assert!(!model_name_is_unique("User", &["User", "Post", "User"]));
    }

    #[test]
    fn has_unique_id_test<'a>() {
        let mut model_member_options: BTreeMap<&str, ModelMemberOptions<'a>> = BTreeMap::new(); 
        assert!(!has_unique_id(&model_member_options));
        let mut uuid_member = ModelMemberOptions::default();
        uuid_member.is_id = true;
        model_member_options.insert("uuid", uuid_member);
        assert!(has_unique_id(&model_member_options));
        let mut uuid_2_member = ModelMemberOptions::default();
        uuid_2_member.is_id = true;
        model_member_options.insert("uuid2", uuid_2_member);
        assert!(!has_unique_id(&model_member_options));
    }

    #[test]
    fn schema_has_issues_test() {
        use serde_json::{ Value, json, from_value };
        let json_schema = json!({
            "schema": {
                "database": {
                    "type": "postgres"
                },
                "models": {
                    "User": {
                        "members": {
                            "uuid": {
                                "type": "Uuid",
                                "default": "Uuid",
                                "is_id": true
                            },
                            "first_name": "String",
                            "last_name": "String",
                            "posts": {
                                "type": "Post",
                                "is_list": true
                            },
                            "jobs": {
                                "type": "Job",
                                "is_list": true
                            }
                        }
                    },
                    "Post": {
                        "members": {
                            "uuid": {
                                "type": "Uuid",
                                "default": "Uuid",
                                "is_id": true
                            },
                            "user": {
                                "type": "User",
                                "relation": {
                                    "name": "user_post",
                                    "fields": [ "user_uuid" ],
                                    "references": [ "uuid" ]
                                }
                            },
                            "user_uuid": "Uuid"
                        }
                    },
                    "Job": {
                        "members": {
                            "uuid": {
                                "type": "Uuid",
                                "default": "Uuid",
                                "is_id": true
                            },
                            "foreman": {
                                "type": "User",
                                "relation": {
                                    "name": "job_foreman",
                                    "fields": [ "foreman_uuid" ],
                                    "references": [ "uuid" ]
                                }
                            },
                            "foreman_uuid": "Uuid",
                            "workers": {
                                "type": "User",
                                "is_list": true
                            }
                        }
                    }
                }
            }
        });
        let schema_input: OuterSchemaInput = from_value(json_schema).unwrap();
        let schema = SchemaOptions::from_outer_input(&schema_input);
        assert_eq!(
            None,
            schema_has_issues(&schema)
        );
    }
}
