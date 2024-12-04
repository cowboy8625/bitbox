use std::any::Any;

pub type Span = std::ops::Range<usize>;

pub trait Token: std::fmt::Debug + Any {
    fn new(lexeme: impl Into<String>, span: Span) -> Self
    where
        Self: Sized;
    fn get_lexeme(&self) -> String;
    fn get_span(&self) -> Span;
    fn as_any(&self) -> &dyn Any;
}

pub fn create<T>(lexeme: impl Into<String>, span: Span) -> Box<dyn Token>
where
    T: Token,
{
    Box::new(T::new(lexeme, span))
}
