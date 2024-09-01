use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use lalrpop_util::lexer::Token;

pub type Span = core::ops::Range<usize>;

#[derive(Debug, Clone)]
pub enum Report<'src> {
	LexerInvalidToken {
		span: Span,
	},

	ParserUnexpectedEOF {
		span: Span,
		expected: Vec<String>,
	},

	ParserUnexpectedToken {
		span: Span,
		expected: Vec<String>,
		token: Token<'src>,
	},

	ParserExtraToken {
		span: Span,
	},

	ParserExpectedInt {
		span: Span,
	},

	AnalyzeEmptyEvDecls,

	AnalyzeOversizeUnreliable {
		ev_span: Span,
		ty_span: Span,
		max_size: usize,
		size: usize,
	},

	AnalyzePotentiallyOversizeUnreliable {
		ev_span: Span,
		ty_span: Span,
		max_size: usize,
	},

	AnalyzeInvalidRange {
		span: Span,
	},

	AnalyzeEmptyEnum {
		span: Span,
	},

	AnalyzeEnumTagUsed {
		tag_span: Span,
		used_span: Span,
		tag: &'src str,
	},

	AnalyzeInvalidOptValue {
		span: Span,
		expected: &'static str,
	},

	AnalyzeUnknownOptName {
		span: Span,
	},

	AnalyzeUnknownTypeRef {
		span: Span,
		name: &'src str,
	},

	AnalyzeNumOutsideRange {
		span: Span,
		min: f64,
		max: f64,
	},

	AnalyzeInvalidOptionalType {
		span: Span,
	},

	AnalyzeUnboundedRecursiveType {
		decl_span: Span,
		use_span: Span,
	},

	AnalyzeMissingOptValue {
		expected: &'static str,
		required_when: &'static str,
	},

	AnalyzeDuplicateDecl {
		prev_span: Span,
		dup_span: Span,
		name: &'src str,
	},
}

impl<'src> Report<'src> {
	pub fn severity(&self) -> Severity {
		match self {
			Self::LexerInvalidToken { .. } => Severity::Error,

			Self::ParserUnexpectedEOF { .. } => Severity::Error,
			Self::ParserUnexpectedToken { .. } => Severity::Error,
			Self::ParserExtraToken { .. } => Severity::Error,
			Self::ParserExpectedInt { .. } => Severity::Error,

			Self::AnalyzeEmptyEvDecls => Severity::Warning,
			Self::AnalyzeOversizeUnreliable { .. } => Severity::Error,
			Self::AnalyzePotentiallyOversizeUnreliable { .. } => Severity::Warning,
			Self::AnalyzeInvalidRange { .. } => Severity::Error,
			Self::AnalyzeEmptyEnum { .. } => Severity::Error,
			Self::AnalyzeEnumTagUsed { .. } => Severity::Error,
			Self::AnalyzeInvalidOptValue { .. } => Severity::Error,
			Self::AnalyzeUnknownOptName { .. } => Severity::Warning,
			Self::AnalyzeUnknownTypeRef { .. } => Severity::Error,
			Self::AnalyzeNumOutsideRange { .. } => Severity::Error,
			Self::AnalyzeInvalidOptionalType { .. } => Severity::Error,
			Self::AnalyzeUnboundedRecursiveType { .. } => Severity::Error,
			Self::AnalyzeMissingOptValue { .. } => Severity::Error,
			Self::AnalyzeDuplicateDecl { .. } => Severity::Error,
		}
	}

