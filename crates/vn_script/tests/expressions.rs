use vn_core::{AssignOp, BinaryOp, Expr, StmtKind, TextEffect, Value};
use vn_script::parse_source;

#[test]
fn parses_assignment_ops_conditionals_and_menu_conditions() {
    let script = parse_source(
        "expr.vn",
        r#"label start:
    $affection = 1
    $affection += 2
    if affection >= 3 and not locked:
        menu:
            "Ask" if affection >= 3:
                end
    else:
        end
"#,
    )
    .unwrap();

    let StmtKind::Set { op, value, .. } = &script.statements[1].kind else {
        panic!("expected set");
    };
    assert_eq!(*op, AssignOp::Set);
    assert_eq!(*value, Expr::Value(Value::Int(1)));

    let StmtKind::Set { op, value, .. } = &script.statements[2].kind else {
        panic!("expected add");
    };
    assert_eq!(*op, AssignOp::Add);
    assert_eq!(*value, Expr::Value(Value::Int(2)));

    let StmtKind::If {
        cond, then_body, ..
    } = &script.statements[3].kind
    else {
        panic!("expected if");
    };
    assert!(matches!(
        cond,
        Expr::Binary {
            op: BinaryOp::And,
            ..
        }
    ));

    let StmtKind::Menu { choices } = &then_body[0].kind else {
        panic!("expected menu");
    };
    assert_eq!(choices.len(), 1);
    assert!(choices[0].condition.is_some());
}

#[test]
fn parses_visual_transitions_and_text_effects() {
    let script = parse_source(
        "effects.vn",
        r#"label start:
    scene bg room with fade(duration=0.5)
    show eileen happy at left with dissolve(duration=1.25)
    eileen "Hello." with typewriter(speed=45)
"#,
    )
    .unwrap();

    let StmtKind::Scene { transition, .. } = &script.statements[1].kind else {
        panic!("expected scene");
    };
    let transition = transition.as_ref().expect("scene transition");
    assert_eq!(transition.kind, "fade");
    assert_eq!(transition.duration_ms, 500);

    let StmtKind::Show { transition, .. } = &script.statements[2].kind else {
        panic!("expected show");
    };
    let transition = transition.as_ref().expect("show transition");
    assert_eq!(transition.kind, "dissolve");
    assert_eq!(transition.duration_ms, 1250);

    let StmtKind::Say { effect, .. } = &script.statements[3].kind else {
        panic!("expected say");
    };
    assert_eq!(
        *effect,
        TextEffect::Typewriter {
            chars_per_second: 45
        }
    );
}

#[test]
fn rejects_unknown_transition_and_text_effect() {
    assert!(parse_source("bad.vn", "scene bg room with spin(duration=1)").is_err());
    assert!(parse_source("bad.vn", "eileen \"Hi\" with sparkle(speed=1)").is_err());
}
