use proc_macro2::Span;

/// Tracks whether a resolved choice came from projection defaults or an
/// explicit user rule.
#[derive(Clone, Copy)]
pub enum RuleOrigin {
	/// Choice derived from the projection's initial selection mode.
	Implicit,
	/// Choice explicitly requested at the recorded source span.
	Explicit(Span),
}

impl RuleOrigin {
	/// Returns the user-written span for explicit choices, allowing later rules
	/// to report both sides of a duplicate or conflict.
	pub fn explicit_span(self) -> Option<Span> {
		match self {
			Self::Implicit => None,
			Self::Explicit(span) => Some(span),
		}
	}
}
