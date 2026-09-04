//! P2 product configuration and pure backend-selection policy.

use std::{fmt, num::NonZeroUsize};

use crate::{
    CapabilityRequirement, CapabilityState, ConfigError, KeyboardPlan, P1Config, PlanCapabilities,
    PlanError, SemanticElementCount, SemanticElementLimit, SensitiveText, TabPolicy, TextAtom,
    build_keyboard_plan,
};

/// User-visible injection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InjectionMode {
    Keyboard,
    Clipboard,
    #[default]
    Auto,
    /// Use paced keyboard actions with code-aware indentation and pair rules.
    Code,
}

/// Backend selected immutably for one session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionBackend {
    Keyboard,
    Clipboard,
    Code,
}

/// Non-zero semantic-element threshold at which `auto` prefers clipboard paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AutoClipboardThreshold(NonZeroUsize);

impl AutoClipboardThreshold {
    pub fn new(value: usize) -> Result<Self, ProductConfigError> {
        NonZeroUsize::new(value)
            .map(Self)
            .ok_or(ProductConfigError::ZeroAutoClipboardThreshold)
    }

    pub const fn get(self) -> usize {
        self.0.get()
    }
}

impl TryFrom<usize> for AutoClipboardThreshold {
    type Error = ProductConfigError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Validated P2 product configuration. Active sessions retain a copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductConfig {
    pub enabled: bool,
    pub mode: InjectionMode,
    pub auto_clipboard_threshold: AutoClipboardThreshold,
    pub jitter_percent: u8,
    pub typo_probability_percent: u8,
    pub safety: P1Config,
}

impl ProductConfig {
    pub fn validate(self) -> Result<Self, ProductConfigError> {
        self.safety
            .validate()
            .map_err(ProductConfigError::InvalidSafety)?;
        if self.jitter_percent > crate::MAX_JITTER_PERCENT {
            return Err(ProductConfigError::JitterOutOfRange);
        }
        if self.typo_probability_percent > crate::MAX_TYPO_PROBABILITY_PERCENT {
            return Err(ProductConfigError::TypoProbabilityOutOfRange);
        }
        Ok(self)
    }

    pub fn keyboard_only(safety: P1Config) -> Result<Self, ProductConfigError> {
        Self {
            enabled: true,
            mode: InjectionMode::Keyboard,
            auto_clipboard_threshold: AutoClipboardThreshold::new(256)?,
            jitter_percent: 0,
            typo_probability_percent: 0,
            safety,
        }
        .validate()
    }
}

impl Default for ProductConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: InjectionMode::Auto,
            auto_clipboard_threshold: AutoClipboardThreshold::new(256)
                .expect("P2 auto clipboard threshold is non-zero"),
            jitter_percent: 0,
            typo_probability_percent: 0,
            safety: P1Config::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductConfigError {
    ZeroAutoClipboardThreshold,
    JitterOutOfRange,
    TypoProbabilityOutOfRange,
    InvalidSafety(ConfigError),
}

impl fmt::Display for ProductConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroAutoClipboardThreshold => {
                formatter.write_str("auto clipboard threshold must be non-zero")
            }
            Self::JitterOutOfRange => formatter.write_str("jitter percent is out of range"),
            Self::TypoProbabilityOutOfRange => {
                formatter.write_str("typo probability percent is out of range")
            }
            Self::InvalidSafety(error) => {
                write!(formatter, "invalid safety configuration: {error}")
            }
        }
    }
}

impl std::error::Error for ProductConfigError {}

/// Native-neutral capability snapshot consumed by P2 pure planning policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductCapabilities {
    pub keyboard: PlanCapabilities,
    pub clipboard_paste: CapabilityState,
    pub clipboard_revision_guard: CapabilityState,
}

/// Content-free clipboard plan. The source remains the current OS clipboard.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ClipboardPlan {
    elements: SemanticElementCount,
}

impl ClipboardPlan {
    pub const fn element_count(self) -> SemanticElementCount {
        self.elements
    }
}

impl fmt::Debug for ClipboardPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClipboardPlan")
            .field("elements", &self.elements.get())
            .field("content", &"[REDACTED]")
            .finish()
    }
}

/// One keyboard action in Code mode.
///
/// `CursorRight` passes over a closing delimiter or quote that the destination
/// editor generated after the corresponding opener. The planner only reasons
/// about the source text; it never reads destination content. Triple-quoted
/// delimiters are emitted as ordinary atoms because editors do not reliably
/// synthesize a skippable three-character closing delimiter. Markdown
/// triple-backtick fences are also emitted as literal atoms and do not enter
/// quote state.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CodeAction {
    Atom(TextAtom),
    CursorRight,
    /// Consumes a matching closer that the editor placed on its own generated
    /// line. The platform first crosses the existing line boundary with
    /// CursorRight, then uses its right-arrow line-end chord to pass both the
    /// editor's indentation and closer without reading destination text.
    CursorRightToLineEnd,
}

