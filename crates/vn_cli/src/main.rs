use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use vn_core::{
    CURRENT_SAVE_VERSION, ProjectId, SaveFile, Stmt, StmtKind, Vm, VmEvent, compile, validate_save,
};
use vn_runtime::{apply_command, commands_from_event};
use vn_script::{
    LocaleCatalog, ProjectError, extract_messages, load_project, render_messages,
    validate_with_locales,
};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a new writer-ready VN project.
    New {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Parse and validate a project.
    Check {
        #[arg(default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        locale: Option<String>,
    },
    /// Parse project; placeholder for future source rewriting.
    Fmt {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Print parsed AST as JSON.
    DumpAst {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Print compiled IR as JSON.
    DumpIr {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Print resolved asset paths referenced by scripts.
    ListAssets {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Print Fluent entries extracted from script text ids.
    ExtractLocales {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Validate and start the rendered desktop player.
    Run {
        #[arg(default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        locale: Option<String>,
        #[arg(long, hide = true)]
        visual_test_output: Option<PathBuf>,
    },
    /// Run deterministic headless VM verification.
    Smoke {
        #[arg(default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        locale: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::New { project } => new_project(project),
        Command::Check { project, locale } => check(project, locale),
        Command::Fmt { project } => fmt_project(project),
        Command::DumpAst { project } => dump_ast(project),
        Command::DumpIr { project } => dump_ir(project),
        Command::ListAssets { project } => list_assets(project),
        Command::ExtractLocales { project } => extract_locales(project),
        Command::Run {
            project,
            locale,
            visual_test_output,
        } => run(project, locale, visual_test_output),
        Command::Smoke { project, locale } => smoke(project, locale),
    }
}

fn new_project(project: PathBuf) -> Result<()> {
    std::fs::create_dir_all(project.join("script"))?;
    std::fs::create_dir_all(project.join("assets/bg"))?;
    std::fs::create_dir_all(project.join("assets/sprites/eileen"))?;
    std::fs::create_dir_all(project.join("assets/audio"))?;
    std::fs::create_dir_all(project.join("locale"))?;
    let id = project
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("vinyl-game");
    std::fs::write(
        project.join("vinyl.toml"),
        format!(
            "[project]\nid = \"{id}\"\ntitle = \"{id}\"\nversion = \"0.1.0\"\ndefault_locale = \"en-US\"\n\n[paths]\nscript = \"script\"\nassets = \"assets\"\nlocales = \"locale\"\n\n[assets]\nbackgrounds = \"bg\"\nsprites = \"sprites\"\naudio = \"audio\"\n"
        ),
    )?;
    std::fs::write(
        project.join("script/start.vn"),
        "label start:\n    eileen [intro-hello] \"Hello.\"\n    menu:\n        [intro-continue] \"Continue\":\n            end\n",
    )?;
    std::fs::write(
        project.join("locale/en-US.ftl"),
        "intro-hello = Hello.\nintro-continue = Continue\n",
    )?;
    println!("created {}", project.display());
    Ok(())
}

fn check(project: PathBuf, locale: Option<String>) -> Result<()> {
    let loaded = load_project_or_print(&project)?;
    if let Err(error) = validate_with_locales(
        &loaded.script,
        &loaded.root,
        &loaded.manifest,
        &selected_locales(&loaded.locales, locale.as_deref())?,
    ) {
        for diagnostic in error.diagnostics() {
            eprintln!("{}", diagnostic.render());
        }
        bail!("validation failed");
    }
    println!("ok");
    Ok(())
}

fn fmt_project(project: PathBuf) -> Result<()> {
    let _ = load_project_or_print(&project)?;
    println!("ok");
    Ok(())
}

fn dump_ast(project: PathBuf) -> Result<()> {
    let loaded = load_project_or_print(&project)?;
    println!("{}", serde_json::to_string_pretty(&loaded.script)?);
    Ok(())
}

fn dump_ir(project: PathBuf) -> Result<()> {
    let loaded = load_project_or_print(&project)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&compile(&loaded.script))?
    );
    Ok(())
}

fn list_assets(project: PathBuf) -> Result<()> {
    let loaded = load_project_or_print(&project)?;
    let resolver = vn_script::AssetResolver::new(&loaded.root, loaded.manifest.clone());
    for asset in referenced_assets(&loaded.script.statements) {
        println!("{}", resolver.resolve(&asset).display());
    }
    Ok(())
}

fn extract_locales(project: PathBuf) -> Result<()> {
    let loaded = load_project_or_print(&project)?;
    print!("{}", render_messages(&extract_messages(&loaded.script)));
    Ok(())
}

#[cfg(feature = "desktop")]
fn run(
    project: PathBuf,
    locale: Option<String>,
    visual_test_output: Option<PathBuf>,
) -> Result<()> {
    let loaded = load_validated_project(&project, locale.as_deref())?;
    let active_locale = locale.unwrap_or_else(|| loaded.manifest.project.default_locale.clone());
    vn_bevy::run_player(vn_bevy::PlayerConfig {
        project_root: loaded.root,
        manifest: loaded.manifest.clone(),
        program: compile(&loaded.script),
        translations: translations_for(&loaded.locales, &active_locale),
        project_id: ProjectId::from(loaded.manifest.project.id),
        project_version: loaded.manifest.project.version,
        script_hash: loaded.script_hash,
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        visual_test: visual_test_output.map(|output| vn_bevy::VisualTestConfig { output }),
    })?;
    Ok(())
}

#[cfg(not(feature = "desktop"))]
fn run(
    project: PathBuf,
    locale: Option<String>,
    _visual_test_output: Option<PathBuf>,
) -> Result<()> {
    let _ = load_validated_project(&project, locale.as_deref())?;
    bail!("desktop player support is disabled in this build")
}

fn smoke(project: PathBuf, locale: Option<String>) -> Result<()> {
    let loaded = load_validated_project(&project, locale.as_deref())?;
    let active_locale = locale.unwrap_or_else(|| loaded.manifest.project.default_locale.clone());
    let program = compile(&loaded.script);
    let translations = translations_for(&loaded.locales, &active_locale);
    let mut vm = Vm::with_translations(program.clone(), translations)?;
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
                save_version: CURRENT_SAVE_VERSION,
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                game_id: ProjectId::from(loaded.manifest.project.id.clone()),
                project_version: loaded.manifest.project.version.clone(),
                script_hash: loaded.script_hash.clone(),
                vm: vm.state().clone(),
                presentation: vm.presentation().clone(),
                rollback: vm.rollback_history().clone(),
                screenshot_png: Vec::new(),
                timestamp: 0,
            })?;
            let save: SaveFile = serde_json::from_str(&save_json)?;
            validate_save(
                &save,
                &ProjectId::from(loaded.manifest.project.id.clone()),
                &loaded.manifest.project.version,
                &loaded.script_hash,
            )?;
            let mut restored = Vm::from_parts(program, save.vm, save.presentation, save.rollback);
            restored.set_translations(translations_for(&loaded.locales, &active_locale));
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

fn load_validated_project(
    project: &std::path::Path,
    locale: Option<&str>,
) -> Result<vn_script::LoadedProject> {
    let loaded = load_project_or_print(project)?;
    if let Err(error) = validate_with_locales(
        &loaded.script,
        &loaded.root,
        &loaded.manifest,
        &selected_locales(&loaded.locales, locale)?,
    ) {
        for diagnostic in error.diagnostics() {
            eprintln!("{}", diagnostic.render());
        }
        bail!("validation failed");
    }
    Ok(loaded)
}

fn referenced_assets(statements: &[Stmt]) -> Vec<vn_script::AssetId> {
    let mut assets = Vec::new();
    for statement in statements {
        match &statement.kind {
            StmtKind::Scene { image, .. } => {
                assets.push(vn_script::AssetId::Background(image.clone()))
            }
            StmtKind::Show { tag, attrs, .. } => assets.push(vn_script::AssetId::Sprite {
                tag: tag.clone(),
                attrs: attrs.clone(),
            }),
            StmtKind::PlayMusic { path } => assets.push(vn_script::AssetId::Audio(path.clone())),
            StmtKind::Menu { choices } => {
                for choice in choices {
                    assets.extend(referenced_assets(&choice.body));
                }
            }
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                assets.extend(referenced_assets(then_body));
                assets.extend(referenced_assets(else_body));
            }
            _ => {}
        }
    }
    assets
}

fn load_project_or_print(project: &std::path::Path) -> Result<vn_script::LoadedProject> {
    match load_project(project) {
        Ok(project) => Ok(project),
        Err(ProjectError::Diagnostics(diagnostics)) => {
            for diagnostic in diagnostics {
                eprintln!("{}", diagnostic.render());
            }
            bail!("parse failed");
        }
        Err(error) => Err(error).with_context(|| format!("loading {}", project.display())),
    }
}

fn selected_locales(locales: &[LocaleCatalog], locale: Option<&str>) -> Result<Vec<LocaleCatalog>> {
    match locale {
        Some(locale) => locales
            .iter()
            .find(|catalog| catalog.locale == locale)
            .cloned()
            .map(|catalog| vec![catalog])
            .with_context(|| format!("locale '{locale}' not loaded")),
        None => Ok(locales.to_vec()),
    }
}

fn translations_for(locales: &[LocaleCatalog], locale: &str) -> HashMap<String, String> {
    locales
        .iter()
        .find(|catalog| catalog.locale == locale)
        .map(|catalog| {
            catalog
                .messages
                .iter()
                .map(|(id, text)| (id.clone(), text.clone()))
                .collect()
        })
        .unwrap_or_default()
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
