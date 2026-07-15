use std::collections::HashMap;

use soa_rs::Soa;
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
                    transition: None,
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Show {
                    tag: "eileen".to_string(),
                    attrs: vec!["happy".to_string()],
                    position: "center".to_string(),
                    transition: None,
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Say {
                    speaker: Some("eileen".to_string()),
                    text_id: None,
                    text: "Hello.".to_string(),
                    effect: vn_core::TextEffect::Instant,
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Menu {
                    choices: vec![Choice {
                        text_id: None,
                        text: "Continue".to_string(),
                        condition: None,
                        body: vec![Stmt {
                            kind: StmtKind::Say {
                                speaker: Some("eileen".to_string()),
                                text_id: None,
                                text: "Good.".to_string(),
                                effect: vn_core::TextEffect::Instant,
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
    let mut vm = Vm::new(program.clone()).unwrap();
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Scene {
            image: "bg room".to_string(),
            transition: None,
        })
    );
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Show {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string()],
            position: "center".to_string(),
            transition: None,
        })
    );
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Dialogue {
            speaker: Some("eileen".to_string()),
            text: "Hello.".to_string(),
            effect: vn_core::TextEffect::Instant,
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
    let mut restored = Vm::from_parts(
        program,
        restored_state,
        restored_presentation,
        vm.rollback_history().clone(),
    );
    assert_eq!(
        restored.choose(0),
        Ok(VmEvent::Dialogue {
            speaker: Some("eileen".to_string()),
            text: "Good.".to_string(),
            effect: vn_core::TextEffect::Instant,
        })
    );
    assert_eq!(
        restored.rollback(),
        Some(VmEvent::Menu {
            choices: vec!["Continue".to_string()]
        })
    );
}

fn dialogue_script(count: usize) -> Script {
    let mut statements = vec![Stmt {
        kind: StmtKind::Label {
            name: "start".to_string(),
        },
        pos: pos(),
    }];
    statements.extend((0..count).map(|index| Stmt {
        kind: StmtKind::Say {
            speaker: None,
            text_id: None,
            text: index.to_string(),
            effect: vn_core::TextEffect::Instant,
        },
        pos: pos(),
    }));
    Script { statements }
}

#[test]
fn rollback_history_persists_and_evicts_the_oldest_checkpoint() {
    let program = compile(&dialogue_script(102));
    let mut vm = Vm::new(program.clone()).unwrap();
    for _ in 0..102 {
        vm.continue_until_interaction().unwrap();
    }
    assert_eq!(vm.rollback_history().len(), 100);
    assert_eq!(vm.rollback_history().vm()[0].pc, 2);

    let json = serde_json::to_string(vm.rollback_history()).unwrap();
    let rollback = serde_json::from_str(&json).unwrap();
    let mut restored = Vm::from_parts(
        program,
        vm.state().clone(),
        vm.presentation().clone(),
        rollback,
    );
    assert_eq!(
        restored.rollback(),
        Some(VmEvent::Dialogue {
            speaker: None,
            text: "100".to_string(),
            effect: vn_core::TextEffect::Instant,
        })
    );
}

#[test]
fn vm_starts_at_start_label_and_restore_keeps_saved_pc() {
    let script = Script {
        statements: vec![
            Stmt {
                kind: StmtKind::Say {
                    speaker: None,
                    text_id: None,
                    text: "before".to_string(),
                    effect: vn_core::TextEffect::Instant,
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Label {
                    name: "start".to_string(),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Say {
                    speaker: None,
                    text_id: None,
                    text: "start".to_string(),
                    effect: vn_core::TextEffect::Instant,
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Say {
                    speaker: None,
                    text_id: None,
                    text: "saved".to_string(),
                    effect: vn_core::TextEffect::Instant,
                },
                pos: pos(),
            },
        ],
    };
    let program = compile(&script);
    let mut vm = Vm::new(program.clone()).unwrap();
    assert!(matches!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Dialogue { ref text, .. }) if text == "start"
    ));
    let mut restored = Vm::from_parts(
        program,
        vm.state().clone(),
        vm.presentation().clone(),
        Soa::new(),
    );
    assert!(matches!(
        restored.continue_until_interaction(),
        Ok(VmEvent::Dialogue { ref text, .. }) if text == "saved"
    ));
}

#[test]
fn branch_uses_deterministic_values() {
    let script = Script {
        statements: vec![
            Stmt {
                kind: StmtKind::Label {
                    name: "start".to_string(),
                },
                pos: pos(),
            },
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
                            text_id: None,
                            text: "seen".to_string(),
                            effect: vn_core::TextEffect::Instant,
                        },
                        pos: pos(),
                    }],
                    else_body: vec![Stmt {
                        kind: StmtKind::Say {
                            speaker: None,
                            text_id: None,
                            text: "new".to_string(),
                            effect: vn_core::TextEffect::Instant,
                        },
                        pos: pos(),
                    }],
                },
                pos: pos(),
            },
        ],
    };
    let mut vm = Vm::new(compile(&script)).unwrap();
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Dialogue {
            speaker: None,
            text: "seen".to_string(),
            effect: vn_core::TextEffect::Instant,
        })
    );
}

