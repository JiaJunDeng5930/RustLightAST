// crates/rustlightast/src/rustlight_ast.rs

mod pattern;

pub use pattern::{BindingMode, Pattern, PatternField, PatternPath};

/// Lightweight Rust abstract syntax tree (module level)
#[derive(Debug, Clone, PartialEq)]
pub struct RustModule {
    pub name: String,
    /// Outer documentation on an inline module declaration.
    pub docs: Vec<String>,
    /// Inner attributes, if any, precede the declarations as raw items.
    pub items: Vec<Item>,
    /// Outer attributes on an inline module declaration.
    pub attrs: Vec<Attribute>,
    pub vis: Visibility, // controls the visibility of the module
}

/// Module item definitions
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Raw(String), // raw Rust item or crate-level directive
    Struct(StructDef),
    Enum(EnumDef),
    Union(UnionDef),
    Function(FunctionDef),
    Impl(ImplBlock),
    Trait(TraitDef),
    Const(ConstDef),
    TypeAlias(TypeAlias),
    Use(UseStatement),
    Mod(Box<RustModule>),      // nested module
    ModuleDecl(ModuleDecl),    // out-of-line module
    LazyStatic(LazyStaticDef), // lazy_static! macro
}

/// Struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub attrs: Vec<DataAttribute>,
    pub name: String,
    pub fields: Fields,
    pub generics: Generics,
    pub docs: Vec<String>,
    pub vis: Visibility, // controls struct visibility
}

/// Union definition
#[derive(Debug, Clone, PartialEq)]
pub struct UnionDef {
    pub attrs: Vec<DataAttribute>,
    pub name: String,
    pub fields: Vec<Field>, // union fields
    pub generics: Generics,
    pub docs: Vec<String>,
    pub vis: Visibility, // controls union visibility
}

/// Enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub attrs: Vec<DataAttribute>,
    pub name: String,
    pub variants: Vec<Variant>,
    pub generics: Generics,
    pub docs: Vec<String>,
    pub vis: Visibility,
}

/// Function definition
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub generics: Generics,
    pub body: Block,
    pub asyncness: bool,
    pub qualifiers: FunctionQualifiers,
    pub vis: Visibility,
    pub docs: Vec<String>,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FunctionQualifiers {
    pub constness: bool,
    pub unsafety: bool,
    pub abi: Option<FunctionAbi>,
    /// Whether the signature explicitly wrote a return type.
    pub explicit_return_type: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionAbi {
    Default,
    Named(String),
}

/// A trait declaration. Default methods share the function and block model
/// used by free functions and impl methods.
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: String,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
    pub generics: Generics,
    pub supertraits: Vec<String>,
    pub unsafety: bool,
    pub auto: bool,
    pub members: Vec<TraitMember>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitMember {
    DefaultMethod(FunctionDef),
    Raw(String),
}

/// Implementation block
#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub unsafety: bool,
    pub attrs: Vec<Attribute>,
    pub target: Type,
    pub generics: Generics,
    pub items: Vec<ImplItem>,
    pub trait_impl: Option<Type>, // implemented for which trait
}

/// Constant definition
#[derive(Debug, Clone, PartialEq)]
pub struct ConstDef {
    pub attrs: Vec<Attribute>,
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub vis: Visibility,
    pub docs: Vec<String>,
}

/// Type alias
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub attrs: Vec<Attribute>,
    pub name: String,
    pub target: Type,
    pub generics: Generics,
    pub vis: Visibility,
    pub docs: Vec<String>,
}

// ========== Basic type definitions ========== //

