use std::{
    fmt::Display
};

pub fn operator<T: Display>(operator: &str, value: &T) -> String {
    format!("{} {}", operator, value)
}

pub fn equals<T: Display>(value: &T) -> String {
    operator("=", value)
}

pub fn not<T: Display>(value: &T) -> String {
    format!("NOT {}", value)
}

pub fn gt<T: Display>(value: &T) -> String {
    operator(">", value)
}

pub fn gte<T: Display>(value: &T) -> String {
    operator(">=", value)
}

pub fn lt<T: Display>(value: &T) -> String {
    operator("<", value)
}

pub fn lte<T: Display>(value: &T) -> String {
    operator("<=", value)
}
// pub fn contains<T: Display>(value: &T) -> String {
//     operator("=", value)
// }

pub fn search<T: Display>(value: &T) -> String {
    operator("LIKE", &format!("'%{}%'", value))
}

pub fn start_with<T: Display>(value: &T) -> String {
    operator("LIKE", &format!("'{}%'", value))
}

pub fn ends_with<T: Display>(value: &T) -> String {
    operator("LIKE", &format!("'%{}'", value))
}

pub fn escape<T: Display>(value: &T) -> String {
    format!("'{}'", value)
}

pub fn is_in<T: Display>(value: &T) -> String {
    format!("IN ({})", value)
}

pub fn enclose<T: Display>(value: &str) -> String {
    format!("({})", value)
}

pub fn prepend_column<T: Display>(column: &str, value: &T) -> String {
    format!("{} {}", column, value)
}