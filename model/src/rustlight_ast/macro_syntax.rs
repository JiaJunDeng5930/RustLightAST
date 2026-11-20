use std::sync::Arc;

use super::{Attribute, Expr, FunctionDef, MacroInvocation, Pattern, Type, Visibility};

/// Macro input as written at ingress, or as constructed by an AST rewrite.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroInput {
    syntax: MacroInputSyntax,
    token_identifiers: Arc<[String]>,
    syntax_size: usize,
    nested_macro_paths: Option<Arc<[Vec<String>]>>,
}

#[derive(Debug, Clone, PartialEq)]
enum MacroInputSyntax {
    Written {
        source: String,
        expressions: Option<Arc<[Expr]>>,
        identifiers: Option<IdentifierArguments>,
        specialized: Option<Arc<SpecializedMacroInput>>,
    },
    Modeled(Arc<ModeledMacroArguments>),
}

/// A macro argument grammar selected by the caller after resolving the macro.
#[derive(Debug, Clone, PartialEq)]
pub enum ModeledMacroInput {
    Expressions {
        arguments: Vec<Expr>,
        format_captures: [Option<Vec<String>>; 3],
    },
    Repeat(Expr, Expr),
    Match(MacroMatchInput),
    Statics(Vec<MacroStatic>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ModeledMacroArguments {
    pub(crate) input: ModeledMacroInput,
    pub(crate) trailing_comma: bool,
}

impl MacroInput {
    /// Construct the written input and its projections at the source boundary.
    pub fn new(
        source: String,
        expressions: Option<Vec<Expr>>,
        identifiers: Option<(Vec<String>, bool)>,
        token_identifiers: Vec<String>,
        token_count: usize,
        nested_macro_paths: Option<Vec<Vec<String>>>,
    ) -> Self {
        Self {
            syntax: MacroInputSyntax::Written {
                source,
                expressions: expressions.map(Into::into),
                identifiers: identifiers.map(|(values, trailing_comma)| IdentifierArguments {
                    values: values.into(),
                    trailing_comma,
                }),
                specialized: None,
            },
            token_identifiers: token_identifiers.into(),
            syntax_size: token_count,
            nested_macro_paths: nested_macro_paths.map(Into::into),
        }
    }

    pub fn with_specialized_syntax(mut self, specialized: SpecializedMacroInput) -> Self {
        if let MacroInputSyntax::Written { specialized: slot, .. } = &mut self.syntax {
            if specialized.repeat.is_some()
                || specialized.match_test.is_some()
                || specialized.statics.is_some()
                || specialized.format_captures.iter().any(Option::is_some)
            {
                *slot = Some(Arc::new(specialized));
            }
        }
        self
    }

    /// The original token spelling, available for macros read from Rust source.
    pub fn written_source(&self) -> Option<&str> {
        match &self.syntax {
            MacroInputSyntax::Written { source, .. } => Some(source),
            MacroInputSyntax::Modeled(_) => None,
        }
    }

    pub fn trailing_comma(&self) -> bool {
        match &self.syntax {
            MacroInputSyntax::Written { source, .. } => source.trim_end().ends_with(','),
            MacroInputSyntax::Modeled(arguments) => arguments.trailing_comma,
        }
    }

    pub(crate) fn modeled_arguments(&self) -> Option<&ModeledMacroArguments> {
        match &self.syntax {
            MacroInputSyntax::Written { .. } => None,
            MacroInputSyntax::Modeled(arguments) => Some(arguments),
        }
    }

    pub(super) fn expressions(&self) -> Option<&[Expr]> {
        match &self.syntax {
            MacroInputSyntax::Written { expressions, .. } => expressions.as_deref(),
            MacroInputSyntax::Modeled(arguments) => match &arguments.input {
                ModeledMacroInput::Expressions { arguments, .. } => Some(arguments),
                _ => None,
            },
        }
    }

    pub(super) fn nested_macro_paths(&self) -> Option<&[Vec<String>]> {
        self.nested_macro_paths.as_deref()
    }

    pub fn identifiers_in_tokens(&self) -> &[String] {
        &self.token_identifiers
    }

    /// The number of tokens read at ingress, including groups.
    pub fn written_token_count(&self) -> Option<usize> {
        self.written_source().map(|_| self.syntax_size)
    }

    /// A work estimate from the tokens read at ingress or modeled syntax nodes.
    pub fn syntax_size(&self) -> usize {
        self.syntax_size
    }

    pub fn specialized_statics(&self) -> Option<&[MacroStatic]> {
        if let Some(arguments) = self.modeled_arguments() {
            if let ModeledMacroInput::Statics(items) = &arguments.input {
                return Some(items);
            }
            return None;
        }
        self.specialized()?.statics.as_deref()
    }

    pub(super) fn repeat(&self) -> Option<(&Expr, &Expr)> {
        if let Some(arguments) = self.modeled_arguments() {
            if let ModeledMacroInput::Repeat(value, count) = &arguments.input {
                return Some((value, count));
            }
            return None;
        }
        let (value, count) = self.specialized()?.repeat.as_ref()?;
        Some((value, count))
    }

    pub(super) fn match_test(&self) -> Option<&MacroMatchInput> {
        if let Some(arguments) = self.modeled_arguments() {
            if let ModeledMacroInput::Match(input) = &arguments.input {
                return Some(input);
            }
            return None;
        }
        self.specialized()?.match_test.as_ref()
    }

    pub(super) fn format_captures(&self, leading_operands: usize) -> Option<&[String]> {
        let captures = if let Some(arguments) = self.modeled_arguments() {
            let ModeledMacroInput::Expressions { format_captures, .. } = &arguments.input else {
                return None;
            };
            format_captures
        } else {
            &self.specialized()?.format_captures
        };
        captures.get(leading_operands)?.as_deref()
    }

    fn specialized(&self) -> Option<&SpecializedMacroInput> {
        match &self.syntax {
            MacroInputSyntax::Written { specialized, .. } => specialized.as_deref(),
            MacroInputSyntax::Modeled(_) => None,
        }
    }

    fn identifier_arguments(&self) -> Option<&IdentifierArguments> {
        match &self.syntax {
            MacroInputSyntax::Written { identifiers, .. } => identifiers.as_ref(),
            MacroInputSyntax::Modeled(_) => None,
        }
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
        let arguments = invocation.tokens.identifier_arguments()?;
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