	fn message(&self) -> String {
		match self {
			Self::LexerInvalidToken { .. } => "invalid token".to_string(),

			Self::ParserUnexpectedEOF { expected, .. } => {
				format!("expected {}, found end of file", expected.join(", "))
			}

			Self::ParserUnexpectedToken { expected, .. } => format!("expected {}", expected.join(", ")),
			Self::ParserExtraToken { .. } => "extra token".to_string(),
			Self::ParserExpectedInt { .. } => "expected integer".to_string(),

			Self::AnalyzeEmptyEvDecls => "no event or function declarations".to_string(),
			Self::AnalyzeOversizeUnreliable { .. } => "oversize unreliable".to_string(),
			Self::AnalyzePotentiallyOversizeUnreliable { .. } => "potentially oversize unreliable".to_string(),
			Self::AnalyzeInvalidRange { .. } => "invalid range".to_string(),
			Self::AnalyzeEmptyEnum { .. } => "empty enum".to_string(),
			Self::AnalyzeEnumTagUsed { .. } => "enum tag used in variant".to_string(),
			Self::AnalyzeInvalidOptValue { expected, .. } => format!("invalid opt value, expected {}", expected),
			Self::AnalyzeUnknownOptName { .. } => "unknown opt name".to_string(),
			Self::AnalyzeUnknownTypeRef { name, .. } => format!("unknown type reference '{}'", name),
			Self::AnalyzeNumOutsideRange { .. } => "number outside range".to_string(),
			Self::AnalyzeInvalidOptionalType { .. } => "invalid optional type".to_string(),
			Self::AnalyzeUnboundedRecursiveType { .. } => "unbounded recursive type".to_string(),
			Self::AnalyzeMissingOptValue { .. } => "missing option expected".to_string(),
			Self::AnalyzeDuplicateDecl { name, .. } => format!("duplicate declaration '{}'", name),
		}
	}

	fn code(&self) -> &str {
		match self {
			Self::LexerInvalidToken { .. } => "1001",

			Self::ParserUnexpectedEOF { .. } => "2001",
			Self::ParserUnexpectedToken { .. } => "2002",
			Self::ParserExtraToken { .. } => "2003",
			Self::ParserExpectedInt { .. } => "2004",

			Self::AnalyzeEmptyEvDecls { .. } => "3001",
			Self::AnalyzeOversizeUnreliable { .. } => "3002",
			Self::AnalyzePotentiallyOversizeUnreliable { .. } => "3003",
			Self::AnalyzeInvalidRange { .. } => "3004",
			Self::AnalyzeEmptyEnum { .. } => "3005",
			Self::AnalyzeEnumTagUsed { .. } => "3006",
			Self::AnalyzeInvalidOptValue { .. } => "3007",
			Self::AnalyzeUnknownOptName { .. } => "3008",
			Self::AnalyzeUnknownTypeRef { .. } => "3009",
			Self::AnalyzeNumOutsideRange { .. } => "3010",
			Self::AnalyzeInvalidOptionalType { .. } => "3011",
			Self::AnalyzeUnboundedRecursiveType { .. } => "3012",
			Self::AnalyzeMissingOptValue { .. } => "3013",
			Self::AnalyzeDuplicateDecl { .. } => "3014",
		}
	}

	fn labels(&self) -> Vec<Label<()>> {
		match self {
			Self::LexerInvalidToken { span } => vec![Label::primary((), span.clone()).with_message("invalid token")],

			Self::ParserUnexpectedEOF { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("unexpected end of file")]
			}

			Self::ParserUnexpectedToken { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("unexpected token")]
			}

			Self::ParserExtraToken { span } => {
				vec![Label::primary((), span.clone()).with_message("extra token")]
			}

			Self::ParserExpectedInt { span } => {
				vec![Label::primary((), span.clone()).with_message("expected integer")]
			}

			Self::AnalyzeEmptyEvDecls => vec![],

			Self::AnalyzeOversizeUnreliable { ev_span, ty_span, .. } => {
				vec![
					Label::primary((), ev_span.clone()).with_message("event is unreliable"),
					Label::secondary((), ty_span.clone()).with_message("type is too large"),
				]
			}

			Self::AnalyzePotentiallyOversizeUnreliable { ev_span, ty_span, .. } => {
				vec![
					Label::primary((), ev_span.clone()).with_message("event is unreliable"),
					Label::secondary((), ty_span.clone()).with_message("type may be too large"),
				]
			}

			Self::AnalyzeInvalidRange { span } => {
				vec![Label::primary((), span.clone()).with_message("invalid range")]
			}

			Self::AnalyzeEmptyEnum { span } => {
				vec![Label::primary((), span.clone()).with_message("empty enum")]
			}

			Self::AnalyzeEnumTagUsed {
				tag_span, used_span, ..
			} => {
				vec![
					Label::primary((), tag_span.clone()).with_message("enum tag"),
					Label::secondary((), used_span.clone()).with_message("used in variant"),
				]
			}