#[test]
fn vm_resolves_text_ids_with_fallback_text() {
    let script = Script {
        statements: vec![
            Stmt {
                kind: StmtKind::Label {
                    name: "start".to_string(),
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Say {
                    speaker: Some("eileen".to_string()),
                    text_id: Some("intro-hello".to_string()),
                    text: "Hello.".to_string(),
                    effect: vn_core::TextEffect::Instant,
                },
                pos: pos(),
            },
            Stmt {
                kind: StmtKind::Menu {
                    choices: vec![Choice {
                        text_id: Some("intro-ask".to_string()),
                        text: "Ask".to_string(),
                        condition: None,
                        body: vec![Stmt {
                            kind: StmtKind::Say {
                                speaker: None,
                                text_id: Some("intro-missing".to_string()),
                                text: "Fallback.".to_string(),
                                effect: vn_core::TextEffect::Instant,
                            },
                            pos: pos(),
                        }],
                        pos: pos(),
                    }],
                },
                pos: pos(),
            },
        ],
    };
    let mut translations = HashMap::new();
    translations.insert("intro-hello".to_string(), "Halo.".to_string());
    translations.insert("intro-ask".to_string(), "Tanya".to_string());
    let mut vm = Vm::with_translations(compile(&script), translations).unwrap();

    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Dialogue {
            speaker: Some("eileen".to_string()),
            text: "Halo.".to_string(),
            effect: vn_core::TextEffect::Instant,
        })
    );
    assert_eq!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Menu {
            choices: vec!["Tanya".to_string()]
        })
    );
    assert_eq!(
        vm.choose(0),
        Ok(VmEvent::Dialogue {
            speaker: None,
            text: "Fallback.".to_string(),
            effect: vn_core::TextEffect::Instant,
        })
    );
}

#[test]
fn arithmetic_assignment_and_conditional_menu_choices_work() {
    let script = Script {
        statements: vec![
            Stmt {
                kind: StmtKind::Label {
                    name: "start".to_string(),
                },
                pos: pos(),
            },
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
                            text_id: None,
                            text: "Locked".to_string(),
                            condition: Some(Expr::Binary {
                                left: Box::new(Expr::Var("affection".to_string())),
                                op: BinaryOp::Lt,
                                right: Box::new(Expr::Value(Value::Int(3))),
                            }),
                            body: vec![Stmt {
                                kind: StmtKind::Say {
                                    speaker: None,
                                    text_id: None,
                                    text: "no".to_string(),
                                    effect: vn_core::TextEffect::Instant,
                                },
                                pos: pos(),
                            }],
                            pos: pos(),
                        },
                        Choice {
                            text_id: None,
                            text: "Unlocked".to_string(),
                            condition: Some(Expr::Binary {
                                left: Box::new(Expr::Var("affection".to_string())),
                                op: BinaryOp::Ge,
                                right: Box::new(Expr::Value(Value::Int(3))),
                            }),
                            body: vec![Stmt {
                                kind: StmtKind::Say {
                                    speaker: None,
                                    text_id: None,
                                    text: "yes".to_string(),
                                    effect: vn_core::TextEffect::Instant,
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
    let mut vm = Vm::new(compile(&script)).unwrap();
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
            text: "yes".to_string(),
            effect: vn_core::TextEffect::Instant,
        })
    );
}
