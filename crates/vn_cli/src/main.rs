use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use vn_core::{ProjectId, SaveFile, Vm, VmEvent, compile};
use vn_runtime::{apply_command, commands_from_event};
use vn_script::{load_project, validate_with_locales};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and validate a project.
    Check { project: PathBuf },
    /// Run deterministic CLI smoke execution.
    Run { project: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { project } => check(project),
        Command::Run { project } => run(project),
    }
}

fn check(project: PathBuf) -> Result<()> {
    let loaded =
        load_project(&project).with_context(|| format!("loading {}", project.display()))?;
    if let Err(error) = validate_with_locales(
        &loaded.script,
        &loaded.root,
        &loaded.manifest,
        &loaded.locales,
    ) {
        for diagnostic in error.diagnostics() {
            eprintln!("{}", diagnostic.render());
        }
        bail!("validation failed");
    }
    println!("ok");
    Ok(())
}

fn run(project: PathBuf) -> Result<()> {
    let loaded =
        load_project(&project).with_context(|| format!("loading {}", project.display()))?;
    if let Err(error) = validate_with_locales(
        &loaded.script,
        &loaded.root,
        &loaded.manifest,
        &loaded.locales,
    ) {
        for diagnostic in error.diagnostics() {
            eprintln!("{}", diagnostic.render());
        }
        bail!("validation failed");
    }
    let program = compile(&loaded.script);
    let mut vm = Vm::new(program.clone());
    let mut events = Vec::new();
    let mut presentation = Default::default();
    loop {
        let event = vm.continue_until_interaction()?;
        for command in commands_from_event(&event) {
            apply_command(&mut presentation, &command);
        }
        events.push(format_event(&event));
        if matches!(event, VmEvent::Menu { .. }) {
            let save_json = serde_json::to_string(&SaveFile {
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                game_id: ProjectId::from(loaded.manifest.project.id.clone()),
                script_hash: loaded.script_hash.clone(),
                vm: vm.state().clone(),
                presentation: vm.presentation().clone(),
                preferences: Default::default(),
                screenshot_png: Vec::new(),
                timestamp: 0,
            })?;
            let save: SaveFile = serde_json::from_str(&save_json)?;
            let mut restored = Vm::from_parts(program, save.vm, save.presentation);
            let next = restored.choose(0)?;
            for command in commands_from_event(&next) {
                apply_command(&mut presentation, &command);
            }
            events.push(format_event(&next));
            let rollback = restored
                .rollback()
                .map(|event| format!("rollback:{}", format_event(&event)))
                .unwrap_or_else(|| "rollback:none".to_string());
            events.push(rollback);
            break;
        }
        if matches!(event, VmEvent::End) {
            break;
        }
    }
    for event in events {
        println!("{event}");
    }
    Ok(())
}

fn format_event(event: &VmEvent) -> String {
    match event {
        VmEvent::Dialogue { speaker, text, .. } => match speaker {
            Some(speaker) => format!("say:{speaker}:{text}"),
            None => format!("say:{text}"),
        },
        VmEvent::Scene { image, .. } => format!("scene:{image}"),
        VmEvent::Show {
            tag,
            attrs,
            position,
            ..
        } => {
            format!("show:{}:{}:{position}", tag, attrs.join(" "))
        }
        VmEvent::Hide { tag } => format!("hide:{tag}"),
        VmEvent::PlayMusic { path } => format!("play-music:{path}"),
        VmEvent::StopMusic => "stop-music".to_string(),
        VmEvent::Menu { choices } => format!("menu:{}", choices.join("|")),
        VmEvent::End => "end".to_string(),
    }
}