impl fmt::Debug for CodeAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atom(atom) => formatter.debug_tuple("Atom").field(atom).finish(),
            Self::CursorRight => formatter.write_str("CursorRight"),
            Self::CursorRightToLineEnd => formatter.write_str("CursorRightToLineEnd"),
        }
    }
}

/// Immutable keyboard plan for source code and structured text.
pub struct CodePlan {
    actions: Vec<CodeAction>,
    elements: SemanticElementCount,
    config: P1Config,
    capabilities: PlanCapabilities,
}

impl CodePlan {
    pub fn actions(&self) -> &[CodeAction] {
        &self.actions
    }

    pub const fn element_count(&self) -> SemanticElementCount {
        self.elements
    }

    pub const fn config(&self) -> P1Config {
        self.config
    }

    pub const fn capabilities(&self) -> PlanCapabilities {
        self.capabilities
    }
}

impl fmt::Debug for CodePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodePlan")
            .field("elements", &self.elements.get())
            .field("actions", &self.actions.len())
            .field("config", &self.config)
            .field("capabilities", &self.capabilities)
            .field("content", &"[REDACTED]")
            .finish()
    }
}

/// Immutable backend-specific plan for one P2 session.
pub enum InjectionPlan {
    Keyboard(KeyboardPlan),
    Clipboard(ClipboardPlan),
    Code(CodePlan),
}

impl InjectionPlan {
    pub const fn backend(&self) -> InjectionBackend {
        match self {
            Self::Keyboard(_) => InjectionBackend::Keyboard,
            Self::Clipboard(_) => InjectionBackend::Clipboard,
            Self::Code(_) => InjectionBackend::Code,
        }
    }

    pub fn element_count(&self) -> SemanticElementCount {
        match self {
            Self::Keyboard(plan) => plan.text().element_count(),
            Self::Clipboard(plan) => plan.element_count(),
            Self::Code(plan) => plan.element_count(),
        }
    }
}

impl fmt::Debug for InjectionPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyboard(plan) => formatter.debug_tuple("Keyboard").field(plan).finish(),
            Self::Clipboard(plan) => formatter.debug_tuple("Clipboard").field(plan).finish(),
            Self::Code(plan) => formatter.debug_tuple("Code").field(plan).finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductPlanError {
    Disabled,
    InvalidConfiguration(ProductConfigError),
    Empty,
    PayloadTooLarge { limit: SemanticElementLimit },
    Keyboard(PlanError),
    ClipboardCapabilityUnavailable,
    ClipboardCapabilityDegraded,
    ClipboardRevisionUnavailable,
}

impl fmt::Display for ProductPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled => formatter.write_str("ClipType is disabled"),
            Self::InvalidConfiguration(error) => {
                write!(formatter, "invalid configuration: {error}")
            }
            Self::Empty => formatter.write_str("clipboard text is empty"),
            Self::PayloadTooLarge { limit } => {
                write!(formatter, "semantic payload exceeds limit {}", limit.get())
            }
            Self::Keyboard(error) => write!(formatter, "keyboard plan unavailable: {error}"),
            Self::ClipboardCapabilityUnavailable => {
                formatter.write_str("clipboard paste capability unavailable")
            }
            Self::ClipboardCapabilityDegraded => {
                formatter.write_str("degraded clipboard paste capability rejected")
            }
            Self::ClipboardRevisionUnavailable => {
                formatter.write_str("clipboard revision evidence unavailable")
            }
        }
    }
}

impl std::error::Error for ProductPlanError {}

/// Selects exactly one backend and creates an immutable plan.
///
/// Explicit modes never fall back. `Auto` may select clipboard paste only when
/// both paste dispatch and revision guarding are fully available for the
/// current snapshot.
pub fn build_injection_plan(
    text: SensitiveText,
    revision_available: bool,
    config: ProductConfig,
    capabilities: ProductCapabilities,
) -> Result<InjectionPlan, ProductPlanError> {
    let config = config
        .validate()
        .map_err(ProductPlanError::InvalidConfiguration)?;
    if !config.enabled {
        return Err(ProductPlanError::Disabled);
    }

    let elements = inspect_elements(&text, config.safety.total_payload_limit)?;
    match config.mode {
        InjectionMode::Keyboard => build_keyboard(text, config, capabilities),
        InjectionMode::Clipboard => {
            require_clipboard(capabilities, revision_available)?;
            Ok(InjectionPlan::Clipboard(ClipboardPlan { elements }))
        }
        InjectionMode::Code => build_code(text, config, capabilities, elements),
        InjectionMode::Auto => {
            let clipboard_available = clipboard_is_available(capabilities, revision_available);
            let keyboard_available =
                keyboard_is_available(&text, config.safety, capabilities.keyboard);
            // Text outside ASCII is safer through the already-current,
            // revision-guarded paste path. This covers CJK, emoji, combining
            // marks, and mixed Unicode text even when the payload is short.
            let prefers_clipboard = !text.expose().is_ascii();

            if clipboard_available
                && (prefers_clipboard
                    || elements.get() >= config.auto_clipboard_threshold.get()
                    || !keyboard_available)
            {
                Ok(InjectionPlan::Clipboard(ClipboardPlan { elements }))
            } else {
                build_keyboard(text, config, capabilities)
            }
        }
    }
}