/// Type representation
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Path(Vec<String>),                // std::vec::Vec
    Named(String),                    // i32, String
    /// A named type applied to arguments, such as `HashMap<K, V>`.
    Generic(String, Vec<GenericArgument>),
    CallableTrait(CallableTraitType), // dyn Fn(A) -> B / impl Fn(A) -> B
    BareFn(Box<BareFnType>),
    RawPointer(Box<Type>, PointerKind),
    ImplTrait {
        trait_path: Vec<String>,
        associated_types: Vec<AssociatedTypeBinding>,
    },
    /// Reference target, optional explicit lifetime, and mutability.
    Reference(Box<Type>, Option<String>, bool),
    Tuple(Vec<Type>),        // (T1, T2)
    Slice(Box<Type>),        // [T]
    Array(Box<Type>, Box<Expr>), // [T; N] fixed-size array
    Unit,                    // ()
    Never,                   // !
    Infer,                   // _
}

impl Type {
}

/// The syntax of an argument in an angle-bracketed generic argument list.
#[derive(Debug, Clone, PartialEq)]
pub enum GenericArgument {
    /// Includes syntax shared with const arguments, such as `N` and `_`.
    /// Resolving that syntax requires information about the generic declaration.
    Type(Type),
    /// Spelling including the apostrophe.
    Lifetime(String),
    Const(Box<Expr>),
}

impl GenericArgument {
    pub fn as_type(&self) -> Option<&Type> {
        match self {
            Self::Type(ty) => Some(ty),
            Self::Lifetime(_) | Self::Const(_) => None,
        }
    }
}

