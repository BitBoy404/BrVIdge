//! Печать отчёта о целевой машине и синтетических фикстурах.
//!
//! Инструмент не загружает и не запускает игры; работает только с
//! синтетическими данными и проверкой наличия файлов Agility SDK.

use std::path::PathBuf;

fn main() {
    fn option(name: &str) -> Option<String> {
        std::env::args()
            .skip_while(|arg| arg != name)
            .nth(1)
    }

    let report_path = option("--report")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/brv-synthetic-report.json"));
    let log_path = option("--log")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/brv-synthetic-xcl.log"));

    let profile = brv_target::TargetProfile::rtx5060();
    let agility = agility_sdk_present();
    let windows_11 = std::env::var_os("BRV_WINDOWS_11").is_some();

    xcl::log::init(Some(&log_path), xcl::log::Level::Info, false);
    xcl::xlog_info!("synthetic title: brv-synthetic-gdk-title, no game launched");
    xcl::xlog_info!(
        "target: VRAM {} MiB, RAM {} MiB, threads {}, wave32 {}",
        profile.vram_mib,
        profile.ram_mib,
        profile.cpu_threads,
        profile.wave32
    );

    println!("Профиль: RTX 5060 / 32 ГБ / 12 потоков");
    println!("  UMA-бюджет: {} MiB", profile.uma_budget_mib());
    println!("  Фоновые потоки: {}", profile.worker_threads());
    println!("  wave64-эмуляция: {}", profile.needs_wave64_emulation());
    println!();
    println!("Требования среды:");
    for requirement in brv_target::feature_checklist(&profile, agility, windows_11) {
        println!(
            "  [{}] {} — {}",
            if requirement.satisfied { "OK" } else { "НЕТ" },
            requirement.name,
            requirement.detail
        );
    }
    println!();
    let synthetic_image = gta6::synthetic_image(7);
    println!(
        "Синтетический образ GTA6: hash=0x{:016X}, known_build={}",
        gta6::content_hash(&synthetic_image),
        gta6::identify(&synthetic_image).is_some()
    );
    println!();
    println!("{}", gta6::SyntheticTitle::fixture().report());

    let timeline_name = option("--timeline").unwrap_or_else(|| "menus".to_owned());
    let (timeline, budget_fps) = match timeline_name.as_str() {
        "world" => (brv_target::FrameTimeline::world(), 5),
        "stall" => (brv_target::FrameTimeline::stall(), 30),
        _ => (brv_target::FrameTimeline::menus(), 30),
    };
    let budget = brv_target::FrameBudget::from_fps(budget_fps).unwrap();
    let frames = timeline.summary(&budget);
    let frame_note = format!(
        "frame timeline {}: {} frames, avg {} ms, max {} ms, over budget {}",
        timeline_name, frames.count, frames.average_ms, frames.max_ms, frames.over_budget
    );
    xcl::xlog_info!("{}", frame_note);

    let scenario = option("--scenario").unwrap_or_else(|| "boot".to_owned());
    let imports = match scenario.as_str() {
        "series-s" => gta6::synthetic_pe::series_s_imports(),
        "host" => gta6::synthetic_pe::host_imports(),
        _ => gta6::synthetic_pe::boot_imports(),
    };
    let pe_bytes = match gta6::synthetic_pe::build_with_imports(imports) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("Не удалось построить синтетический PE: {error}");
            return;
        }
    };
    match write_synthetic_report(&scenario, &frame_note, &pe_bytes, &report_path, &profile) {
        Ok(()) => println!("Синтетический лог XCL: {}", log_path.display()),
        Err(error) => {
            eprintln!("Не удалось записать синтетический отчёт: {error}");
        }
    }
    xcl::log::flush();
}

