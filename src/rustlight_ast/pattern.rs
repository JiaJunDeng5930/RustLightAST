use super::{Expr, Literal, Member, RangeEnd, Type};

/// A Rust pattern used by bindings, parameters, and match arms.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// A bare identifier may name a binding or a constant. Resolution decides
    /// which it denotes when no binding modifier or subpattern is present.
    Ident {
        name: String,
        mode: BindingMode,
        subpattern: Option<Box<Pattern>>,
    },
    Path(Vec<String>),
    Literal(Literal),
    Range {
        start: Option<Box<Expr>>,
        end: RangeEnd,
    },
    /// Source retained when a supported expression contains an unmodeled pattern.
    Opaque(String),
    Wildcard,
    Rest,
    Tuple(Vec<Pattern>),
    TupleStruct {
        path: Vec<String>,
        fields: Vec<Pattern>,
    },
    Record {
        path: Vec<String>,
        fields: Vec<PatternField>,
        rest: bool,
    },
    Or(Vec<Pattern>),
    Slice(Vec<Pattern>),
    Typed {
        pattern: Box<Pattern>,
        ty: Type,
    },
    Reference {
        pattern: Box<Pattern>,
        mutable: bool,
    },
    Parenthesized(Box<Pattern>),
}

impl Pattern {
    /// Create an ordinary identifier binding, or a wildcard for `_`.
    pub(crate) fn binding(name: impl Into<String>) -> Self {
        Self::binding_with_mode(name, BindingMode::Value)
    }

    /// Create an identifier binding with its written binding modifier.
    pub(crate) fn binding_with_mode(name: impl Into<String>, mode: BindingMode) -> Self {
        let name = name.into();
        if name == "_" && mode == BindingMode::Value {
            return Self::Wildcard;
        }
        Self::Ident {
            name,
            mode,
            subpattern: None,
        }
    }



}

/// The binding modifiers written on an identifier pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    Value,
    Mut,
    Ref,
    RefMut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternField {
    pub member: Member,
    pub pattern: Pattern,
    /// Request shorthand if the pattern still binds the same-named field.
    pub shorthand: bool,
}