impl From<Type> for GenericArgument {
    fn from(ty: Type) -> Self {
        Self::Type(ty)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerKind {
    Const,
    Mut,
}

/// A bare function pointer. An omitted result type has Rust's unit semantics.
#[derive(Debug, Clone, PartialEq)]
pub struct BareFnType {
    pub higher_ranked: Vec<LifetimeParameter>,
    pub unsafety: bool,
    pub abi: Option<FunctionAbi>,
    pub inputs: Vec<BareFnParam>,
    pub variadic: Option<BareFnVariadic>,
    pub output: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BareFnParam {
    pub attrs: Vec<Attribute>,
    pub name: Option<String>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BareFnVariadic {
    pub attrs: Vec<Attribute>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssociatedTypeBinding {
    pub name: String,
    pub ty: Type,
}

/// Callable trait type such as `dyn Fn(A) -> B` or `impl Fn(A) -> B`.
#[derive(Debug, Clone, PartialEq)]
pub struct CallableTraitType {
    pub qualifier: CallableTraitQualifier,
    /// Namespace segments of the callable trait, with an empty first segment
    /// for an absolute path.
    pub trait_path: Vec<String>,
    pub args: Vec<Type>,
    pub return_type: Box<Type>,
    pub leading_bounds: Vec<CallableBound>,
    pub trailing_bounds: Vec<CallableBound>,
    pub higher_ranked: Vec<LifetimeParameter>,
}

/// A bound on the callable in addition to its call signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallableBound {
    Lifetime(String),
    Trait(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallableTraitQualifier {
    Dyn,
    Impl,
}

/// A closure parameter with its pattern and optional type kept separately.
///
/// Keeping the type structured lets optimization passes inspect and rewrite
/// closure annotations without reparsing a printed `pattern: Type` string.
#[derive(Debug, Clone, PartialEq)]
pub struct ClosureParam {
    pub pattern: Pattern,
    pub ty: Option<Type>,
}

impl ClosureParam {
    pub fn untyped(pattern: Pattern) -> Self {
        Self { pattern, ty: None }
    }

    pub fn typed(pattern: Pattern, ty: Type) -> Self {
        Self {
            pattern,
            ty: Some(ty),
        }
    }

    pub(crate) fn binding(name: impl Into<String>) -> Self {
        Self::untyped(Pattern::binding(name))
    }

}

impl From<&str> for ClosureParam {
    fn from(name: &str) -> Self {
        Self::binding(name)
    }
}

impl From<String> for ClosureParam {
    fn from(name: String) -> Self {
        Self::binding(name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    Namespace, // separated by :: (e.g., std::thread)
    NamespaceWithTypeArguments(Vec<PathTypeArguments>),
    Qualified(PathQualification),
}

/// Generic arguments attached to a namespace segment.
#[derive(Debug, Clone, PartialEq)]
pub struct PathTypeArguments {
    pub segment: usize,
    pub arguments: Vec<GenericArgument>,
}

/// The `<T>` or `<T as Trait>` prefix of an associated expression path.
#[derive(Debug, Clone, PartialEq)]
pub struct PathQualification {
    pub self_type: QualifiedSelfType,
    pub trait_path: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QualifiedSelfType {
    Type(Box<Type>),
    /// Preserved syntax outside the type model. Consumers cannot infer its type.
    Verbatim(String),
}

/// A path used by a record constructor.
#[derive(Debug, Clone, PartialEq)]
pub enum RecordPath {
    /// Namespace segments with any unmodeled argument syntax preserved.
    Segments(Vec<String>),
    WithTypeArguments {
        /// The final segment contains just the constructor name.
        segments: Vec<String>,
        arguments: Vec<GenericArgument>,
    },
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    /// An opaque macro invocation preserved as Rust source, e.g. `panic!("msg")`.
    Macro(Box<MacroExpr>),
    Path(Vec<String>, PathType),
    Field {
        base: Box<Expr>,
        member: Member,
    },
    Record {
        path: RecordPath,
        fields: Vec<RecordField>,
        rest: Option<Box<Expr>>,
    },
    Literal(Literal),
    Array(Vec<Expr>),
    Repeat {
        value: Box<Expr>,
        len: Box<Expr>,
    },
    Range {
        start: Option<Box<Expr>>,
        end: RangeEnd,
    },
    Tuple(Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, MethodName, Vec<Expr>),
    Block(Block),
    LabeledBlock {
        label: String,
        body: Block,
    },
    Loop(Box<Block>, Option<String>),
    While {
        label: Option<String>,
        condition: Box<Expr>,
        body: Block,
    },
    For {
        label: Option<String>,
        pattern: Pattern,
        iter: Box<Expr>,
        body: Block,
    },
    Let {
        pattern: Pattern,
        value: Box<Expr>,
    },
    Return(Option<Box<Expr>>),
    Break {
        label: Option<String>,
        value: Option<Box<Expr>>,
    },
    Continue(Option<String>),
    Async {
        body: Block,
        move_capture: bool,
    },
    Await(Box<Expr>),
    Try(Box<Expr>),
    /// `(params, body, is_move)` — third field is `true` for `move |…| …` closures.
    Closure(Vec<ClosureParam>, Box<Expr>, bool),
    /// A closure whose explicit Rust return type must survive parse/print.
    TypedClosure(Vec<ClosureParam>, Type, Box<Expr>, bool),
    AsyncClosure {
        params: Vec<ClosureParam>,
        return_type: Option<Type>,
        body: Box<Expr>,
        move_capture: bool,
    },
    BuilderChain(Vec<BuilderMethod>), // represents builder-style chained calls
    Unsafe(Box<Block>),               // unsafe expression support
    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
    }, // conditional expression
    IfLet {
        pattern: Pattern,
        value: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    }, // match expression
    Reference(Box<Expr>, bool, bool), // &expr, first flag for &, second flag for mut
    BinaryOp(Box<Expr>, String, Box<Expr>), // left op right, e.g., a != 0
    UnaryOp(String, Box<Expr>),       // op expr, e.g., !x, -y
    Index(Box<Expr>, Box<Expr>),      // expr[index], e.g., array[i]
    Parenthesized(Box<Expr>),         // (expr), e.g., (a + b)
    Cast(Box<Expr>, Type),            // expr as Type
    Assign(Box<Expr>, Box<Expr>), // left = right, e.g., state = State::S0 // lazy_static! macro support
}

/// A method name and the explicit generic arguments written after it.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodName {
    pub name: String,
    pub generic_arguments: Option<Vec<GenericArgument>>,
}

impl From<String> for MethodName {
    fn from(name: String) -> Self {
        Self { name, generic_arguments: None }
    }
}

impl From<&str> for MethodName {
    fn from(name: &str) -> Self {
        name.to_owned().into()
    }
}

/// A half-open range can omit its end. An inclusive range requires one.
#[derive(Debug, Clone, PartialEq)]
pub enum RangeEnd {
    Exclusive(Option<Box<Expr>>),
    Inclusive(Box<Expr>),
}

impl RangeEnd {
    pub fn expression(&self) -> Option<&Expr> {
        match self {
            Self::Exclusive(end) => end.as_deref(),
            Self::Inclusive(end) => Some(end),
        }
    }

    pub fn expression_mut(&mut self) -> Option<&mut Expr> {
        match self {
            Self::Exclusive(end) => end.as_deref_mut(),
            Self::Inclusive(end) => Some(end),
        }
    }

    pub fn map_expression(&self, rewrite: impl FnOnce(&Expr) -> Expr) -> Self {
        match self {
            Self::Exclusive(end) => {
                Self::Exclusive(end.as_deref().map(|end| Box::new(rewrite(end))))
            }
            Self::Inclusive(end) => Self::Inclusive(Box::new(rewrite(end))),
        }
    }
}

/// A named field or the position of a tuple field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Member {
    Named(String),
    Index(u32),
}

/// The value of a field initializer, in source evaluation order.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordField {
    pub member: Member,
    pub value: Expr,
    /// Preserve `field` when the value is still the same-named binding.
    pub shorthand: bool,
}

// Match expression arm
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Block,
    /// An explicit block in the source remains a block even with no statements.
    pub braced: bool,
    pub comma: bool,
}

// Distinguish different builder methods such as .name(), .stack_size(), etc.
#[derive(Debug, Clone, PartialEq)]
pub enum BuilderMethod {
    Named(String), // e.g., .name("thread_name")
    //StackSize(Box<Expr>), // TODO .stack_size(expr)
    Spawn {
        closure: Box<Expr>,
        move_kw: bool, // move semantics handled in BuilderMethod
    },
}

/// Literal
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Literal source preserved verbatim when its spelling carries Rust type or
    /// escaping information that must survive a parse/print round trip.
    Raw(String),
    /// An integer token with its radix, separators and optional type suffix.
    Integer {
        digits: String,
        suffix: Option<String>,
    },
    /// A byte token, retaining its exact escape spelling.
    Byte { spelling: String, value: u8 },
    /// A character token, retaining its exact escape spelling.
    CharToken { spelling: String, value: char },
    /// A floating-point token with its original mantissa, exponent and suffix.
    FloatToken { digits: String, suffix: Option<String> },
    /// A string token, retaining its original escaping and raw delimiters.
    StringToken { spelling: String, value: String },
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Char(char),
}

/// Code block
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Statement>,
    pub expr: Option<Box<Expr>>,
}

/// Statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(LetStmt),
    LetElse {
        attrs: Vec<String>,
        pattern: Pattern,
        value: Box<Expr>,
        else_block: Block,
    },
    Expr(Expr),
    Macro(MacroStmt),
    Item(Box<Item>),
    Comment(String), // comment statement
}

/// A macro invocation whose input follows the grammar of the named macro.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroInvocation {
    /// An empty first segment represents a leading `::`.
    pub path: Vec<String>,
    pub delimiter: MacroDelimiter,
    pub tokens: String,
}

