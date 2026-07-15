use std::{collections::HashMap, mem::size_of};

use soa_rs::Soa;
use vn_core::{DialogueSnapshot, PresentationSnapshot, RollbackCheckpoint, Value, VmState};

fn checkpoint(index: usize) -> RollbackCheckpoint {
    RollbackCheckpoint {
        vm: VmState {
            pc: index,
            variables: HashMap::from([("route".to_string(), Value::Int(index as i64))]),
            current_choices: Vec::new(),
            history: Vec::new(),
        },
        presentation: PresentationSnapshot {
            background: Some("bg room".to_string()),
            music: Some("theme.mp3".to_string()),
            dialogue: Some(DialogueSnapshot {
                speaker: Some("eileen".to_string()),
                text: format!("Checkpoint {index}"),
            }),
            ..Default::default()
        },
    }
}

#[test]
fn compare_rollback_aos_and_soa_layout() {
    let aos_struct = (0..100).map(checkpoint).collect::<Vec<_>>();
    let aos_tuple = aos_struct
        .iter()
        .cloned()
        .map(|checkpoint| (checkpoint.vm, checkpoint.presentation))
        .collect::<Vec<_>>();
    let mut soa = Soa::<RollbackCheckpoint>::with_capacity(100);
    for index in 0..100 {
        soa.push(checkpoint(index));
    }

    let vm_bytes = size_of::<VmState>();
    let presentation_bytes = size_of::<PresentationSnapshot>();
    let checkpoint_bytes = size_of::<RollbackCheckpoint>();
    let tuple_bytes = size_of::<(VmState, PresentationSnapshot)>();
    let aos_container_bytes = aos_struct.capacity() * checkpoint_bytes;
    let soa_container_bytes = soa.capacity() * (vm_bytes + presentation_bytes);
    let tuple_json_bytes = serde_json::to_vec(&aos_tuple).unwrap().len();
    let struct_json_bytes = serde_json::to_vec(&aos_struct).unwrap().len();
    let soa_json_bytes = serde_json::to_vec(&soa).unwrap().len();

    println!(
        "VmState={vm_bytes}, PresentationSnapshot={presentation_bytes}, RollbackCheckpoint={checkpoint_bytes}, tuple={tuple_bytes}; 100 checkpoints: AoS={aos_container_bytes} bytes/1 allocation, SoA={soa_container_bytes} bytes/1 allocation; JSON tuple={tuple_json_bytes}, AoS struct={struct_json_bytes}, SoA={soa_json_bytes}"
    );
    assert_eq!(soa.vm().len(), 100);
    assert_eq!(soa.presentation().len(), 100);
    assert_eq!(checkpoint_bytes, vm_bytes + presentation_bytes);
    assert_eq!(tuple_bytes, checkpoint_bytes);
    assert_eq!(soa_container_bytes, aos_container_bytes);
    assert_eq!(soa_json_bytes, struct_json_bytes);
}
