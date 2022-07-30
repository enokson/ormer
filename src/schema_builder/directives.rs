use regex::Regex;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use super::{
    error::{OrmerError, StackError},
    DefaultValue,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directives {
    pub member_type: Rc<String>,
    pub is_array: bool,
    pub is_optional: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: Option<DefaultValue>,
    pub relation: Option<Rc<RefCell<RelationDirectiveInfo>>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModelModifiersInfo {
    pub model: String,
    pub is_array: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationDirectiveInfo {
    pub name: Option<Rc<RefCell<String>>>,
    pub fields: Rc<Vec<Rc<String>>>,
    pub references: Rc<Vec<Rc<String>>>,
}

fn get_default_value(directives: &str) -> Option<DefaultValue> {
    let filter = Regex::new(r"@default\s*\(.*?\)").unwrap();
    if let Some(default_value) = filter.find(directives) {
        let default_value_str = default_value.as_str().to_string();
        let default_value = {
            if default_value_str.contains("AutoInc") {
                DefaultValue::AutoInc
            } else if default_value_str.contains("Uuid") {
                DefaultValue::Uuid
            } else if default_value_str.contains("Now") {
                DefaultValue::Now
            } else {
                panic!("Invalid @default type");
            }
        };
        return Some(default_value);
    }
    None
}

fn get_relation_directive(directives: &str) -> Option<Rc<RefCell<RelationDirectiveInfo>>> {
    // let possible_captures = get_value_between_two_char(r"Ref\((.*?)\)", directives);
    let re = Regex::new(r"@relation\((.*?)\)").unwrap();
    let captures = match re.captures(directives) {
        Some(captures) => captures,
        None => return None,
    };
    let directive_str = match captures.get(1) {
        Some(matched) => matched.as_str().to_string(),
        None => return None,
    };
    let fields = match Regex::new(r"fields\s*:\s*\[(.*?)\]\s*,")
        .unwrap()
        .captures(&directive_str)
    {
        Some(captures) => match captures.get(1) {
            Some(value_match) => value_match
                .as_str()
                .split(",")
                .map(|v| Rc::new(v.to_string()))
                .collect::<Vec<Rc<String>>>(),
            None => panic!("failed to parse fields."),
        },
        None => panic!("failed to parse fields."),
    };
    let references = match Regex::new(r"references\s*:\s*\[(.*?)\]\s*")
        .unwrap()
        .captures(&directive_str)
    {
        Some(captures) => match captures.get(1) {
            Some(value_match) => value_match
                .as_str()
                .split(",")
                .map(|v| Rc::new(v.to_string()))
                .collect::<Vec<Rc<String>>>(),
            None => panic!("failed to parse fields."),
        },
        None => panic!("failed to parse fields."),
    };

    // this regex can get the name (if it exists) from the directive
    // @relation\s*\(\s*(?:name\s*:)?\s*(.*?)\s*,\s(?:fields\s*:\s*)|references\s*:\s*\)

    // this regex can get both the name (if it exists) and fields from the directive
    // @relation\s*\(\s*(?:name\s*:)?\s*(.*?)\s*,\s(?:fields\s*:\s*\[(.*?)\])|references\s*:\s*\)
    // old name\s*:\s*(.*?)\s*,
    let name = match Regex::new(r"name\s*:\s*(.*?)\s*,")
        .unwrap()
        .captures(&directive_str)
    {
        Some(captures) => match captures.get(1) {
            Some(value_match) => Some(Rc::new(RefCell::new(value_match.as_str().to_string()))),
            None => None,
        },
        None => None,
    };

    Some(Rc::new(RefCell::new(RelationDirectiveInfo {
        name,
        fields: Rc::new(fields),
        references: Rc::new(references),
    })))
}

fn get_model_name(directives: &str) -> Rc<String> {
    match Regex::new(r"^\s*\n*\r*([a-zA-Z]*?)(?:\[|\?|\s|\z)")
        .unwrap()
        .captures(directives)
    {
        Some(captures) => match captures.get(1) {
            Some(value_match) => Rc::new(value_match.as_str().to_string()),
            None => panic!("failed to parse fields."),
        },
        None => panic!("failed to parse fields."),
    }
}

fn is_model_list(directives: &str) -> bool {
    Regex::new(r"(?:.*?)(\[[^\S]*\])(?:\s|\?|\z)")
        .unwrap()
        .is_match(directives)
}

fn is_optional(directives: &str) -> bool {
    Regex::new(r"(?:.*?)(\?)(?:\s|\z)")
        .unwrap()
        .is_match(directives)
}

pub fn parse_directives_str(directives: &str) -> Directives {
    Directives {
        member_type: get_model_name(directives),
        is_array: is_model_list(directives),
        is_optional: is_optional(directives),
        is_primary_key: directives.contains("@primary"),
        is_unique: directives.contains("@unique"),
        default_value: get_default_value(directives),
        relation: get_relation_directive(directives),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultType2 {
    Uuid,
    AutoInc,
    Now,
}
impl TryFrom<String> for DefaultType2 {
    type Error = StackError;
    fn try_from(str: String) -> Result<Self, StackError> {
        let auto_inc_rgx = match Regex::new(r"@default\(\s*(@\s*autoInc)\s*\)") {
            Ok(re) => re,
            Err(err) => {
                return Err(StackError::from_regex_error(
                    err,
                    file!(),
                    line!(),
                    Some(format!("Failed to parse {} as a @default_type.", str)),
                ))
            }
        };
        if auto_inc_rgx.is_match(&str) {
            return Ok(Self::AutoInc);
        }
        let uuid_rgx = match Regex::new(r"@default\(\s*(@\s*uuid)\s*\)") {
            Ok(re) => re,
            Err(err) => {
                return Err(StackError::from_regex_error(
                    err,
                    file!(),
                    line!(),
                    Some(format!("Failed to parse {} as a @default_type.", str)),
                ))
            }
        };
        if uuid_rgx.is_match(&str) {
            return Ok(Self::Uuid);
        }
        let now_rgx = match Regex::new(r"@default\(\s*(@\s*now)\s*\)") {
            Ok(re) => re,
            Err(err) => {
                return Err(StackError::from_regex_error(
                    err,
                    file!(),
                    line!(),
                    Some(format!("Failed to parse {} as a @default_type.", str)),
                ))
            }
        };
        if now_rgx.is_match(&str) {
            return Ok(Self::Now);
        }
        return Err(StackError::user_config_error(
            file!(),
            line!(),
            Some(format!("{} is not a default type", &str)),
        ));
    }
}

type Member2 = BTreeMap<Rc<String>, Rc<RefCell<Directives>>>;
type Model2 = BTreeMap<Rc<String>, Rc<RefCell<Member2>>>;

#[derive(Debug, PartialEq, Eq)]
pub struct RelationDirective2 {
    pub name: Option<Rc<RefCell<String>>>,
    pub fields: Vec<String>,
    pub references: Vec<String>,
    pub referenced_member: Option<(Rc<RefCell<Model2>>, Rc<RefCell<Member2>>)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Directives2 {
    pub model_type: String,
    pub is_list: bool,
    pub is_optional: bool,
    pub is_id: bool,
    pub default: Option<DefaultType2>,
    pub relation: Option<Rc<RefCell<RelationDirective2>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PartialDirectives {
    pub remainder: String,
    pub model_type: Option<String>,
    pub is_list: bool,
    pub is_optional: bool,
    pub is_id: bool,
    pub default: Option<DefaultType2>,
    pub relation: Option<Rc<RefCell<RelationDirective2>>>,
}

impl Default for PartialDirectives {
    fn default() -> Self {
        PartialDirectives {
            remainder: String::new(),
            model_type: None,
            is_list: false,
            is_optional: false,
            is_id: false,
            default: None,
            relation: None,
        }
    }
}

fn parse_relation_directive(
    directives: PartialDirectives,
) -> Result<PartialDirectives, StackError> {
    let re = match Regex::new(r"@relation\s*\(.*?\)") {
        Ok(re) => re,
        Err(err) => {
            return Err(StackError::from_regex_error(
                err,
                file!(),
                line!(),
                Some(""),
            ))
        }
    };
    let captures = match re.captures(&directives.remainder) {
        Some(captures) => captures,
        None => return Ok(directives),
    };
    if captures.len() > 1 {
        return Err(StackError::new_wo_error(
            OrmerError::UserConfigError,
            file!(),
            line!(),
            Some("Members cannot have more than one relation directive."),
        ));
    }
    let relation_string: String = match captures.get(0) {
        Some(matched) => matched.as_str().to_string(),
        None => {
            return Err(StackError::new_wo_error(
                OrmerError::ParsingError,
                file!(),
                line!(),
                Some("a match was found for the relation directive, but it could be retrieved."),
            ))
        }
    };
    let fields_member_rgx = match Regex::new(r"fields\s*:\s*\[.*?\]") {
        Ok(rgx) => rgx,
        Err(err) => {
            return Err(StackError::from_regex_error(
                err,
                file!(),
                line!(),
                Some(""),
            ))
        }
    };
    let field_captures = match fields_member_rgx.captures(&relation_string) {
        Some(captures) => captures,
        None => return Ok(directives),
    };
    if field_captures.len() > 1 {
        return Err(StackError::new_wo_error(
            OrmerError::UserConfigError,
            file!(),
            line!(),
            Some("@relation directives cannot not have more than one fields member"),
        ));
    }
    let fields_string: String = match field_captures.get(0) {
        Some(matched) => matched.as_str().to_string(),
        None => {
            return Err(StackError::new_wo_error(
                OrmerError::ParsingError,
                file!(),
                line!(),
                Some("a match was found for the fields member, but it could be retrieved."),
            ))
        }
    };
    let fields: Vec<String> = fields_string
        .replace("fields", "")
        .replace(":", "")
        .replace("[", "")
        .replace("]", "")
        .replace(" ", "")
        .split(",")
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
    let removed_fields_member_string = fields_member_rgx
        .replace_all(&relation_string, "")
        .to_string();

    let references_member_rgx = match Regex::new(r"references\s*:\s*\[.*?\]") {
        Ok(rgx) => rgx,
        Err(err) => {
            return Err(StackError::from_regex_error(
                err,
                file!(),
                line!(),
                Some(""),
            ))
        }
    };
    let references_captures = match references_member_rgx.captures(&removed_fields_member_string) {
        Some(captures) => captures,
        None => return Ok(directives),
    };
    if references_captures.len() > 1 {
        return Err(StackError::new_wo_error(
            OrmerError::UserConfigError,
            file!(),
            line!(),
            Some("@relation directives cannot not have more than one references member"),
        ));
    }
    let references_string: String = match references_captures.get(0) {
        Some(matched) => matched.as_str().to_string(),
        None => {
            return Err(StackError::new_wo_error(
                OrmerError::ParsingError,
                file!(),
                line!(),
                Some("a match was found for the reference member, but it could be retrieved."),
            ))
        }
    };
    let references: Vec<String> = references_string
        .replace("references", "")
        .replace(":", "")
        .replace("[", "")
        .replace("]", "")
        .replace(" ", "")
        .split(",")
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
    let name_options: Vec<String> = references_member_rgx
        .replace_all(&removed_fields_member_string, "")
        .to_string()
        .replace("@relation", "")
        .replace("(", "")
        .replace(")", "")
        .replace("name", "")
        .replace(":", "")
        .replace("'", "")
        .replace("\"", "")
        .split(",")
        .map(|str| str.trim_start().trim_end())
        .filter(|str| !str.is_empty())
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
    if name_options.len() > 1 {
        return Err(StackError::new_wo_error(
            OrmerError::UserConfigError,
            file!(),
            line!(),
            Some("Could not parse @relation. There are too many options."),
        ));
    }
    let name = if !name_options.is_empty() {
        Some(Rc::new(RefCell::new(name_options[0].to_string())))
    } else {
        None
    };
    let directives_with_removed_relation = re.replace_all(&directives.remainder, "").to_string();
    Ok(PartialDirectives {
        remainder: directives_with_removed_relation,
        relation: Some(Rc::new(RefCell::new(RelationDirective2 {
            name,
            fields,
            references,
            referenced_member: None,
        }))),
        ..directives
    })
}

fn parse_default_directive(directives: PartialDirectives) -> Result<PartialDirectives, StackError> {
    let default_directive_rgx = match Regex::new(r"@default\s*\(\s*@[a-zA-Z]*\s*\)") {
        Ok(re) => re,
        Err(err) => {
            return Err(StackError::from_regex_error(
                err,
                file!(),
                line!(),
                Some("Could not parse regex"),
            ))
        }
    };
    let captures = match default_directive_rgx.captures(&directives.remainder) {
        Some(captures) => captures,
        None => return Ok(directives),
    };
    if captures.len() > 1 {
        return Err(StackError::user_config_error(
            file!(),
            line!(),
            Some("Members cannot have more than one @default directive."),
        ));
    }
    let default_string: String = match captures.get(0) {
        Some(matched) => matched.as_str().to_string(),
        None => {
            return Err(StackError::new_wo_error(
                OrmerError::ParsingError,
                file!(),
                line!(),
                Some("a match was found for the @default directive, but it could be retrieved."),
            ))
        }
    };
    let modified_remainder = default_directive_rgx
        .replace_all(&directives.remainder, "")
        .to_string();
    Ok(PartialDirectives {
        remainder: modified_remainder,
        default: Some(DefaultType2::try_from(default_string)?),
        ..directives
    })
}

fn parse_id_directive(mut directives: PartialDirectives) -> Result<PartialDirectives, StackError> {
    let id_directive_rgx = match Regex::new(r"@id") {
        Ok(re) => re,
        Err(err) => {
            return Err(StackError::from_regex_error(
                err,
                file!(),
                line!(),
                Some("Could not parse regex"),
            ))
        }
    };
    let captures = match id_directive_rgx.captures(&directives.remainder) {
        Some(captures) => captures,
        None => {
            directives.is_id = false;
            return Ok(directives);
        }
    };
    if captures.len() > 1 {
        return Err(StackError::user_config_error(
            file!(),
            line!(),
            Some("Members cannot have more than one @id directive."),
        ));
    }
    let is_id = captures.len() == 1;
    let modified_remainder = id_directive_rgx
        .replace_all(&directives.remainder, "")
        .to_string();
    Ok(PartialDirectives {
        remainder: modified_remainder,
        is_id,
        ..directives
    })
}

fn parse_optional_directive(
    mut directives: PartialDirectives,
) -> Result<PartialDirectives, StackError> {
    let is_optional_directive_rgx = match Regex::new(r"\?") {
        Ok(re) => re,
        Err(err) => {
            return Err(StackError::from_regex_error(
                err,
                file!(),
                line!(),
                Some("Could not parse regex"),
            ))
        }
    };
    let captures = match is_optional_directive_rgx.captures(&directives.remainder) {
        Some(captures) => captures,
        None => {
            directives.is_optional = false;
            return Ok(directives);
        }
    };
    if captures.len() > 1 {
        return Err(StackError::user_config_error(
            file!(),
            line!(),
            Some("Members cannot have more than one @id directive."),
        ));
    }
    let is_optional = captures.len() == 1;
    let modified_remainder = is_optional_directive_rgx
        .replace_all(&directives.remainder, "")
        .to_string();
    Ok(PartialDirectives {
        remainder: modified_remainder,
        is_optional,
        ..directives
    })
}

fn parse_model_directive(directives: PartialDirectives) -> Result<PartialDirectives, StackError> {
    let model_directive_rgx = match Regex::new(r"^\s*\r*\n*[A-Z]+[a-zA-Z]+") {
        Ok(re) => re,
        Err(err) => {
            return Err(StackError::from_regex_error(
                err,
                file!(),
                line!(),
                Some("Could not parse regex"),
            ))
        }
    };
    let captures = match model_directive_rgx.captures(&directives.remainder) {
        Some(captures) => captures,
        None => {
            let model_directive_rgx = match Regex::new(r"^\s*\r*\n*[a-zA-Z]+") {
                Ok(re) => re,
                Err(err) => {
                    return Err(StackError::from_regex_error(
                        err,
                        file!(),
                        line!(),
                        Some("Could not parse regex"),
                    ))
                }
            };
            let captures = match model_directive_rgx.captures(&directives.remainder) {
                Some(captures) => captures,
                None => {
                    return Err(StackError::user_config_error(
                        file!(),
                        line!(),
                        Some("the model could not be determined."),
                    ))
                }
            };
            if captures.len() == 0 {
                return Err(StackError::user_config_error(
                    file!(),
                    line!(),
                    Some("the model could not be determined."),
                ));
            }
            let model_string: String = match captures.get(0) {
                Some(matched) => matched.as_str().to_string(),
                None => {
                    return Err(StackError::user_config_error(
                        file!(),
                        line!(),
                        Some("Expected model name to be PascalCase, found unknown string"),
                    ))
                }
            };
            return Err(StackError::user_config_error(
                file!(),
                line!(),
                Some(format!(
                    "Expected model name to be PascalCase, found {}",
                    model_string
                )),
            ));
        }
    };
    if captures.len() > 1 {
        return Err(StackError::user_config_error(
            file!(),
            line!(),
            Some(format!("Expected a model, found {}", directives.remainder)),
        ));
    }
    let model_string: String = match captures.get(0) {
        Some(matched) => matched.as_str().to_string(),
        None => {
            return Err(StackError::new_wo_error(
                OrmerError::ParsingError,
                file!(),
                line!(),
                Some("a match was found for the model directive, but it could be retrieved."),
            ))
        }
    };
    let modified_remainder = model_directive_rgx
        .replace_all(&directives.remainder, "")
        .to_string();
    Ok(PartialDirectives {
        remainder: modified_remainder,
        model_type: Some(model_string),
        ..directives
    })
}

fn parse_list_directive(
    mut directives: PartialDirectives,
) -> Result<PartialDirectives, StackError> {
    // ^\s*\r*\n*(?P<y>(?:[A-Z]+[a-zA-Z]+)|^)\s*\[\s*?\]
    // (?P<m>^\s*\r*\n*[A-Z]+[a-zA-Z]+)\s*\[\s*?\]
    let is_list_directive_rgx =
        match Regex::new(r"^\s*\r*\n*(?P<m>(?:[A-Z]+[a-zA-Z]+)|^)\s*\[\s*?\]") {
            Ok(re) => re,
            Err(err) => {
                return Err(StackError::from_regex_error(
                    err,
                    file!(),
                    line!(),
                    Some("Could not parse regex"),
                ))
            }
        };
    let captures = match is_list_directive_rgx.captures(&directives.remainder) {
        Some(captures) => captures,
        None => {
            return {
                directives.is_list = false;
                Ok(directives)
            }
        }
    };
    if captures.len() > 2 {
        return Err(StackError::user_config_error(
            file!(),
            line!(),
            Some("If the type is a list or not could not be determined"),
        ));
    }
    let modified_remainder = is_list_directive_rgx
        .replace_all(&directives.remainder, "$m")
        .to_string();
    Ok(PartialDirectives {
        remainder: modified_remainder,
        is_list: true,
        ..directives
    })
}

fn sweep_remaining(mut directives: PartialDirectives) -> Result<PartialDirectives, StackError> {
    let white_space_rgx = match Regex::new(r"\s*\r*\n*") {
        Ok(re) => re,
        Err(err) => {
            return Err(StackError::from_regex_error(
                err,
                file!(),
                line!(),
                Some("Could not parse regex"),
            ))
        }
    };
    let no_remaining = white_space_rgx
        .replace_all(&directives.remainder, "")
        .to_string();
    if !no_remaining.is_empty() {
        return Err(StackError::user_config_error(
            file!(),
            line!(),
            Some(format!("Found remaining tokens: '{}'", no_remaining)),
        ));
    }
    directives.remainder = no_remaining;
    Ok(directives)
}

fn into_directives(directives: PartialDirectives) -> Result<Directives2, StackError> {
    Ok(Directives2 {
        model_type: match directives.model_type {
            Some(model_type) => model_type,
            None => {
                return Err(StackError::new_wo_error(
                    OrmerError::ParsingError,
                    file!(),
                    line!(),
                    Some("Expected a model type, but it wasn't found."),
                ))
            }
        },
        is_id: directives.is_id,
        is_optional: directives.is_optional,
        is_list: directives.is_list,
        default: directives.default,
        relation: directives.relation,
    })
}

pub fn parse_directives(directives_string: String) -> Result<Directives2, StackError> {
    into_directives(pipe!(
        PartialDirectives {
            remainder: directives_string,
            ..Default::default()
        },
        parse_relation_directive,
        parse_default_directive,
        parse_id_directive,
        parse_optional_directive,
        parse_list_directive,
        parse_model_directive,
        sweep_remaining
    )?)
}

// #[macro_export]
macro_rules! pipe {
    ($acc:expr, $( $fn:expr ),+ $(,)?) => {{
        let mut value = $acc;
        $(value = $fn(value)?; )*
        Ok(value)
    }};
}
pub(crate) use pipe;

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn parse_directives_test() {
        assert_eq!(
            Directives2 {
                model_type: "User".to_string(),
                is_id: true,
                is_list: true,
                is_optional: true,
                default: Some(DefaultType2::Uuid),
                relation: Some(Rc::new(RefCell::new(RelationDirective2 {
                    name: Some(Rc::new(RefCell::new("myName".to_string()))),
                    fields: vec!["userUuid".to_string()],
                    references: vec!["uuid".to_string()],
                    referenced_member: None,
                })))
            },
            parse_directives("User[]? @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string()).unwrap()
        );
    }

    #[test]
    fn parse_list_directive_test() {
        assert_eq!(
            PartialDirectives {
                remainder: "User? @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                is_list: true,
                ..Default::default()
            },
            parse_list_directive(PartialDirectives {
                remainder: "User[]? @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                ..Default::default()
            }).unwrap()
        );
        assert_eq!(
            PartialDirectives {
                remainder:
                    "? @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])"
                        .to_string(),
                is_list: true,
                ..Default::default()
            },
            parse_list_directive(PartialDirectives {
                remainder:
                    "[]? @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])"
                        .to_string(),
                ..Default::default()
            })
            .unwrap()
        );
    }

    #[test]
    fn parse_model_test() {
        assert_eq!(
            PartialDirectives {
                remainder: "[]? @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                model_type: Some("User".to_string()),
                ..Default::default()
            },
            parse_model_directive(PartialDirectives {
                remainder: "User[]? @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                ..Default::default()
            }).unwrap()
        );
    }

    #[test]
    fn parse_optional_directive_test() {
        assert_eq!(
            PartialDirectives {
                remainder: "User @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                is_optional: true,
                ..Default::default()
            },
            parse_optional_directive(PartialDirectives {
                remainder: "User? @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                ..Default::default()
            }).unwrap()
        );
        assert_eq!(
            PartialDirectives {
                remainder:
                    "User @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])"
                        .to_string(),
                is_optional: false,
                ..Default::default()
            },
            parse_optional_directive(PartialDirectives {
                remainder:
                    "User @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])"
                        .to_string(),
                ..Default::default()
            })
            .unwrap()
        );
    }

    #[test]
    fn parse_id_directive_test() {
        assert_eq!(
            PartialDirectives {
                remainder: "User?  @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                is_id: true,
                ..Default::default()
            },
            parse_id_directive(PartialDirectives {
                remainder: "User? @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                ..Default::default()
            }).unwrap()
        );
        assert_eq!(
            PartialDirectives {
                remainder:
                    "User? @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])"
                        .to_string(),
                is_id: false,
                ..Default::default()
            },
            parse_id_directive(PartialDirectives {
                remainder:
                    "User? @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])"
                        .to_string(),
                ..Default::default()
            })
            .unwrap()
        );
    }

    #[test]
    fn parse_default_type_directive_test() {
        assert_eq!(
            PartialDirectives {
                remainder: "User? @id  @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                default: Some(DefaultType2::Uuid),
                ..Default::default()
            },
            parse_default_directive(PartialDirectives {
                remainder: "User? @id @default(@uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                ..Default::default()
            }).unwrap()
        )
    }

    #[test]
    fn parse_relation_directive_test() {
        assert_eq!(
            PartialDirectives {
                remainder: "User? @id @default(Uuid) ".to_string(),
                relation: Some(Rc::new(RefCell::new(
                    RelationDirective2 {
                        name: Some(Rc::new(RefCell::new("myName".to_string()))),
                        fields: vec![ "userUuid".to_string() ],
                        references: vec![ "uuid".to_string() ],
                        referenced_member: None
                    }
                ))),
                ..Default::default()
            },
            parse_relation_directive(PartialDirectives {
                remainder: "User? @id @default(Uuid) @relation(myName, fields:[userUuid], references:[uuid])".to_string(),
                ..Default::default()
            }).unwrap()
        )
    }
}