/// An expression-position macro invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroExpr {
    pub attrs: Vec<String>,
    pub invocation: MacroInvocation,
}

/// A statement-position macro invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroStmt {
    pub attrs: Vec<String>,
    pub invocation: MacroInvocation,
    pub semicolon: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroDelimiter {
    Parenthesis,
    Brace,
    Bracket,
}

impl MacroDelimiter {
    pub fn brackets(self) -> (char, char) {
        match self {
            Self::Parenthesis => ('(', ')'),
            Self::Brace => ('{', '}'),
            Self::Bracket => ('[', ']'),
        }
    }
}

/// let binding
#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub attrs: Vec<String>,
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
}

// ========== Auxiliary types ========== //

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Restricted(Vec<String>), // pub(in path)
    None,                    // no visibility needed for trait implementations, follows the trait
}

#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
    Structured {
        name: String,
        args: Vec<AttributeArg>,
    },
    /// A parsed Rust attribute with its original token structure.
    Source { meta: String, inner: bool },
}

impl Attribute {
    pub fn is_inner(&self) -> bool {
        matches!(self, Self::Source { inner: true, .. })
    }

    pub fn is_doc(&self) -> bool {
        match self {
            Self::Structured { name, .. } => name == "doc",
            Self::Source { meta, .. } => meta.strip_prefix("doc").is_some_and(|tail| {
                let tail = tail.trim_start();
                tail.is_empty() || tail.starts_with(['=', '(', '[', '{'])
            }),
        }
    }
}

