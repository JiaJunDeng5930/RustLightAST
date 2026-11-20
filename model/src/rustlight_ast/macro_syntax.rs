use std::{fmt, ops::Deref, sync::Arc};

use super::{Attribute, Expr, FunctionDef, MacroInvocation, Pattern, Type, Visibility};

/// Macro input spelling and the syntax recognized at the source boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroInput {
    source: String,
    expressions: Option<Arc<[Expr]>>,
    identifiers: Option<IdentifierArguments>,
    token_identifiers: Arc<[String]>,
    token_count: usize,
    nested_macro_paths: Option<Arc<[Vec<String>]>>,
    specialized: Option<Arc<SpecializedMacroInput>>,
}

impl MacroInput {
    /// Construct the input using projections of the same token sequence.
    pub fn new(
        source: String,
        expressions: Option<Vec<Expr>>,
        identifiers: Option<(Vec<String>, bool)>,
        token_identifiers: Vec<String>,
        token_count: usize,
        nested_macro_paths: Option<Vec<Vec<String>>>,
    ) -> Self {
        Self {
            source,
            expressions: expressions.map(Into::into),
            identifiers: identifiers.map(|(values, trailing_comma)| IdentifierArguments {
                values: values.into(),
                trailing_comma,
            }),
            token_identifiers: token_identifiers.into(),
            token_count,
            nested_macro_paths: nested_macro_paths.map(Into::into),
            specialized: None,
        }
    }

    pub fn with_specialized_syntax(mut self, specialized: SpecializedMacroInput) -> Self {
        if specialized.repeat.is_some()
            || specialized.match_test.is_some()
            || specialized.statics.is_some()
            || specialized.format_captures.iter().any(Option::is_some)
        {
            self.specialized = Some(Arc::new(specialized));
        }
        self
    }

    pub(super) fn expressions(&self) -> Option<&[Expr]> {
        self.expressions.as_deref()
    }

    pub(super) fn nested_macro_paths(&self) -> Option<&[Vec<String>]> {
        self.nested_macro_paths.as_deref()
    }

    pub fn identifiers_in_tokens(&self) -> &[String] {
        &self.token_identifiers
    }

    /// Count groups and their contained tokens, excluding the invocation delimiters.
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    pub fn specialized_statics(&self) -> Option<&[MacroStatic]> {
        self.specialized()?.statics.as_deref()
    }

    pub(super) fn specialized(&self) -> Option<&SpecializedMacroInput> {
        self.specialized.as_deref()
    }
}

impl Deref for MacroInput {
    type Target = str;

    fn deref(&self) -> &str {
        &self.source
    }
}

impl fmt::Display for MacroInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct IdentifierArguments {
    values: Arc<[String]>,
    trailing_comma: bool,
}

/// Additional interpretations of the same tokens. Callers select a grammar
/// only after resolving the invocation's identity.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpecializedMacroInput {
    pub repeat: Option<(Expr, Expr)>,
    pub match_test: Option<MacroMatchInput>,
    pub statics: Option<Vec<MacroStatic>>,
    pub format_captures: [Option<Vec<String>>; 3],
}

/// The input of a macro testing one expression against a pattern and guard.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroMatchInput {
    pub value: Expr,
    pub pattern: Pattern,
    pub guard: Option<Expr>,
}

/// A static declaration in macro input, even if its initializer is opaque.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroStatic {
    pub name: String,
    pub visibility: Visibility,
    pub attrs: Vec<Attribute>,
    pub ty: Option<Type>,
    pub initializer: Option<Expr>,
}

/// Syntax evidence for a macro item kept in its original spelling.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemMacroSyntax {
    Rules {
        invocation: MacroInvocation,
        has_attributes: bool,
        function_template: Option<IdentifierMacroFunctionTemplate>,
    },
    Invocation(MacroInvocation),
    NamedInvocation { name: String, invocation: MacroInvocation },
}

impl ItemMacroSyntax {
    pub fn invocation(&self) -> &MacroInvocation {
        match self {
            Self::Rules { invocation, .. } | Self::Invocation(invocation)
            | Self::NamedInvocation { invocation, .. } => invocation,
        }
    }
}

/// A function name written literally or supplied by an identifier matcher.
#[derive(Debug, Clone, PartialEq)]
pub enum IdentifierMacroFunctionName {
    Literal(String),
    Argument(usize),
}

/// A single-rule identifier template with a function-only transcriber.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierMacroFunctionTemplate {
    arity: usize,
    trailing_comma: bool,
    functions: Vec<(IdentifierMacroFunctionName, Visibility, Option<FunctionDef>)>,
    has_local_items: bool,
}

impl IdentifierMacroFunctionTemplate {
    pub fn new(
        arity: usize,
        trailing_comma: bool,
        functions: Vec<(IdentifierMacroFunctionName, Visibility, Option<FunctionDef>)>,
        has_local_items: bool,
    ) -> Option<Self> {
        if functions.is_empty() || functions.iter().any(|(name, _, _)| {
            matches!(name, IdentifierMacroFunctionName::Argument(index) if *index >= arity)
        }) {
            return None;
        }
        Some(Self {
            arity,
            trailing_comma,
            functions,
            has_local_items,
        })
    }

    pub fn has_local_items(&self) -> bool {
        self.has_local_items
    }

    /// Resolve the names introduced by this syntactically matching call.
    /// The caller must establish which macro definition the invocation denotes.
    pub fn functions<'a>(
        &'a self,
        invocation: &'a MacroInvocation,
    ) -> Option<impl Iterator<Item = (&'a str, &'a Visibility)>> {
        Some(self.resolved_functions(invocation)?.map(|(name, visibility, _)| (name, visibility)))
    }

    /// Return signatures whose types do not depend on macro arguments.
    pub fn fixed_signatures<'a>(
        &'a self,
        invocation: &'a MacroInvocation,
    ) -> Option<impl Iterator<Item = FunctionDef> + 'a> {
        Some(self.resolved_functions(invocation)?.filter_map(|(name, _, signature)| {
            let mut signature = signature?.clone();
            signature.name = name.to_owned();
            Some(signature)
        }))
    }

    fn resolved_functions<'a>(
        &'a self,
        invocation: &'a MacroInvocation,
    ) -> Option<impl Iterator<Item = (&'a str, &'a Visibility, Option<&'a FunctionDef>)>> {
        let arguments = invocation.tokens.identifiers.as_ref()?;
        if arguments.values.len() != self.arity || arguments.trailing_comma != self.trailing_comma {
            return None;
        }
        Some(self.functions.iter().map(move |(name, visibility, signature)| {
            let name = match name {
                IdentifierMacroFunctionName::Literal(name) => name,
                IdentifierMacroFunctionName::Argument(index) => &arguments.values[*index],
            };
            (name.as_str(), visibility, signature.as_ref())
        }))
    }
}
