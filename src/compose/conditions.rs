use postgres::types::ToSql;
use std::fmt::Write;

pub fn all<'a, 'b>(
    conditions: &'b [
        &'b dyn Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String)
    ]
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, params, sql| {
        let (first, rest) = conditions.split_first().unwrap();
        let (mut index, mut params, mut sql) = first(index, params, sql);
        for thunk in rest.iter() {
            write!(&mut sql, " AND ").unwrap();
            let (i, p, s) = thunk(index, params, sql);
            index = i;
            params = p;
            sql = s;
        }
        (index, params, sql)
    }
}

pub fn any<'a, 'b>(
    conditions: &'b [
        &'b dyn Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String)
    ]
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) + 'b {
    move |index, params, sql| {
        let (first, rest) = conditions.split_first().unwrap();
        let (mut index, mut params, mut sql) = first(index, params, sql);
        for thunk in rest.iter() {
            write!(&mut sql, " OR ").unwrap();
            let (i, p, s) = thunk(index, params, sql);
            index = i;
            params = p;
            sql = s;
        }
        (index, params, sql)
    }
}

pub fn condition<'a>(
    cond: impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String), 
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) {
    move |index, params, mut sql| {
        write!(&mut sql, "WHERE ").unwrap();
        let (index, params, sql) = cond(index, params, sql);
        (index, params, sql)
    }
}

pub fn enclose<'a>(
    param: impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String)
) -> impl Fn(i32, Vec<&'a (dyn ToSql + Sync)>, String) -> (i32, Vec<&'a (dyn ToSql + Sync)>, String) {
    move |index, params, mut sql| {
        write!(&mut sql, "(").unwrap();
        let (index, params, mut sql) = param(index, params, sql);
        write!(&mut sql, ")").unwrap();
        (index, params, sql)
    }
}

pub fn in_sub<'a>(col: &'a str, func: impl Fn(i32, Vec<&'a dyn ToSql>) -> (i32, Vec<&'a dyn ToSql>, String)) -> 
impl Fn(i32, Vec<&'a dyn ToSql>) -> (i32, Vec<&'a dyn ToSql>, String) {
    move |index, params| {
        let (index, params, sql) = func(index, params);
        (index, params, format!("{} IN ({})", col, sql))
    }
}

pub fn in_values<'a, 'b>(col: &'b str, values: &'b [&'a impl ToSql]) -> 
impl Fn(i32, Vec<&'a dyn ToSql>) -> (i32, Vec<&'a dyn ToSql>, String) + 'b {
    move |mut index, mut params| {
        let c = values.len() as i32;
        let mut args: Vec<String> = Vec::with_capacity(c as usize);
        for v in values {
            params.push(*v);
            args.push(format!("${}", index));
            index += 1;
        }
        (index, params, format!("{} IN ({})", col, args.join(", ")))
    }
}

pub fn or<'a, 'b>(
    condition_1: impl Fn((i32, Vec<&'a dyn ToSql>)) -> (i32, Vec<&'a dyn ToSql>, String), 
    condition_2: impl Fn((i32, Vec<&'a dyn ToSql>)) -> (i32, Vec<&'a dyn ToSql>, String)) -> 
impl Fn(i32, Vec<&'a dyn ToSql>) -> (i32, Vec<&'a dyn ToSql>, String) {
    move |index, params| {
        let (index, params, cond1_sql) = condition_1((index, params));
        let (index, params, cond2_sql) = condition_2((index, params));
        (index, params, format!("{} OR {}", cond1_sql, cond2_sql))
    }
}

#[cfg(test)]
mod test {
    use super::{
        *,
        super::{
            operators::*
        }
    };

    #[test]
    fn all_test() {
        let (_index, _params, sql) = enclose(all(&[ 
            &eqls("id", &1),
            &eqls("name", &"foo"),
        ]))(1, Vec::with_capacity(1), String::with_capacity(50));
        assert_eq!(
            "(id=$1 AND name=$2)",
            sql);
    }

    #[test]
    fn any_test() {
        let (_index, _params, sql) = enclose(any(&[ 
            &eqls("id", &1),
            &eqls("name", &"foo"),
        ]))(1, Vec::with_capacity(1), String::with_capacity(50));
        assert_eq!(
            "(id=$1 OR name=$2)",
            sql);
    }

    #[test]
    fn condition_test() {
        let (index, params, sql) = condition(any(&[
            &enclose(all(&[
                 &gt("foo", &10),
                 &lte("foo", &100),
            ])),
            &enclose(any(&[
                 &eqls("bar", &"baz"),
                 &like("bar", &"fizz%buzz")
            ]))
        ]))(1, Vec::with_capacity(4), String::with_capacity(1000));
        assert_eq!(
            "WHERE (foo>$1 AND foo<=$2) OR (bar=$3 OR bar LIKE $4)",
            sql
        );
        assert_eq!(5, index);
        assert_eq!(4, params.len());
    }

}

