use vn_core::{AssignOp, BinaryOp, Expr, StmtKind, Value};
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
