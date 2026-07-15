use vn_bevy::VnStory;
use vn_core::{Script, SourcePos, Stmt, StmtKind, compile};

fn stmt(kind: StmtKind) -> Stmt {
    Stmt {
        kind,
        pos: SourcePos {
            file: "driver.vn".to_string(),
            line: 1,
            column: 1,
        },
    }
}

fn story(body: Vec<Stmt>) -> VnStory {
    let mut statements = vec![stmt(StmtKind::Label {
        name: "start".to_string(),
    })];
    statements.extend(body);
    VnStory::new(compile(&Script { statements })).unwrap()
}

#[test]
fn end_and_revision_follow_every_story_mutation() {
    let mut story = story(vec![
        stmt(StmtKind::Say {
            speaker: None,
            text_id: None,
            text: "Done.".to_string(),
            effect: Default::default(),
        }),
        stmt(StmtKind::End),
    ]);

    assert_eq!(story.revision(), 0);
    assert!(!story.ended());

    story.continue_story().unwrap();
    assert_eq!(story.revision(), 1);
    assert!(!story.ended());

    story.continue_story().unwrap();
    assert_eq!(story.revision(), 2);
    assert!(story.ended());

    assert!(story.rollback().is_none());
    assert_eq!(story.revision(), 2);
    assert!(story.ended());
}
