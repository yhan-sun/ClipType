use cliptype_core::{
    CapabilityState, CodeAction, InjectionMode, InjectionPlan, PlanCapabilities,
    ProductCapabilities, ProductConfig, SensitiveText, TextAtom, build_injection_plan,
};

fn actions(source: &str) -> Vec<CodeAction> {
    let capabilities = ProductCapabilities {
        keyboard: PlanCapabilities {
            unicode_text: CapabilityState::Available,
            line_break: CapabilityState::Available,
            tab: CapabilityState::Available,
            cursor_right: CapabilityState::Available,
            modifier_observation: CapabilityState::Available,
        },
        clipboard_paste: CapabilityState::Unavailable,
        clipboard_revision_guard: CapabilityState::Unavailable,
    };
    let plan = build_injection_plan(
        SensitiveText::new(source.to_owned()),
        false,
        ProductConfig {
            mode: InjectionMode::Code,
            ..ProductConfig::default()
        },
        capabilities,
    )
    .expect("Code mode needs no paste backend or clipboard revision");
    let InjectionPlan::Code(plan) = plan else {
        panic!("Code must not fall back to another backend");
    };
    plan.actions().to_vec()
}

#[test]
fn heap_sort_plan_continues_after_first_inner_closing_line() {
    let source = include_str!("fixtures/heapify-code-mode.txt");
    let planned = actions(source);
    let boundary = planned
        .iter()
        .position(|action| *action == CodeAction::CursorRightToLineEnd)
        .expect("inner if has a line-leading closer");
    let expected = [
        CodeAction::Atom(TextAtom::LineBreak),
        CodeAction::Atom(TextAtom::Scalar('i')),
        CodeAction::Atom(TextAtom::Scalar('f')),
        CodeAction::Atom(TextAtom::Scalar(' ')),
        CodeAction::Atom(TextAtom::Scalar('r')),
    ];
    assert_eq!(
        &planned[boundary + 1..boundary + 1 + expected.len()],
        &expected
    );
    assert!(planned.len() - boundary > 500, "tail must not be discarded");
    let tail: String = planned[boundary + 1..]
        .iter()
        .filter_map(|action| match action {
            CodeAction::Atom(TextAtom::Scalar(value)) => Some(*value),
            _ => None,
        })
        .collect();
    assert!(tail.contains("pub fn heap_sort<T: Ord>"));
    assert!(tail.contains("fn main"));
    assert!(tail.contains("排序后浮点数"));
}

#[test]
fn lf_and_crlf_make_identical_code_plans() {
    let lf = include_str!("fixtures/heapify-code-mode.txt").replace("\r\n", "\n");
    assert_eq!(actions(&lf), actions(&lf.replace('\n', "\r\n")));
}