fn write_synthetic_report(
    scenario: &str,
    frame_note: &str,
    pe_bytes: &[u8],
    path: &PathBuf,
    profile: &brv_target::TargetProfile,
) -> Result<(), String> {
    xcl::xlog_info!("scenario: {scenario}");
    let pe = xcl::pe::Pe::parse(pe_bytes, xcl::pe::Layout::File)
        .map_err(|error| format!("PE parse: {error:?}"))?;
    let imports = pe
        .imports()
        .map_err(|error| format!("PE imports: {error:?}"))?;
    let routes = gta6::routes_from_xcl_imports(&imports);
    for (import, kind) in &routes {
        xcl::xlog_info!("import {}!{} -> {}", import.dll, import.function, kind);
    }

    let mut report = xcl::report::Report::default();
    report.title = format!("brv-synthetic-{scenario}");
    report.exe = "synthetic-pe32plus".to_owned();
    for (import, kind) in &routes {
        *report
            .imports_by_source
            .entry(kind.to_string())
            .or_default() += 1;
        if *kind == gta6::RouteKind::Stub {
            report.stubbed_imports.push(xcl::report::MissingImport {
                importer: report.title.clone(),
                dll: import.dll.clone(),
                function: import.function.clone(),
            });
        }
    }
    report.environment = environment_checks(profile);
    report.exit_code = Some(0);
    report.crash = None;
    report.notes.push(format!(
        "UMA {} MiB, worker {}, wave64 emulation {}",
        profile.uma_budget_mib(),
        profile.worker_threads(),
        profile.needs_wave64_emulation()
    ));
    report.notes.push("синтетический отчёт, игра не запускалась".to_owned());
    report.notes.push(frame_note.to_owned());
    report.notes.push(
        "ошибок: 0; выделение памяти ограничено бюджетом профиля".to_owned(),
    );

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("create_dir_all {parent:?}: {error}"))?;
        }
    }
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("serialize report: {error}"))?;
    std::fs::write(path, json).map_err(|error| format!("write {path:?}: {error}"))?;
    println!("Синтетический отчёт XCL: {}", path.display());
    Ok(())
}

fn environment_checks(
    profile: &brv_target::TargetProfile,
) -> Vec<xcl::preflight::Check> {
    use xcl::preflight::Level;
    let agility = agility_sdk_present();
    let windows_11 = std::env::var_os("BRV_WINDOWS_11").is_some();
    let mut checks = vec![
        xcl::preflight::Check {
            level: Level::Ok,
            what: "AVX2 on Ryzen 5 5600".to_owned(),
            fix: None,
        },
        xcl::preflight::Check {
            level: Level::Ok,
            what: "DirectX 12 on RTX 5060".to_owned(),
            fix: None,
        },
    ];
    checks.push(if agility || windows_11 {
        xcl::preflight::Check {
            level: Level::Ok,
            what: "Shader Model 6.6".to_owned(),
            fix: None,
        }
    } else {
        xcl::preflight::Check {
            level: Level::Warning,
            what: "Shader Model 6.6".to_owned(),
            fix: Some("install D3D12 Agility SDK or use Windows 11".to_owned()),
        }
    });
    checks.push(xcl::preflight::Check {
        level: if profile.wave32 { Level::Note } else { Level::Ok },
        what: "wave64 console programs".to_owned(),
        fix: if profile.wave32 {
            Some("XCL split_wave64 (two 32-lane halves)".to_owned())
        } else {
            None
        },
    });
    checks.push(xcl::preflight::Check {
        level: Level::Note,
        what: format!("VRAM {} MiB", profile.vram_mib),
        fix: Some("cap UMA at 4096 MiB; stream assets".to_owned()),
    });
    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_pe_returns_error_without_writing_report() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = PathBuf::from(format!(
            "{}/brv-invalid-pe-{nanos}.json",
            std::env::temp_dir().display()
        ));
        let profile = brv_target::TargetProfile::rtx5060();
        let result = write_synthetic_report("boot", "frame test", &[], &path, &profile);
        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn environment_checks_describe_target() {
        let profile = brv_target::TargetProfile::rtx5060();
        let checks = environment_checks(&profile);
        assert_eq!(checks.len(), 5);
        assert!(checks.iter().any(|check| check.what == "wave64 console programs"
            && check.level == xcl::preflight::Level::Note));
        assert!(checks.iter().any(|check| check.what.starts_with("VRAM")));
    }
}

fn agility_sdk_present() -> bool {
    let relative = PathBuf::from("XCL").join("D3D12").join("D3D12Core.dll");
    let path = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|root| root.join(relative))
        .unwrap_or_else(|| PathBuf::from("D3D12/D3D12Core.dll"));
    path.is_file()
}