fn build_keyboard(
    text: SensitiveText,
    config: ProductConfig,
    capabilities: ProductCapabilities,
) -> Result<InjectionPlan, ProductPlanError> {
    build_keyboard_plan(text, config.safety, capabilities.keyboard)
        .map(InjectionPlan::Keyboard)
        .map_err(ProductPlanError::Keyboard)
}

fn build_code(
    text: SensitiveText,
    config: ProductConfig,
    capabilities: ProductCapabilities,
    elements: SemanticElementCount,
) -> Result<InjectionPlan, ProductPlanError> {
    let keyboard = build_keyboard_plan(text, config.safety, capabilities.keyboard)
        .map_err(ProductPlanError::Keyboard)?;
    let actions = build_code_actions(keyboard.text().atoms());
    require_keyboard_capability(
        capabilities.keyboard.cursor_right,
        CapabilityRequirement::CursorRight,
    )?;

    Ok(InjectionPlan::Code(CodePlan {
        actions,
        elements,
        config: keyboard.config(),
        capabilities: keyboard.capabilities(),
    }))
}

fn require_keyboard_capability(
    state: CapabilityState,
    requirement: CapabilityRequirement,
) -> Result<(), ProductPlanError> {
    match state {
        CapabilityState::Available => Ok(()),
        CapabilityState::Degraded => Err(ProductPlanError::Keyboard(
            PlanError::CapabilityDegraded(requirement),
        )),
        CapabilityState::Unavailable => Err(ProductPlanError::Keyboard(
            PlanError::CapabilityUnavailable(requirement),
        )),
    }
}

#[derive(Clone, Copy)]
enum CodeLexState {
    Normal,
    LineComment,
    BlockComment { closing: bool },
    String { quote: QuoteKind, escaped: bool },
}