/// Attributes on a data declaration, in source order.
#[derive(Debug, Clone, PartialEq)]
pub enum DataAttribute {
    Derive {
        /// Paths use an empty first segment for a leading `::`.
        paths: Vec<Vec<String>>,
        trailing_comma: bool,
    },
    Other(Attribute),
}

impl DataAttribute {
    pub fn push_derive(attrs: &mut Vec<Self>, path: Vec<String>) {
        if let Some(paths) = attrs.iter_mut().rev().find_map(|attr| match attr {
            Self::Derive { paths, .. } => Some(paths),
            Self::Other(_) => None,
        }) {
            paths.push(path);
        } else {
            attrs.push(Self::Derive {
                paths: vec![path],
                trailing_comma: false,
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeArg {
    Ident(String),
    Literal(Literal),
    KeyValue(String, Literal),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub vis: Visibility,
    pub docs: Vec<String>,
    pub attrs: Vec<Attribute>,
}

/// Field forms shared by structures and enum variants. Empty tuple and
/// record constructors retain their syntax independently of unit constructors.
#[derive(Debug, Clone, PartialEq)]
pub enum Fields {
    Unit,
    Tuple(Vec<Field>),
    Named(Vec<Field>),
}

impl Fields {
    pub fn as_slice(&self) -> &[Field] {
        match self {
            Self::Unit => &[],
            Self::Tuple(fields) | Self::Named(fields) => fields,
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [Field] {
        match self {
            Self::Unit => &mut [],
            Self::Tuple(fields) | Self::Named(fields) => fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Fields,
    pub discriminant: Option<Expr>,
    /// Outer attributes, including doc attributes, in declaration order.
    pub attrs: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    /// Constraints written in the where clause.
    pub where_predicates: Vec<WherePredicate>,
}

impl From<Vec<GenericParam>> for Generics {
    fn from(params: Vec<GenericParam>) -> Self {
        Self {
            params,
            where_predicates: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WherePredicate {
    Type {
        subject: Type,
        bounds: Vec<TypeBound>,
    },
    Lifetime {
        subject: String,
        bounds: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericParam {
    Type(TypeParameter),
    Lifetime(LifetimeParameter),
    Const {
        name: String,
        ty: Type,
        default: Option<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParameter {
    pub name: String,
    pub bounds: Vec<TypeBound>,
    pub default: Option<Type>,
}

/// A type parameter or where-predicate bound.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeBound {
    Lifetime(String),
    Trait(TraitBound),
    /// Syntax retained when the structured bound cannot represent every argument.
    Opaque(String),
}

impl TypeBound {

    /// Create a required trait bound without generic arguments.
    pub(crate) fn trait_path<I, S>(absolute: bool, path: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Trait(TraitBound {
            absolute,
            higher_ranked: Vec::new(),
            optional: false,
            parenthesized: false,
            path: path
                .into_iter()
                .map(|name| TraitBoundSegment {
                    name: name.into(),
                    arguments: TraitBoundArguments::None,
                })
                .collect(),
        })
    }


    pub fn lifetime(&self) -> Option<&str> {
        match self {
            Self::Lifetime(name) => Some(name),
            _ => None,
        }
    }

}

impl From<String> for TypeBound {
    fn from(source: String) -> Self {
        fn identifier(spelling: &str) -> bool {
            let mut chars = spelling.strip_prefix("r#").unwrap_or(spelling).chars();
            chars
                .next()
                .is_some_and(|ch| ch == '_' || ch.is_alphabetic())
                && chars.all(|ch| ch == '_' || ch.is_alphanumeric())
        }

        if source.strip_prefix("'").is_some_and(identifier) {
            return Self::Lifetime(source);
        }
        let absolute = source.starts_with("::");
        let path = source.strip_prefix("::").unwrap_or(&source);
        if path.split("::").all(identifier) {
            Self::trait_path(absolute, path.split("::"))
        } else {
            Self::Opaque(source)
        }
    }
}

impl From<&str> for TypeBound {
    fn from(source: &str) -> Self {
        source.to_owned().into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitBound {
    pub absolute: bool,
    pub higher_ranked: Vec<LifetimeParameter>,
    pub optional: bool,
    pub parenthesized: bool,
    pub path: Vec<TraitBoundSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitBoundSegment {
    pub name: String,
    pub arguments: TraitBoundArguments,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitBoundArguments {
    None,
    Angle(Vec<TraitBoundArgument>),
    Parenthesized {
        inputs: Vec<Type>,
        output: Option<Box<Type>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitBoundArgument {
    Generic(GenericArgument),
    AssociatedType {
        name: String,
        arguments: Option<Vec<TraitBoundArgument>>,
        value: Type,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifetimeParameter {
    /// Spelling including the apostrophe.
    pub name: String,
    pub bounds: Vec<String>,
}

impl GenericParam {
    pub fn name(&self) -> &str {
        match self {
            Self::Type(param) => &param.name,
            Self::Lifetime(param) => &param.name,
            Self::Const { name, .. } => name,
        }
    }


    pub fn type_parameter(&self) -> Option<&TypeParameter> {
        match self {
            Self::Type(param) => Some(param),
            Self::Lifetime(_) | Self::Const { .. } => None,
        }
    }

    pub fn type_parameter_mut(&mut self) -> Option<&mut TypeParameter> {
        match self {
            Self::Type(param) => Some(param),
            Self::Lifetime(_) | Self::Const { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    /// Mutability of the binding, independently of a reference type's mutability.
    pub mutable: bool,
    /// The written pattern for parameters that cannot use the simple name form.
    pub pattern: Option<Pattern>,
}

impl Param {
    pub fn binding_name(&self) -> Option<&str> {
        (!self.name.is_empty() && self.name != "_").then_some(self.name.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplItem {
    Method(FunctionDef),
    AssocConst(ConstDef),
    AssocType(TypeAlias),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub name: String,
    pub vis: Visibility,
    /// Outer attributes in declaration order, including doc attributes.
    pub attrs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    pub tree: UseTree,
    pub absolute: bool,
    pub vis: Visibility,
    /// Outer attributes in declaration order, including doc attributes.
    pub attrs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseTree {
    Path { segment: String, tree: Box<UseTree> },
    Name(String),
    Rename { name: String, alias: String },
    Glob,
    Group(Vec<UseTree>),
}

/// lazy_static! macro definition
#[derive(Debug, Clone, PartialEq)]
pub struct LazyStaticDef {
    pub name: String,
    pub ty: Type,
    pub init: Block,
    pub vis: Visibility,
    pub docs: Vec<String>,
}
