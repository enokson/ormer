use std::{
    borrow::{Borrow},
    fmt::Display,
};

pub fn compose<T: Display + Borrow<str>>(conditions: Vec<T>) -> String {
    let values =  conditions
        .into_iter()
        .filter_map(|cond| {
            let value = cond.borrow();
            if value.is_empty() {
                return None;
            }
            Some(cond)
        })
        .collect::<Vec<T>>();
    if values.is_empty() {
        return "".to_string()
    }
    values.join(" ")
}

pub fn list<T: Display + Borrow<str>>(conditions: Vec<T>) -> String {
    let values =  conditions
        .into_iter()
        .filter_map(|cond| {
            let value = cond.borrow();
            if value.is_empty() {
                return None;
            }
            Some(cond)
        })
        .collect::<Vec<T>>();
    if values.is_empty() {
        return "".to_string()
    }
    values.join(",")
}

pub fn filter<T: Display + Borrow<str>>(conditions: T) -> String {
    let value = conditions.borrow();
    if value.is_empty() {
        return "".to_string();
    }
    format!("WHERE {}", conditions)
}

pub fn any<T: Display + Borrow<str>>(conditions: Vec<T>) -> String {
    let values = conditions
        .into_iter()
        .filter_map(|cond| {
            let value = cond.borrow();
            if value.is_empty() {
                return None;
            }
            Some(cond)
        })
        .collect::<Vec<T>>();
    if values.is_empty() {
        return "".to_string()
    }
    if values.len() == 1 {
        return values[0].to_string()
    }
    values.join(" OR ")
}

pub fn all<T: Display + Borrow<str>>(conditions: Vec<T>) -> String {
    let values = conditions
        .into_iter()
        .filter_map(|cond| {
            let value = cond.borrow();
            if value.is_empty() {
                return None;
            }
            Some(cond)
        })
        .collect::<Vec<T>>();
    if values.is_empty() {
        return "".to_string()
    }
    if values.len() == 1 {
        return values[0].to_string()
    }
    values.join(" AND ")
}

pub fn compare<T1: Display, T2: Display>(left_w_operator: T1, right: T2) -> String {
    format!("{} '{}'", left_w_operator, right)
}

pub fn append<T1: Display, T2: Display>(src: T1, value: T2) -> String {
    format!("{} {}", src, value)
}

pub fn column<T1: Display, T2: Display>(column: T1, operation: T2) -> String {
    format!("{} {}", column, operation)
}

pub fn equals<T: Display>(value: T) -> String {
    format!("= '{}'", value)
}

pub fn to<T: Display>(to: T) -> T {
    to
}

pub fn and<T1: Display, T2: Display>(left: T1, right: T2) -> String {
    format!("{} AND {}", left, right)
}



mod lambda {
    use std::{
        borrow::{
            Borrow
        },
        fmt::{
            Display
        }
    };

    // here the sql query goes last by default
    #[macro_export]
    macro_rules! pipe {
        ($($arg:expr),*) => {{
            let mut args: Vec<String> = vec![];
            $(
                args.push($arg.to_string());
            )*
            args.join(" ")
        }};
    }

    #[macro_export]
    macro_rules! pipe_to_end {
        ($($f:expr),*) => {{
            let mut fncs: Vec<Box<dyn Fn(String) -> String>> = vec![];
            $(
                fncs.push($f);
            )*
            let mut sql = "".to_string();
            for f in fncs {
                sql = f(sql);
            }
            sql
        }};
    }

    #[macro_export]
    macro_rules! filter {
        ($($f:expr),*) => {{
            let mut fncs: Vec<Box<dyn Fn(String) -> String>> = vec![];
            $(
                fncs.push($f);
            )*
            let mut sql = "".to_string();
            for f in fncs {
                sql = f(sql);
            }
            sql
        }};
    }

    pub enum Compose {
        Fn(Box<dyn Fn(dyn Display) -> Compose>),
        FnFinish(Box<dyn Fn() -> Compose>),
        Value(Box<dyn Display>),
        Finish
    }

    fn test() {
        // let sql = pipe!(
        //     limit(1),
        //     group_by("uuid")
        // );
        
    }

    mod runtime {

        use std::{
            borrow::{
                Borrow
            },
            fmt::{
                Display
            }
        };

        pub trait Joinable: Borrow<str> + Display {

        }

        pub trait Composable<T>: Display {
            fn to_composable(self) -> Box<Compose<T>>;
        }

        impl Composable<String> for String {
            fn to_composable(self) -> Box<Compose<String>> {
                Box::new(Compose::Value(self))
            }
        }

        pub enum Compose<T> {
            Fn(Box<dyn Fn(T) -> Compose<T>>),
            Value(T),
            Finish
        }

        pub fn limit<T>(limit: Box<dyn Composable<T>>) -> impl Composable<T> {
            
        }

        // pub fn group_by<T1: Display>(column: Compose) -> Compose {
        //     Compose::Fn(Box::new(move |sql| Compose::Value(Box::new(format!("{} {}", sql, column)))))
        // }
    }

    pub fn group_by<T1: Display>(column: T1) -> String {
        format!("group by {}", column)
    }

    pub fn limit<T1: Display>(limit: T1) -> String {
        format!("limit {}", limit)
    }

    pub fn skip<T1: Display>(limit: T1) -> String {
        format!("skip {}", limit)
    }

    pub fn asc<T1: Display>() -> String {
        format!("asc")
    }

    pub fn desc<T1: Display>() -> String {
        format!("desc")
    }

    pub fn select<T1: Display>(selection: T1) -> String {
        format!("select {}", selection)
    }

    pub fn from<T1: Display>(from: T1) -> String {
        format!("from {}", from)
    }

    pub fn filter<T1: Display>(filter: T1) -> String {
        format!("where {}", filter)
    }

    pub fn or_next<T1: Display>(next: T1) -> String {
        format!("or {}", next)
    }

    pub fn and_next<T1: Display>(next: T1) -> String {
        format!("and {}", next)
    }
    
    pub fn placehold<T1: Borrow<str> + Display, T2: Display>(f: Box<dyn Fn(T1) -> Box<dyn Fn(T2) -> String>>, right: T2) -> Box<impl FnOnce(T1) -> String> {
        Box::new(move |left| f(left)(right))
    }
}

#[cfg(test)]
mod compose_tests {

    use super::lambda::*;
    use crate::{pipe, compose::lambda::group_by, pipe_to_end};

    #[test]
    fn filter_test() {

        assert_eq!(
            "select * from users where name = 'joshua' limit 10 group by name skip 5",
            pipe!(
                "select * from users",
                filter(pipe!(
                    "name = 'joshua'"
                )),
                limit(10),
                group_by("name"),
                skip(5)
            )
        );

        assert_eq!(
            "select * from users where name = 'joshua' or id = '1' limit 10 group by name skip 5",
            pipe!(
                select("*"),
                from("users"),
                filter(pipe!(
                    "name = 'joshua'",
                    or_next("id = '1'")
                )),
                limit(10),
                group_by("name"),
                skip(5)
            )
        );

        // assert_eq!(
        //     "",
        //     filter("")
        // );

    }


}