fn build_code_actions(atoms: &[TextAtom]) -> Vec<CodeAction> {
    let mut actions = Vec::with_capacity(atoms.len());
    let mut state = CodeLexState::Normal;
    let mut pair_stack = Vec::with_capacity(atoms.len().min(32));
    let mut line_start = true;
    let mut index = 0;

    while index < atoms.len() {
        let atom = atoms[index];
        let next = atoms.get(index.saturating_add(1)).copied();

        match state {
            CodeLexState::Normal => {
                if line_start && is_code_indentation(atom) {
                    index = index.saturating_add(1);
                    continue;
                }

                match atom {
                    TextAtom::LineBreak => {
                        if let Some(line_closers) =
                            line_leading_matching_closers(atoms, index, &pair_stack)
                        {
                            // The first Return inside an empty pair is still
                            // needed to ask the editor for its indented body
                            // line. Once a prior Return has already moved the
                            // generated closer to a following line, emitting
                            // the source Return here would create an extra
                            // blank line and leave CursorRight short of it.
                            if !line_closers.were_line_separated {
                                push_code_line_break(&mut actions, &mut pair_stack);
                            }
                            pair_stack
                                .truncate(pair_stack.len().saturating_sub(line_closers.pair_count));
                            actions.push(CodeAction::CursorRightToLineEnd);
                            line_start = false;
                            index = line_closers.end_index;
                            continue;
                        }

                        push_code_line_break(&mut actions, &mut pair_stack);
                        line_start = true;
                    }
                    TextAtom::Tab => {
                        actions.push(CodeAction::Atom(atom));
                        line_start = false;
                    }
                    TextAtom::Scalar(value) => {
                        if value == '/' && next == Some(TextAtom::Scalar('/')) {
                            actions.push(CodeAction::Atom(atom));
                            state = CodeLexState::LineComment;
                            line_start = false;
                        } else if value == '/' && next == Some(TextAtom::Scalar('*')) {
                            actions.push(CodeAction::Atom(atom));
                            state = CodeLexState::BlockComment { closing: false };
                            line_start = false;
                        } else if let Some(fence_length) =
                            markdown_fence_length(atoms, index, value)
                        {
                            // Markdown fences are delimiters, not backtick
                            // strings. Keep their fence atoms literal so code
                            // inside the fence remains pair-aware.
                            push_scalar_run(&mut actions, value, fence_length);
                            line_start = false;
                            index = index.saturating_add(fence_length);
                            continue;
                        } else if let Some(quote) = quote_kind_at(atoms, index, value) {
                            push_quote_run(&mut actions, quote);
                            pair_stack.push(Pair::Quote(quote));
                            state = CodeLexState::String {
                                quote,
                                escaped: false,
                            };
                            line_start = false;
                            index = index.saturating_add(quote.length());
                            continue;
                        } else if let Some(expected) = opening_pair(value) {
                            actions.push(CodeAction::Atom(atom));
                            pair_stack.push(Pair::Bracket {
                                closer: expected,
                                line_separated: false,
                            });
                            line_start = false;
                        } else if pair_stack
                            .last()
                            .is_some_and(|pair| pair.matches_bracket(value))
                        {
                            pair_stack.pop();
                            actions.push(CodeAction::CursorRight);
                            line_start = false;
                        } else {
                            actions.push(CodeAction::Atom(atom));
                            line_start = false;
                        }
                    }
                }
            }
            CodeLexState::LineComment => {
                if matches!(atom, TextAtom::LineBreak) {
                    push_code_line_break(&mut actions, &mut pair_stack);
                    state = CodeLexState::Normal;
                    line_start = true;
                } else {
                    actions.push(CodeAction::Atom(atom));
                }
            }
            CodeLexState::BlockComment { closing } => match atom {
                TextAtom::Scalar(value) if closing && value == '/' => {
                    actions.push(CodeAction::Atom(atom));
                    state = CodeLexState::Normal;
                    line_start = false;
                }
                TextAtom::Scalar(value)
                    if !closing && value == '*' && next == Some(TextAtom::Scalar('/')) =>
                {
                    actions.push(CodeAction::Atom(atom));
                    state = CodeLexState::BlockComment { closing: true };
                    line_start = false;
                }
                TextAtom::LineBreak => {
                    push_code_line_break(&mut actions, &mut pair_stack);
                    line_start = true;
                }
                _ => {
                    actions.push(CodeAction::Atom(atom));
                    line_start = false;
                }
            },
            CodeLexState::String { quote, escaped } => match atom {
                TextAtom::Scalar(_) if escaped => {
                    actions.push(CodeAction::Atom(atom));
                    state = CodeLexState::String {
                        quote,
                        escaped: false,
                    };
                    line_start = false;
                }
                TextAtom::Scalar('\\') => {
                    actions.push(CodeAction::Atom(atom));
                    state = CodeLexState::String {
                        quote,
                        escaped: true,
                    };
                    line_start = false;
                }
                TextAtom::Scalar(value)
                    if quote.is_triple()
                        && value == quote.delimiter()
                        && has_scalar_run(atoms, index, value, quote.length()) =>
                {
                    // Triple-quoted strings are not treated as an editor-generated
                    // pair. Type both boundaries and the body explicitly, then
                    // resume normal pair handling after the closing run.
                    push_quote_run(&mut actions, quote);
                    if pair_stack.last() == Some(&Pair::Quote(quote)) {
                        pair_stack.pop();
                    }
                    state = CodeLexState::Normal;
                    line_start = false;
                    index = index.saturating_add(quote.length());
                    continue;
                }
                TextAtom::Scalar(value) if !quote.is_triple() && opening_pair(value).is_some() => {
                    // Some editors also auto-complete brackets typed inside a
                    // string. Keep those generated closers in the same logical
                    // stack so a later source closer or string boundary can
                    // consume them with CursorRight.
                    actions.push(CodeAction::Atom(atom));
                    pair_stack.push(Pair::Bracket {
                        closer: opening_pair(value).expect("checked above"),
                        line_separated: false,
                    });
                    line_start = false;
                }
                TextAtom::Scalar(value)
                    if !quote.is_triple()
                        && pair_stack
                            .last()
                            .is_some_and(|pair| pair.matches_bracket(value)) =>
                {
                    pair_stack.pop();
                    actions.push(CodeAction::CursorRight);
                    line_start = false;
                }
                TextAtom::Scalar(value) if !quote.is_triple() && value == quote.delimiter() => {
                    // A bracket such as the `{` in `"{"` can be auto-completed
                    // before the editor's generated quote. Move over all such
                    // generated closers first, then move over the quote.
                    flush_string_bracket_pairs(&mut actions, &mut pair_stack);
                    if pair_stack.last() == Some(&Pair::Quote(quote)) {
                        pair_stack.pop();
                        actions.push(CodeAction::CursorRight);
                    } else {
                        actions.push(CodeAction::Atom(atom));
                    }
                    state = CodeLexState::Normal;
                    line_start = false;
                }
                TextAtom::LineBreak => {
                    push_code_line_break(&mut actions, &mut pair_stack);
                    line_start = true;
                }
                _ => {
                    actions.push(CodeAction::Atom(atom));
                    line_start = false;
                }
            },
        }

        index = index.saturating_add(1);
    }

    actions
}

fn flush_string_bracket_pairs(actions: &mut Vec<CodeAction>, pair_stack: &mut Vec<Pair>) {
    while matches!(pair_stack.last(), Some(Pair::Bracket { .. })) {
        pair_stack.pop();
        actions.push(CodeAction::CursorRight);
    }
}

fn push_code_line_break(actions: &mut Vec<CodeAction>, pair_stack: &mut [Pair]) {
    actions.push(CodeAction::Atom(TextAtom::LineBreak));
    for pair in pair_stack {
        pair.mark_line_separated();
    }
}

struct LineClosers {
    end_index: usize,
    pair_count: usize,
    were_line_separated: bool,
}

