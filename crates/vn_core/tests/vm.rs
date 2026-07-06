use vn_core::{
    AssignOp, BinaryOp, Choice, Expr, Script, SourcePos, Stmt, StmtKind, Value, Vm, VmEvent,
    compile,
};

fn pos() -> SourcePos {
    SourcePos {
        file: "test.vn".to_string(),
        line: 1,
        column: 1,
    }
}

#[test]
fn vm_runs_menu_save_restore_and_rollback() {
    let script = Script {
        statements: vec![
            Stmt {
                kind: StmtKind::Label {
                    name: "start".to_string(),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Scene {
                    image: "bg room".to_string(),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Show {
                    tag: "eileen".to_string(),
                    attrs: vec!["happy".to_string()],
                    position: "center".to_string(),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Say {
                    speaker: Some("eileen".to_string()),
                    text: "Hello.".to_string(),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Menu {
                    choices: vec![Choice {
                        text: "Continue".to_string(),
                        condition: None,
                        body: vec![Stmt {
                            kind: StmtKind::Say {
                                speaker: Some("eileen".to_string()),
                                text: "Good.".to_string(),
                            },
                            pos: pos(),
                        }],
                        pos: pos(),
                    }],
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::End,
                pos: pos(),
            },
        ],
    };
    let program = compile(&script);
    let mut vm = Vm::new(program.clone());
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Scene {
            image: "bg room".to_string()
        })
    );
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Show {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string()],
            position: "center".to_string()
        })
    );
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Dialogue {
            speaker: Some("eileen".to_string()),
            text: "Hello.".to_string()
        })
    );
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Menu {
            choices: vec!["Continue".to_string()]
        })
    );
    let restored_state = vm.state().clone();
    let restored_presentation = vm.presentation().clone();
    let mut restored = Vm::from_parts(program, restored_state, restored_presentation);
    assert_eq!(
        restored.choose(0),
        Ok(VmEvent::Dialogue {
            speaker: Some("eileen".to_string()),
            text: "Good.".to_string()
        })
    );
    assert_eq!(
        restored.rollback(),
        Some(VmEvent::Menu {
            choices: vec!["Continue".to_string()]
        })
    );
}

#[test]
fn branch_uses_deterministic_values() {
    let script = Script {
        statements: vec![
            Stmt {
                kind: StmtKind::Set {
                    var: "seen".to_string(),
                    op: AssignOp::Set,
                    value: Expr::Value(Value::Bool(true)),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::If {
                    cond: Expr::Var("seen".to_string()),
                    then_body: vec![Stmt {
                        kind: StmtKind::Say {
                            speaker: None,
                            text: "seen".to_string(),
                        },
                        pos: pos(),
                    }],
                    else_body: vec![Stmt {
                        kind: StmtKind::Say {
                            speaker: None,
                            text: "new".to_string(),
                        },
                        pos: pos(),
                    }],
                },
                pos: pos(),
            },
        ],
    };
    let mut vm = Vm::new(compile(&script));
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Dialogue {
            speaker: None,
            text: "seen".to_string()
        })
    );
}

#[test]
fn arithmetic_assignment_and_conditional_menu_choices_work() {
    let script = Script {
        statements: vec![
            Stmt {
                kind: StmtKind::Set {
                    var: "affection".to_string(),
                    op: AssignOp::Set,
                    value: Expr::Value(Value::Int(1)),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Set {
                    var: "affection".to_string(),
                    op: AssignOp::Add,
                    value: Expr::Value(Value::Int(2)),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Menu {
                    choices: vec![
                        Choice {
                            text: "Locked".to_string(),
                            condition: Some(Expr::Binary {
                                left: Box::new(Expr::Var("affection".to_string())),
                                op: BinaryOp::Lt,
                                right: Box::new(Expr::Value(Value::Int(3))),
                            }),
                            body: vec![Stmt {
                                kind: StmtKind::Say {
                                    speaker: None,
                                    text: "no".to_string(),
                                },
                                pos: pos(),
                            }],
                            pos: pos(),
                        },
                        Choice {
                            text: "Unlocked".to_string(),
                            condition: Some(Expr::Binary {
                                left: Box::new(Expr::Var("affection".to_string())),
                                op: BinaryOp::Ge,
                                right: Box::new(Expr::Value(Value::Int(3))),
                            }),
                            body: vec![Stmt {
                                kind: StmtKind::Say {
                                    speaker: None,
                                    text: "yes".to_string(),
                                },
                                pos: pos(),
                            }],
                            pos: pos(),
                        },
                    ],
                },
                pos: pos(),
            },
        ],
    };
    let mut vm = Vm::new(compile(&script));
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Menu {
            choices: vec!["Unlocked".to_string()]
        })
    );
    assert_eq!(
        vm.choose(0),
        Ok(VmEvent::Dialogue {
            speaker: None,
            text: "yes".to_string()
        })
    );
}
