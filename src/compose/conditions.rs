use postgres::types::ToSql;
use std::fmt::Write;

pub fn all<'a>(
    conditions: &'a [&'a (dyn Fn(String) -> String)] 
) -> impl Fn(String) -> String + 'a {
    move |sql| {
        let (first, rest) = conditions.split_first().unwrap();
        let mut sql = first(sql);
        for thunk in rest.iter() {
            write!(&mut sql, " AND ").unwrap();
            let s = thunk(sql);
            sql = s;
        }
        sql
    }
}

pub fn any<'a>(
    conditions: &'a [ &'a dyn Fn(String) -> String ]
) -> impl Fn(String) -> String + 'a {
    move |sql| {
        let (first, rest) = conditions.split_first().unwrap();
        let mut sql = first(sql);
        for thunk in rest.iter() {
            write!(&mut sql, " OR ").unwrap();
            let s = thunk(sql);
            sql = s;
        }
        sql
    }
}

pub fn condition<'a>(cond: &'a impl Fn(String) -> String) -> impl Fn(String) -> String + 'a {
    move |mut sql| {
        write!(&mut sql, "WHERE ").unwrap();
        let sql = cond(sql);
        sql
    }
}


pub fn in_sub<'a>(
    col: impl Fn(String) -> String + 'a, 
    func: impl Fn(String) -> String + 'a) -> impl Fn(String) -> String + 'a {
    move |sql| {
        let mut sql = col(sql);
        write!(&mut sql, " IN ").unwrap();
        super::enclose(&func)(sql)
    }
}

pub fn in_values<'a>(
    col: impl Fn(String) -> String + 'a, 
    params: impl Fn(String) -> String + 'a) -> impl Fn(String) -> String + 'a {
    move |sql| {
        let mut sql = col(sql);
        write!(&mut sql, " IN ").unwrap();
        super::enclose(&params)(sql)
    }
}

pub fn or<'a, 'b>(
    condition_1: impl Fn(String) -> String + 'a, 
    condition_2: impl Fn(String) -> String + 'a) -> impl Fn(String) -> String {
    move |sql| {
        let mut sql = condition_1(sql);
        write!(&mut sql, " OR ").unwrap();
        let sql = condition_2(sql);
        sql
    }
}

#[cfg(test)]
mod test {
    use super::{
        *,
        super::{
            operators::*,
            enclose
        }
    };

    #[test]
    fn all_test() {
        let sql = enclose(all(&[ 
            &eqls("id", 1),
            &eqls("name", 2),
        ]))(String::with_capacity(50));
        assert_eq!(
            "(id=$1 AND name=$2)",
            sql);
    }

    #[test]
    fn any_test() {
        let sql = enclose(any(&[ 
            &eqls("id", 1),
            &eqls("name", 2),
        ]))(String::with_capacity(50));
        assert_eq!(
            "(id=$1 OR name=$2)",
            sql);
    }

    // #[test]
    // fn condition_test() {
    //     let sql = condition(any(&[
    //         &enclose(all(&[
    //              &gt("foo", 10),
    //              &lte("foo", 100),
    //         ])),
    //         &enclose(any(&[
    //              &eqls("bar", 3),
    //              &like("bar", 4)
    //         ]))
    //     ]))(String::with_capacity(1000));
    //     assert_eq!(
    //         "WHERE (foo>$1 AND foo<=$2) OR (bar=$3 OR bar LIKE $4)",
    //         sql
    //     );
    //     // assert_eq!(5, index);
    //     // assert_eq!(4, params.len());
    // }

}

