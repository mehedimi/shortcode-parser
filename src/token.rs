#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Text(&'a str),
    SelfClose(&'a str),
    SelfCloseAttr(&'a str, Vec<(&'a str, Option<&'a str>)>),
    CloseTag(&'a str),
}