			Self::AnalyzeInvalidOptValue { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("invalid opt value")]
			}

			Self::AnalyzeUnknownOptName { span } => {
				vec![Label::primary((), span.clone()).with_message("unknown opt name")]
			}

			Self::AnalyzeUnknownTypeRef { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("unknown type reference")]
			}

			Self::AnalyzeNumOutsideRange { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("number outside range")]
			}

			Self::AnalyzeInvalidOptionalType { span, .. } => {
				vec![Label::primary((), span.clone()).with_message("must be removed")]
			}

			Self::AnalyzeUnboundedRecursiveType {
				decl_span, use_span, ..
			} => {
				vec![
					Label::secondary((), decl_span.clone()).with_message("declared here"),
					Label::primary((), use_span.clone()).with_message("used recursively here"),
				]
			}

			Self::AnalyzeMissingOptValue { .. } => vec![],

			Self::AnalyzeDuplicateDecl {
				prev_span, dup_span, ..
			} => {
				vec![
					Label::secondary((), prev_span.clone()).with_message("previous declaration"),
					Label::primary((), dup_span.clone()).with_message("duplicate declaration"),
				]
			}
		}
	}

	fn notes(&self) -> Option<Vec<String>> {
		match self {
			Self::LexerInvalidToken { .. } => None,

			Self::ParserUnexpectedEOF { .. } => None,
			Self::ParserUnexpectedToken { .. } => None,
			Self::ParserExtraToken { .. } => None,
			Self::ParserExpectedInt { .. } => None,

			Self::AnalyzeEmptyEvDecls => Some(vec![
				"add an event or function declaration to allow zap to output code".to_string()
			]),
			Self::AnalyzeOversizeUnreliable { max_size, .. } => Some(vec![
				format!("all unreliable events must be under {max_size} bytes in size"),
				"consider adding a upper limit to any arrays or strings".to_string(),
				"upper limits can be added for arrays by doing `[..10]`".to_string(),
				"upper limits can be added for strings by doing `(..10)`".to_string(),
			]),
			Self::AnalyzePotentiallyOversizeUnreliable { max_size, .. } => Some(vec![
				format!("all unreliable events must be under {max_size} bytes in size"),
				"consider adding a upper limit to any arrays or strings".to_string(),
				"upper limits can be added for arrays by doing `[..10]`".to_string(),
				"upper limits can be added for strings by doing `(..10)`".to_string(),
			]),
			Self::AnalyzeInvalidRange { .. } => Some(vec![
				"ranges must be in the form `min..max`".to_string(),
				"ranges can be invalid if `min` is greater than `max`".to_string(),
			]),
			Self::AnalyzeEmptyEnum { .. } => Some(vec![
				"enums cannot be empty".to_string(),
				"if you're looking to create an empty type, use a struct with no fields".to_string(),
				"a struct with no fields can be created by doing `struct {}`".to_string(),
			]),
			Self::AnalyzeEnumTagUsed { .. } => Some(vec![
				"tagged enums use the tag field in passed structs to determine what variant to use".to_string(),
				"you cannot override this tag field in a variant as that would break this behavior".to_string(),
			]),
			Self::AnalyzeInvalidOptValue { .. } => None,
			Self::AnalyzeUnknownOptName { .. } => None,
			Self::AnalyzeUnknownTypeRef { .. } => None,
			Self::AnalyzeNumOutsideRange { min, max, .. } => Some(vec![
				format!("(inclusive) min: {}", min),
				format!("(inclusive) max: {}", max),
			]),
			Self::AnalyzeInvalidOptionalType { .. } => Some(vec![
				"you cannot have 'double optional' types, where a type is optional twice".to_string(),
				"maps cannot have optional keys or values, as those are impossible to represent in luau".to_string(),
				"additionally the `unknown` type cannot be optional".to_string(),
			]),
			Self::AnalyzeUnboundedRecursiveType { .. } => Some(vec![
				"this is an unbounded recursive type".to_string(),
				"unbounded recursive types cause infinite loops".to_string(),
			]),
			Self::AnalyzeMissingOptValue {
				expected,
				required_when,
			} => Some(vec![format!(
				"the {expected} option should not be empty if {required_when}"
			)]),
			Self::AnalyzeDuplicateDecl { .. } => None,
		}
	}

	pub fn to_diagnostic(&self, no_warnings: bool) -> Diagnostic<()> {
		let severity = if no_warnings { Severity::Error } else { self.severity() };

		let diagnostic = Diagnostic::new(severity)
			.with_code(self.code())
			.with_message(self.message())
			.with_labels(self.labels());

		if let Some(notes) = self.notes() {
			diagnostic.with_notes(notes)
		} else {
			diagnostic
		}
	}
}
