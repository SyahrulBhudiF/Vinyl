use std::{fs, hint::black_box, time::Instant};

use vn_core::compile;
use vn_script::{load_project, validate};

const FILES: usize = 128;
const DIALOGUES_PER_FILE: usize = 32;

#[test]
#[ignore = "manual release-mode parser performance probe"]
fn multi_file_parser_performance_probe() {
    let root = std::env::temp_dir().join(format!("vn_parser_performance_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    for file_index in 0..FILES {
        let directory = root.join(format!("script/chapter-{:02}", file_index / 16));
        fs::create_dir_all(&directory).unwrap();
        let mut source = if file_index == 0 {
            "label start:\n".to_string()
        } else {
            format!("label chapter_{file_index}:\n")
        };
        for dialogue_index in 0..DIALOGUES_PER_FILE {
            source.push_str(&format!(
                "    \"Chapter {file_index}, line {dialogue_index}.\"\n"
            ));
        }
        if file_index + 1 == FILES {
            source.push_str("    end\n");
        } else {
            source.push_str(&format!("    jump chapter_{}\n", file_index + 1));
        }
        fs::write(directory.join(format!("scene-{file_index:03}.vn")), source).unwrap();
    }

    let load_started = Instant::now();
    let loaded = black_box(load_project(&root).unwrap());
    let load_elapsed = load_started.elapsed();

    let validate_started = Instant::now();
    validate(black_box(&loaded.script), black_box(&root)).unwrap();
    let validate_elapsed = validate_started.elapsed();

    let compile_started = Instant::now();
    let program = black_box(compile(&loaded.script));
    let compile_elapsed = compile_started.elapsed();

    assert_eq!(
        loaded.script.statements.len(),
        FILES * (DIALOGUES_PER_FILE + 2)
    );
    assert_eq!(program.labels.len(), FILES);
    println!(
        "files={FILES} statements={} ops={} load_parse_hash_us={} validate_us={} compile_us={} total_us={}",
        loaded.script.statements.len(),
        program.ops.len(),
        load_elapsed.as_micros(),
        validate_elapsed.as_micros(),
        compile_elapsed.as_micros(),
        (load_elapsed + validate_elapsed + compile_elapsed).as_micros(),
    );

    let _ = fs::remove_dir_all(root);
}