fn line_leading_matching_closers(
    atoms: &[TextAtom],
    line_break_index: usize,
    pair_stack: &[Pair],
) -> Option<LineClosers> {
    let (_, were_line_separated) = pair_stack.last()?.bracket()?;
    let mut index = line_break_index.saturating_add(1);

    while atoms.get(index).copied().is_some_and(is_code_indentation) {
        index = index.saturating_add(1);
    }

    let mut pair_count = 0;
    while pair_count < pair_stack.len() {
        let pair = &pair_stack[pair_stack.len() - pair_count - 1];
        let Some(TextAtom::Scalar(value)) = atoms.get(index).copied() else {
            break;
        };
        if !pair.matches_bracket(value) {
            break;
        }
        pair_count = pair_count.saturating_add(1);
        index = index.saturating_add(1);
    }

    (pair_count > 0).then_some(LineClosers {
        end_index: index,
        pair_count,
        were_line_separated,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pair {
    Bracket { closer: char, line_separated: bool },
    Quote(QuoteKind),
}

impl Pair {
    const fn matches_bracket(self, value: char) -> bool {
        matches!(self, Self::Bracket { closer, .. } if closer == value)
    }

    const fn bracket(self) -> Option<(char, bool)> {
        match self {
            Self::Bracket {
                closer,
                line_separated,
            } => Some((closer, line_separated)),
            Self::Quote(_) => None,
        }
    }

    fn mark_line_separated(&mut self) {
        if let Self::Bracket { line_separated, .. } = self {
            *line_separated = true;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum QuoteKind {
    Single(char),
    Triple(char),
}

impl QuoteKind {
    const fn delimiter(self) -> char {
        match self {
            Self::Single(delimiter) | Self::Triple(delimiter) => delimiter,
        }
    }

    const fn length(self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Triple(_) => 3,
        }
    }

    const fn is_triple(self) -> bool {
        matches!(self, Self::Triple(_))
    }
}

const fn is_code_indentation(atom: TextAtom) -> bool {
    matches!(atom, TextAtom::Scalar(' ') | TextAtom::Tab)
}

const fn opening_pair(value: char) -> Option<char> {
    match value {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        _ => None,
    }
}

const fn quote_delimiter(value: char) -> Option<char> {
    match value {
        '\'' | '"' | '`' => Some(value),
        _ => None,
    }
}

fn quote_kind_at(atoms: &[TextAtom], index: usize, value: char) -> Option<QuoteKind> {
    let delimiter = quote_delimiter(value)?;
    if supports_triple_quote(delimiter) && has_scalar_run(atoms, index, delimiter, 3) {
        Some(QuoteKind::Triple(delimiter))
    } else {
        Some(QuoteKind::Single(delimiter))
    }
}

const fn supports_triple_quote(value: char) -> bool {
    matches!(value, '\'' | '"')
}

fn has_scalar_run(atoms: &[TextAtom], start: usize, value: char, length: usize) -> bool {
    atoms
        .get(start..start.saturating_add(length))
        .map(|run| run.iter().all(|atom| *atom == TextAtom::Scalar(value)))
        .unwrap_or(false)
}

fn scalar_run_length(atoms: &[TextAtom], start: usize, value: char) -> usize {
    atoms
        .get(start..)
        .map(|run| {
            run.iter()
                .take_while(|atom| **atom == TextAtom::Scalar(value))
                .count()
        })
        .unwrap_or(0)
}

fn markdown_fence_length(atoms: &[TextAtom], start: usize, value: char) -> Option<usize> {
    if value != '`' {
        return None;
    }

    let length = scalar_run_length(atoms, start, value);
    (length >= 3).then_some(length)
}

fn push_quote_run(actions: &mut Vec<CodeAction>, quote: QuoteKind) {
    push_scalar_run(actions, quote.delimiter(), quote.length());
}

fn push_scalar_run(actions: &mut Vec<CodeAction>, value: char, length: usize) {
    for _ in 0..length {
        actions.push(CodeAction::Atom(TextAtom::Scalar(value)));
    }
}

fn require_clipboard(
    capabilities: ProductCapabilities,
    revision_available: bool,
) -> Result<(), ProductPlanError> {
    match capabilities.clipboard_paste {
        CapabilityState::Unavailable => {
            return Err(ProductPlanError::ClipboardCapabilityUnavailable);
        }
        CapabilityState::Degraded => {
            return Err(ProductPlanError::ClipboardCapabilityDegraded);
        }
        CapabilityState::Available => {}
    }
    match capabilities.clipboard_revision_guard {
        CapabilityState::Unavailable => {
            return Err(ProductPlanError::ClipboardRevisionUnavailable);
        }
        CapabilityState::Degraded => {
            return Err(ProductPlanError::ClipboardCapabilityDegraded);
        }
        CapabilityState::Available => {}
    }
    if !revision_available {
        return Err(ProductPlanError::ClipboardRevisionUnavailable);
    }
    Ok(())
}

fn clipboard_is_available(capabilities: ProductCapabilities, revision_available: bool) -> bool {
    revision_available
        && capabilities.clipboard_paste == CapabilityState::Available
        && capabilities.clipboard_revision_guard == CapabilityState::Available
}

fn keyboard_is_available(
    text: &SensitiveText,
    config: P1Config,
    capabilities: PlanCapabilities,
) -> bool {
    let mut contains_scalar = false;
    let mut contains_line_break = false;
    let mut contains_tab = false;
    let mut chars = text.expose().chars().peekable();

    while let Some(value) = chars.next() {
        match value {
            '\r' => {
                let _ = chars.next_if_eq(&'\n');
                contains_line_break = true;
            }
            '\n' => contains_line_break = true,
            '\t' => {
                if config.tab_policy == TabPolicy::Reject {
                    return false;
                }
                contains_tab = true;
            }
            control if control.is_control() => return false,
            _ => contains_scalar = true,
        }
    }

    (!contains_scalar || capabilities.unicode_text == CapabilityState::Available)
        && (!contains_line_break || capabilities.line_break == CapabilityState::Available)
        && (!contains_tab || capabilities.tab == CapabilityState::Available)
        && capabilities.modifier_observation == CapabilityState::Available
}

fn inspect_elements(
    text: &SensitiveText,
    limit: SemanticElementLimit,
) -> Result<SemanticElementCount, ProductPlanError> {
    if text.is_empty() {
        return Err(ProductPlanError::Empty);
    }

    let mut count = 0_usize;
    let mut chars = text.expose().chars().peekable();
    while let Some(value) = chars.next() {
        if value == '\r' {
            let _ = chars.next_if_eq(&'\n');
        }
        if count >= limit.get() {
            return Err(ProductPlanError::PayloadTooLarge { limit });
        }
        count = count.saturating_add(1);
    }

    Ok(SemanticElementCount::new(count))
}

#[cfg(test)]
mod tests {
    use super::{
        AutoClipboardThreshold, CapabilityRequirement, InjectionBackend, InjectionMode,
        InjectionPlan, ProductCapabilities, ProductConfig, ProductPlanError, build_injection_plan,
    };
    use crate::{CapabilityState, P1Config, PlanCapabilities, PlanError, SensitiveText};

    fn capabilities() -> ProductCapabilities {
        ProductCapabilities {
            keyboard: PlanCapabilities {
                unicode_text: CapabilityState::Available,
                line_break: CapabilityState::Available,
                tab: CapabilityState::Available,
                cursor_right: CapabilityState::Available,
                modifier_observation: CapabilityState::Available,
            },
            clipboard_paste: CapabilityState::Available,
            clipboard_revision_guard: CapabilityState::Available,
        }
    }

    fn config(mode: InjectionMode, threshold: usize) -> ProductConfig {
        ProductConfig {
            mode,
            auto_clipboard_threshold: AutoClipboardThreshold::new(threshold)
                .expect("test threshold"),
            ..ProductConfig::default()
        }
    }

    #[test]
    fn explicit_modes_never_fall_back() {
        let unavailable_clipboard = ProductCapabilities {
            clipboard_paste: CapabilityState::Unavailable,
            ..capabilities()
        };
        assert!(matches!(
            build_injection_plan(
                SensitiveText::new("hello".to_owned()),
                true,
                config(InjectionMode::Clipboard, 8),
                unavailable_clipboard,
            ),
            Err(ProductPlanError::ClipboardCapabilityUnavailable)
        ));

        let unavailable_keyboard = ProductCapabilities {
            keyboard: PlanCapabilities {
                unicode_text: CapabilityState::Unavailable,
                ..capabilities().keyboard
            },
            ..capabilities()
        };
        assert!(matches!(
            build_injection_plan(
                SensitiveText::new("hello".to_owned()),
                true,
                config(InjectionMode::Keyboard, 8),
                unavailable_keyboard,
            ),
            Err(ProductPlanError::Keyboard(_))
        ));
    }

    #[test]
    fn code_mode_uses_keyboard_code_actions_for_indentation_and_pairs() {
        let plan = build_injection_plan(
            SensitiveText::new("if (items[0] == '{') {\n  \treturn {};\n}".to_owned()),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("Code mode needs the keyboard code capabilities");

        assert_eq!(plan.backend(), InjectionBackend::Code);
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };
        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::CursorRight))
                .count(),
            5
        );
        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::CursorRightToLineEnd))
                .count(),
            1
        );
        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::Atom(crate::TextAtom::Tab)))
                .count(),
            0
        );
    }

    #[test]
    fn code_mode_consumes_string_brackets_before_quote_and_paren_boundaries() {
        let plan = build_injection_plan(
            SensitiveText::new(r#"if (value[0] == "{") {"#.to_owned()),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("Code mode needs the keyboard code capabilities");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };

        let first_quote = plan
            .actions()
            .iter()
            .position(|action| *action == super::CodeAction::Atom(crate::TextAtom::Scalar('"')))
            .expect("string opener");
        assert_eq!(
            &plan.actions()[first_quote..first_quote.saturating_add(7)],
            &[
                super::CodeAction::Atom(crate::TextAtom::Scalar('"')),
                super::CodeAction::Atom(crate::TextAtom::Scalar('{')),
                super::CodeAction::CursorRight,
                super::CodeAction::CursorRight,
                super::CodeAction::CursorRight,
                super::CodeAction::Atom(crate::TextAtom::Scalar(' ')),
                super::CodeAction::Atom(crate::TextAtom::Scalar('{')),
            ]
        );
    }

    #[test]
    fn code_mode_navigates_to_line_leading_generated_closers() {
        let plan = build_injection_plan(
            SensitiveText::new(
                "fn main() {\nlet value = {\"x\": [1, 2]};\nif (value[0] == \"{\") {\nprintln!(\"value = {}\", value);\n}\n}\nNEXT"
                    .to_owned(),
            ),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("line-leading generated closers are supported");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };

        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::CursorRightToLineEnd))
                .count(),
            2
        );
        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(
                    action,
                    super::CodeAction::Atom(crate::TextAtom::LineBreak)
                ))
                .count(),
            4
        );
        assert_eq!(
            plan.actions().last(),
            Some(&super::CodeAction::Atom(crate::TextAtom::Scalar('T')))
        );
    }

    #[test]
    fn code_mode_keeps_first_line_break_for_an_empty_multiline_pair() {
        let plan = build_injection_plan(
            SensitiveText::new("fn main() {\n}\nNEXT".to_owned()),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("empty multiline pair is supported");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };
        let navigation = plan
            .actions()
            .iter()
            .position(|action| *action == super::CodeAction::CursorRightToLineEnd)
            .expect("line closer navigation");

        assert_eq!(
            plan.actions().get(navigation.saturating_sub(1)),
            Some(&super::CodeAction::Atom(crate::TextAtom::LineBreak))
        );
        assert_eq!(
            plan.actions().get(navigation.saturating_add(1)),
            Some(&super::CodeAction::Atom(crate::TextAtom::LineBreak))
        );
    }

    #[test]
    fn code_mode_consumes_grouped_closers_on_one_generated_line() {
        let plan = build_injection_plan(
            SensitiveText::new("call({\nvalue();\n});\nNEXT".to_owned()),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("grouped line closers are supported");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };

        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::CursorRightToLineEnd))
                .count(),
            1
        );
        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::CursorRight))
                .count(),
            1
        );
    }

    #[test]
    fn code_mode_types_triple_quote_delimiters_explicitly() {
        for source in [
            "const doc = \"\"\"hello\"\"\";",
            "const doc = \"\"\"it's fine\"\"\";",
            "const doc = '''hello''';",
            "const doc = '''it's fine''';",
        ] {
            let plan = build_injection_plan(
                SensitiveText::new(source.to_owned()),
                true,
                config(InjectionMode::Code, 256),
                capabilities(),
            )
            .expect("triple-quoted source is supported");
            let InjectionPlan::Code(plan) = plan else {
                panic!("Code mode must produce a Code plan");
            };

            let expected: Vec<_> = source
                .chars()
                .map(|value| super::CodeAction::Atom(crate::TextAtom::Scalar(value)))
                .collect();
            assert_eq!(plan.actions(), expected.as_slice());
        }
    }

    #[test]
    fn code_mode_keeps_triple_quoted_body_indentation() {
        let source = "const doc = \"\"\"\n  body\n\"\"\";";
        let plan = build_injection_plan(
            SensitiveText::new(source.to_owned()),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("multiline triple-quoted source is supported");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };

        let expected: Vec<_> = source
            .chars()
            .map(|value| match value {
                '\n' => super::CodeAction::Atom(crate::TextAtom::LineBreak),
                '\t' => super::CodeAction::Atom(crate::TextAtom::Tab),
                value => super::CodeAction::Atom(crate::TextAtom::Scalar(value)),
            })
            .collect();
        assert_eq!(plan.actions(), expected.as_slice());
    }

    #[test]
    fn code_mode_keeps_markdown_fences_literal_and_parses_code_inside() {
        let plan = build_injection_plan(
            SensitiveText::new("```cpp\nif (x) {\n    return;\n}\n```".to_owned()),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("fenced source is supported");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };

        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(
                    action,
                    super::CodeAction::Atom(crate::TextAtom::Scalar('`'))
                ))
                .count(),
            6
        );
        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::CursorRight))
                .count(),
            1
        );
        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::CursorRightToLineEnd))
                .count(),
            1
        );
    }

    #[test]
    fn code_mode_does_not_require_clipboard_paste_or_revision() {
        let keyboard_only = ProductCapabilities {
            clipboard_paste: CapabilityState::Unavailable,
            clipboard_revision_guard: CapabilityState::Unavailable,
            ..capabilities()
        };
        let plan = build_injection_plan(
            SensitiveText::new("fn main() {}".to_owned()),
            false,
            config(InjectionMode::Code, 256),
            keyboard_only,
        )
        .expect("Code mode is keyboard-based");

        assert!(matches!(plan, InjectionPlan::Code(_)));
    }

    #[test]
    fn code_mode_requires_cursor_right_capability() {
        let no_cursor_right = ProductCapabilities {
            keyboard: PlanCapabilities {
                cursor_right: CapabilityState::Unavailable,
                ..capabilities().keyboard
            },
            ..capabilities()
        };
        assert!(matches!(
            build_injection_plan(
                SensitiveText::new("fn main() {}".to_owned()),
                true,
                config(InjectionMode::Code, 256),
                no_cursor_right,
            ),
            Err(ProductPlanError::Keyboard(
                PlanError::CapabilityUnavailable(CapabilityRequirement::CursorRight)
            ))
        ));
    }

    #[test]
    fn code_mode_keeps_comment_and_string_delimiters_literal() {
        let plan = build_injection_plan(
            SensitiveText::new(
                "// [not a pair]\n/* { not a pair } */\nlet value = \"[{}]\";\n".to_owned(),
            ),
            true,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("comment and string fixture is supported");
        let InjectionPlan::Code(plan) = plan else {
            panic!("Code mode must produce a Code plan");
        };

        assert_eq!(
            plan.actions()
                .iter()
                .filter(|action| matches!(action, super::CodeAction::CursorRight))
                .count(),
            3
        );
    }

    #[test]
    fn auto_uses_keyboard_below_threshold_and_clipboard_at_threshold() {
        let short = build_injection_plan(
            SensitiveText::new("short".to_owned()),
            true,
            config(InjectionMode::Auto, 8),
            capabilities(),
        )
        .expect("short keyboard plan");
        assert_eq!(short.backend(), InjectionBackend::Keyboard);

        let long = build_injection_plan(
            SensitiveText::new("12345678".to_owned()),
            true,
            config(InjectionMode::Auto, 8),
            capabilities(),
        )
        .expect("threshold clipboard plan");
        assert_eq!(long.backend(), InjectionBackend::Clipboard);
    }

    #[test]
    fn auto_uses_clipboard_when_keyboard_text_is_ineligible() {
        let plan = build_injection_plan(
            SensitiveText::new("a\u{0007}b".to_owned()),
            true,
            config(InjectionMode::Auto, 100),
            capabilities(),
        )
        .expect("clipboard can deliver text without emitting the control as a key");
        assert_eq!(plan.backend(), InjectionBackend::Clipboard);
    }

    #[test]
    fn auto_prefers_revision_guarded_clipboard_for_short_unicode_text() {
        for text in [
            "你好，世界",
            "繁體中文測試",
            "こんにちは世界",
            "안녕하세요",
            "中文与 ASCII 123 混合",
            "😀👍",
            "e\u{0301}",
        ] {
            let plan = build_injection_plan(
                SensitiveText::new(text.to_owned()),
                true,
                config(InjectionMode::Auto, 256),
                capabilities(),
            )
            .expect("short Unicode text should use the guarded paste path");
            assert_eq!(plan.backend(), InjectionBackend::Clipboard, "{text:?}");
        }
    }

    #[test]
    fn auto_can_use_unicode_keyboard_only_when_clipboard_guard_is_unavailable() {
        let capabilities = ProductCapabilities {
            clipboard_paste: CapabilityState::Unavailable,
            clipboard_revision_guard: CapabilityState::Unavailable,
            ..capabilities()
        };
        let plan = build_injection_plan(
            SensitiveText::new("你好".to_owned()),
            false,
            config(InjectionMode::Auto, 256),
            capabilities,
        )
        .expect("Auto retains the Unicode keyboard fallback when paste is unavailable");
        assert_eq!(plan.backend(), InjectionBackend::Keyboard);
    }

    #[test]
    fn clipboard_requires_revision_evidence() {
        assert!(matches!(
            build_injection_plan(
                SensitiveText::new("hello".to_owned()),
                false,
                config(InjectionMode::Clipboard, 8),
                capabilities(),
            ),
            Err(ProductPlanError::ClipboardRevisionUnavailable)
        ));
    }

    #[test]
    fn plan_debug_is_content_free() {
        let marker = "PRODUCT_PLAN_PRIVATE_SENTINEL";
        let plan = build_injection_plan(
            SensitiveText::new(marker.to_owned()),
            true,
            config(InjectionMode::Clipboard, 8),
            capabilities(),
        )
        .expect("clipboard plan");
        let debug = format!("{plan:?}");

        assert!(!debug.contains(marker));
        assert!(matches!(plan, InjectionPlan::Clipboard(_)));

        let code = build_injection_plan(
            SensitiveText::new(marker.to_owned()),
            false,
            config(InjectionMode::Code, 256),
            capabilities(),
        )
        .expect("code plan");
        let debug = format!("{code:?}");
        assert!(!debug.contains(marker));
        assert!(matches!(code, InjectionPlan::Code(_)));
    }

    #[test]
    fn threshold_must_be_non_zero() {
        assert!(AutoClipboardThreshold::new(0).is_err());
        assert!(ProductConfig::keyboard_only(P1Config::default()).is_ok());
    }
}
