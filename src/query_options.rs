pub trait QueryOption {
    fn to_option(&self) -> String;
}

pub struct QueryOptions {
    select: Option<Box<dyn QueryOption>>
}