# PROJECT_AUDIT — сравнение BrVIdge с проектами Xbox → Windows

Дата: 25 ноября 2026 года. Тип: архитектурный аудит, без изменения кода.

---

## 1. Executive Summary

- BrVIdge прошёл фазы 1–6 и имеет **81 синтетический тест**, работающий PE loader/нативный запуск, базовый D3D12-рендер, SwapChain, прототип RDNA2→DXIL и административные модули. Ни одного реального игрового теста нет; Phase 7 формально не закрыта.
- Все пять указанных репозиториев доступны, `git ls-remote` не показал перенаправлений/переименований.
- Лучшая **архитектурная база для дальнейшего развития** — `xcl-dev/xcl` (GPL-2.0-only): 934 теста, 163 файла `.rs`, реальная работа с играми, полный транслятор шейдеров, D3D12.X, UMA, XGameRuntime.
- Лучшая **база именно для GTA VI-демонстрации** — `fxceless/victory-fork` (GPL-3.0-or-later): единственный проект, где мир GTA VI уже рендерится с текстурами, небом и HUD. Но он сильно привязан к одной игре и GPU, работает 2,5–4 fps на RTX 4070 12 ГБ и требует около 24 ГБ памяти.
- `scarlett-layer` (Apache-2.0) и `Xbox2Windows` (MIT) — ценные источники открытых компонентов и тестовых идей, но пока не обладают ни зрелым GPU-бэкендом, ни демонстрацией GTA VI.
- Лицензии несовместимы попарно: GPL-2-only (XCL) нельзя объединять с GPL-3 (BrVIdge/VIctory) или Apache-2.0 (scarlett-layer). MIT-код Xbox2Windows совместим с GPL-2 и GPL-3.
- **Рекомендация:** основной репозиторий — форк `xcl-dev/xcl`; BrVIdge оставить исследовательской веткой и архивом архитектурных решений/тестов. GTA VI-специфичные наработки реализовывать в форке XCL clean-room, не перенося GPL-3-код VIctory.

---

## 2. Текущее состояние BrVIdge

### 2.1 Подтверждённая реализация (по исходникам и тестам)

| Компонент | Файлы/символы | Статус |
|---|---|---|
| PE32+-анализ | `crates/brv_pe/src/lib.rs`: `PeHeaders::parse`, `imports`, `rva_to_offset`, `synthetic::build_minimal_pe32plus`, `build_native_pe32plus` | Реализовано, тесты есть |
| Подготовка PE | `crates/brv_loader/src/lib.rs`: `prepare`, `Resolver::resolve`, `runtime_status` | Реализовано, тесты есть |
| Нативный запуск | `crates/brv_exec/src/lib.rs`: `map_image`, `run_entry`, `run_entry_in_thread`, `capture_crash` | Реализовано, 5 тестов |
| Конфиг/логи/диагностика | `crates/brv_config`, `brv_log`, `brv_diag`; `brv_diag::check_dir`, `report_json` | Реализовано, 15 тестов |
| HLE и runtime | `crates/brv_hle/src/lib.rs`: `Registry::with_builtin`, `HostPolicy`, `census`; `brv_runtime::run_title`, `SyntheticTitle` | Частично реализовано (3 функции) |
| D3D12 | `crates/brv_gpu/src/lib.rs`: `render_triangle`, `Presentation::new/present_clear`, `adapter_info` | Экспериментально, тесты на реальном GPU |
| Списки команд | `crates/brv_gpu/src/console_list.rs`: `interpret`, 9 тегов | Экспериментально, собственный формат записей |
| Менеджеры | `crates/brv_gpu/src/manager.rs`: `PipelineCache`, `DescriptorAllocator` | Экспериментально |
| RDNA2 → DXIL | `crates/brv_shader/src/lib.rs`: `decode_u32` (11 инструкций), `emit_compute_hlsl`; `crates/brv_dxc/src/lib.rs`: `compile_rdna2_subset` | Частично реализовано |
| Кэш шейдеров | `crates/brv_shader_cache/src/lib.rs`: `ShaderCache::get/put/key` | Реализовано как прототип |
| Тайлинг/BC1/BCPack | `crates/brv_formats/src/lib.rs`: `tiling::tile_8bpp/detile_8bpp`, `bc1::decode_block`, `bcpack::decode` | Тайлинг и BC1 реализованы; BCPack явно не реализован |

### 2.2 Заглушки и честно нереализованное

- `brv_loader::runtime_status()` буквально сообщает, что отображение реального PE/TLS/потоки в general-case не реализованы.
- `brv_formats::bcpack::decode()` возвращает `FormatError::Unsupported` — это явный отказ, а не подставной результат.
- `brv_hle` реализует 3 функции (`XMemAlloc`, `XMemFree`, `XUserGetXUID`); XGameRuntime offline, XAudio2, GameInput, DirectStorage, VFS games — не реализованы.
- `brv_gpu::console_list` использует **собственный** формат записей; в комментариях прямо сказано, что это не реконструкция D3D12.X.
- Нет UMA guest-модели, нет PresentX, нет wave64-on-wave32, нет 64-bit atomics, нет Job Object для DXC.
- Ни один тест не использует файлы GTA VI.

### 2.3 Соответствие статусу Phase 6

Документация заявляет Phase 6 завершённой. Проверка подтверждает это **в объёме синтетического прототипа**:

- `cargo test --workspace` — 81 тест, 0 failed;
- `cargo clippy --workspace --all-targets -- -D warnings` — чисто;
- `brv native` — PE отображается, `DIR64`-релок применяется, machine-код возвращает ожидаемый ExitCode;
- `brv run` — `data/report.json` содержит 3 вызова API и `unsupported_api = "нет"`.

Не подтверждено: TLS/SEH реальных игр, загрузка настоящих PE, работа с GTA VI. Если под «Phase 6» понимать «готовый CPU backend для GTA VI», статус завышен; если понимать «прототип с проверяемыми тестами» — корректен. В отчётах BrVIdge это ограничение зафиксировано в `docs/reports/phase6.md`.

### 2.4 Что сохранить независимо от выбора основы

- Документация: `README.md`, `RESEARCH.md`, `ARCHITECTURE.md`, `FEASIBILITY.md`, `RISKS.md`, отчёты `docs/reports/`.
- Собственные тесты и фикстуры: `brv_pe::synthetic::build_native_pe32plus`, `brv_exec` тесты релокаций/потоков, `brv_diag` asset-policy.
- Модули `brv_config`, `brv_log`, `brv_diag` — это наш собственный код, при необходимости может быть перелицензирован в GPL-2 для transfers в форк XCL.
- Идеи версионированного content-addressed кэша (`brv_shader_cache`) и честного `unsupported`-отчёта.

### 2.5 Что придётся переписывать

- `brv_formats::tiling` — собственная упрощённая модель, несовместимая с GFX10.
- `brv_gpu::console_list` — не реальный формат D3D12.X.
- `brv_gpu::manager` — прототипной логики без реальных D3D12-объектов.
- `brv_shader` — 11 инструкций из сотен.
- `brv_hle` — 3 функции вместо сотен.

---

## 3. Анализ каждого внешнего проекта

### 3.1 fxceless/victory-fork

- URL: `https://github.com/fxceless/victory-fork`; HEAD `4e17478b9c27bb530643e5b0e16a767cf4419e08` (2026-11-24 15:45 -0500). Перенаправлений нет.
- Язык: Rust, workspace как у VIctory (`crates/*`, `apps/*`, `xtask`), 169 файлов, license GPL-3.0-or-later (`LICENSE`).
- Зависимость от upstream: README прямо называет `Marvisak/VIctory` основой; история upstream сохранена под одним коммитом с изменениями форка.
- XCL упомянут как reference; исправления написаны заново в коде VIctory (утверждение автора, не независимо проверено).
- diff с `Marvisak/VIctory@ffe310e` (сравнение рабочих деревьев): **106 файлов, +37 509 / −1 491**. Крупнейшие изменения: `victory-gpu/src/host.rs` (+4594), `victory-shader/src/emit/lower.rs` (+4363), `victory-gpu/src/shadow.rs` (+2789), новые `emit/bitcode/opt.rs` (+2136), `victory-gpu/src/inplace.rs` (+1170), `victory-shader/src/bvh.rs` (+696), `emit/intersect.rs` (+557), `emit/wave64_tests.rs` (+452).
- GTA VI-специфичные исправления по README: meshlet culling view, descriptor table exhaustion, readback compute-packed light, fog reconstruction/shadow-mask/hair/blend-shape skinning/fire/material programs, gather mip level, implicit-level sampling в 730 compute programs, rectangle lists/depth twins/texture readback.
- Ray tracing in progress, off by default: 59/64 trace programs translate, DXIL-эмуляция ray/box и ray/triangle, проверка на GPU против reference; ничего не освещается. Файлы: `docs/fork-ray-tracing.md`, `victory-shader/src/bvh.rs`, `emit/intersect.rs`.
- Скорость/память: `VICTORY_LATE_WAIT`, `VICTORY_RELAY`, `VICTORY_WIDE_BACK`, `VICTORY_INPLACE`, `VICTORY_CARD_BUFFERS_*`, `VICTORY_SHADER_OPT`. Оптимизация DXIL снизила host instruction count ~43% и ~20 мс/кадр на тестовой машине (утверждение README).
- Статус по README: 2,5–4 fps, 420–450 мс/кадр (240–350 мс с новыми флагами), тени чёрные, ~24 ГБ памяти во время игры, тестовая машина i7-14700KF + RTX 4070 12 ГБ + 32 ГБ RAM.
- Открытых issues: 32, PR: 50 (GitHub issues page). Известные проблемы в issue: RTX 5070 Laptop 8 ГБ — `VICTORY_RELAY=2` ломает skin/subtitles (#89); RTX 3070 32-lane — driver отказывает ~80 compute pipeline (#75); RTX 3060 Ti 8 ГБ — 2–3 fps, device loss (#18); Series S profile — FSR4 E_NOTIMPL/device removed (#13).
- Привязка к GPU: высокая — много флагов, `shortcut:` в коде, часть изменений специфична для одного тайтла и одного GPU.
- Лицензия: GPL-3.0-or-later, совместима с BrVIdge.

### 3.2 xcl-dev/xcl

- URL: `https://github.com/xcl-dev/xcl`; HEAD `e9e94ce665d16bb0e2d2e4cb6b37e149f0d27721` (2026-11-24 22:00 +0100). Перенаправлений нет.
- Rust 2021, `crates/xcl` + `tests/testtitle` + `tests/testgamedll`; 163 `.rs`, статически 934 `#[test]`, 42 `#[ignore]`.
- CPU: `loader.rs:60 map_image`, `:64 map_image_ex`, `:337 run_main`, `:248 run_tls_callbacks`; нативный x64 без эмуляции; TLS, SEH, delay-load; resolver с 5 уровнями; `stubs.rs` и `data/stubdb.toml`.
- Память: `gpu/uma.rs` — `MAX_SIZE = (4096 << 20) - GRAIN`, 4 окна по 4 GiB, `OpenExistingHeapFromAddress`, first-fit allocator.
- HLE: `hle/kernelx.rs exports()` — 60 экспортов; `hle/xgameruntime/mod.rs` — 127; `gdk_table.rs` — 396 GDK-слотов; VFS overlay (`vfs.rs`, `vfs_hooks.rs`), XAudio2/XMA/Opus/GameInput, DirectStorage/BCPack.
- GPU: `gpu/recomp::translate/translate_mesh/translate_geometry`, `gpu/dxil/*` (прямой DXIL), `hle/d3d12x/console_list.rs` (`PIPE_MARKER 0x8C0F_E002`, `DISPATCH 0x4002_C400`, `LIST_SLOTS=600`, `close()`), `real.rs/list_x.rs/device_x.rs/present.rs`.
- Текстуры: `gpu/tiling.rs` — 14 2D-режимов, `detile`, `detile_level`, `detile_volume`, `image_layout`; BCPack decode в `hle/bcpack/mod.rs`.
- Тесты: 16 integration-файлов; `gpu_pso_e2e.rs::retail_vs_ps_through_the_pso_hook_render_and_read_back`, `e2e.rs::test_title_runs_and_game_folder_is_untouched`, `console_dlls_e2e.rs::bink2_console_dll_decodes_a_game_movie`.
- Управление ресурсами: `XCL_HUGE_PSO_BYTES`, `XCL_HUGE_PSO_CONCURRENCY`, гейт 3 ГБ commit, `XCL_DSTORAGE_*`, DRED, frame dumps (`README.md`).
- Архитектурная пригодность: высокая — data-driven маршрутизация импортов, модульные HLE/GPU, тесты с real-data opt-in, явные ограничения. Это не только демо: есть инфраструктура для расширения.
- Лицензия: GPL-2.0-only, `THIRD_PARTY.md` фиксирует AnyPS5 GPL-2, FFmpeg LGPL-2.1+, AMD/Mesa/silicon-rans MIT, LLVM Apache-with-exception, VIctory BCPack по персональному разрешению.
- Известные ограничения: GPU developed on RDNA4, wave64 на NVIDIA через две половины, GTA VI gameplay не достигнут, часть D3D12.X-записей не реконструирована.

### 3.3 VIctory upstream (Marvisak/VIctory)

- URL: `https://github.com/Marvisak/VIctory`; HEAD `ffe310eb2098e52169f4c44db5ffdabf43a72622` (2026-11-24 10:53 +0100). Репозиторий доступен, это и есть актуальный upstream для `fxceless/victory-fork` (по README форка).
- Rust 2024, GPL-3.0-or-later, 352 теста по аудиту исходников (Phase 1 research), `cargo test --workspace` без игровых данных.
- Архитектура: `victory-loader` (native x64 + micro-stubs + TLS patch + VEH для AMD-only инструкций), `victory-hle` (15 модулей, 133 экспорта, 12 `QueryApiImpl` area), `victory-gpu` (D3D12.X → D3D12, indirect draws, shadow copies 64 KiB), `victory-shader` (SSA IR, structure, resource tracing, собственный LLVM bitcode writer DXIL), `victory-formats` (BCPack, XMA, tiling), `victory-game` (GTA VI identity/hooks).
- Ограничения upstream: меню 30 fps, мир загружается, но тёмный, 0,5–1,8 с/кадр, риск сброса драйвера; Series S только до загрузки; 64-bit image atomics/merged geometry/RT builder не переведены.
- Лицензия GPL-3.0-or-later — совместима с BrVIdge и форком.

### 3.4 jadeowaowa/scarlett-layer

- URL: `https://github.com/jadeowaowa/scarlett-layer`; HEAD `0e436b3e1c654765b193c1cb705918fbf101b10b` (2026-11-23 19:30 -0500). Перенаправлений нет.
- C++23 + CMake + MSVC, 122 файла, Apache-2.0 (`LICENSE`), `THIRD_PARTY.md`.
- Структура: `src/loader` (`pe_patcher.cpp`, `scarlett_hook.cpp`), `src/shims` (16 DLL: `d3d12_xs`, `dstorage_xs`, `xmem`, `xg_xs`, `xgameruntime`, `gameinput`, `xaudio2_9`, `acphal`, ...), `src/common`, `src/tests`.
- Реально реализовано по README/исходникам: suspended launch + APC injection + IAT rewrite, VFS `G:\/T:\/S:\`, Headless/Flip-Discard PresentX (`src/shims/d3d12_xs/command_queue_x.cpp:129 PresentX`), 78/78 Tier 1 synthetic checks (42 CPU, 36 GPU) на RTX 5070 и WARP, 16 shim DLL / 166 exports, Terraria (Xbox GDK) menus + audio.
- RDNA2 lifter: `src/shims/d3d12_xs/rdna2_lifter.hpp`, `rdna2_isa.hpp`, `shader_translator.hpp`; 28-case oracle; 255-ELF gfx1030 sweep: 100% decode, 49,4% strict / 96,1% compat DXBC compile (утверждение README).
- Ограничения: Tier 2 CI Xbox-targeted binaries = 0; SCLZ/SCXS и часть COM-контрактов `ASSUMED` (`docs/SPEC_SOURCES.md`); hardware testing только RTX 5070 + WARP; `pe_patcher.cpp`/`scarlett_hook.cpp` ждут human audit; код AI-assisted, clean-room certification не заявлена.
- Открытых issues: 0 (GitHub issues page), PR: 2 — активность низкая.
- Лицензия Apache-2.0: совместима с BrVIdge GPL-3, **несовместима** с GPL-2-only XCL.

### 3.5 R0Y00/Xbox2Windows

- URL: `https://github.com/R0Y00/Xbox2Windows`; HEAD `9881fef010d7275d6a3a9d3a5802b1c50b8d4646` (2026-11-25 05:20 +0800). Перенаправлений нет.
- C++/Python/CMake, 1409 файлов (265 `.cpp`, 77 `.h`, 532 `.py`), MIT license (`LICENSE`).
- Подход: **статическая рекомпиляция** x64 → C++ и gfx1013 shaders → DXIL. Пример: `tools/cpu_exe_aot.py` использует `pefile` и `iced_x86`, генерирует C++ из PE entry point.
- Реально подтверждено README/issue: source-built EXE читает BC7-текстуры и рисует; native/mapped/AOT совпадают на RTX 4060 Laptop и WARP; 26 500 из 26 503 shader records имеют DXIL-compilation route; 851/12 101 PS compiled и совпадают с controlled output; 12/2689 compute shaders под тестовыми UAV bindings; mesh draw на 32×32; BVH differential 408 queries.
- Открытых issues: 92, PR: 99 (GitHub page) — высокая активность. Issue #870 — P0 startup progress; текущая граница — IAT/OOM/BVH, gameplay не подтверждён.
- Память: в issue #870 — 14 GiB cap insufficient, 32 GiB cap проходит загрузку, но дальше C++ exception из `XboxBvhBuilder.dll`; пик commit 16,1 GiB; title ~10–13 GiB + stand-ins.
- Железо: в таблице hardware coverage есть **RTX 5060** — native 32 lanes, forced wave64 skipped, BVH 408 queries passed (несколько reports). Это прямое свидетельство для конфигурации пользователя, не нужно экстраполировать.
- Лицензия MIT — совместима и с GPL-2, и с GPL-3; лучший кандидат для переноса отдельных идей/кода.

---

## 4. Сравнительная таблица

Оценки: `Не реализовано`, `Экспериментально`, `Частично реализовано`, `Реализовано`, `Не удалось проверить`.

| Критерий | BrVIdge | fxceless fork | VIctory upstream | XCL | scarlett-layer | Xbox2Windows |
|---|---|---|---|---|---|---|
| Качество архитектуры | Частично реализовано | Частично реализовано | Реализовано | Реализовано | Частично реализовано | Частично реализовано |
| Зрелость CPU backend | Частично реализовано | Реализовано (native) | Реализовано (native) | Реализовано (native) | Реализовано (loader+shims) | Экспериментально (static recomp) |
| Зрелость GPU backend | Экспериментально | Частично реализовано | Частично реализовано | Реализовано | Частично реализовано | Частично реализовано |
| Поддержка Xbox API | Частично реализовано | Частично реализовано | Частично реализовано | Реализовано | Частично реализовано | Частично реализовано |
| Поддержка памяти | Не реализовано | Частично реализовано | Частично реализовано | Реализовано | Частично реализовано | Частично реализовано |
| Загрузчик | Реализовано (synthetic) | Реализовано | Реализовано | Реализовано | Реализовано | Реализовано |
| Совместимость Windows 10/11 | Реализовано (настройки) | Частично реализовано | Частично реализовано | Реализовано | Реализовано | Реализовано |
| Производительность | Не удалось проверить | Частично реализовано (2,5–4 fps) | Частично реализовано (0,5–1,8 с/кадр) | Реализовано (30 fps в меню) | Частично реализовано (78/78 synth) | Не реализовано (gameplay) |
| Стабильность | Экспериментально | Частично реализовано | Частично реализовано | Реализовано | Частично реализовано | Частично реализовано |
| Переносимость на разные GPU | Экспериментально | Частично реализовано | Частично реализовано | Частично реализовано | Частично реализовано | Реализовано (hardware table) |
| Документация | Реализовано | Реализовано | Реализовано | Реализовано | Реализовано | Реализовано |
| Тестовое покрытие | Реализовано (81 synth) | Не удалось проверить (issue #29: 496 passed на RX 7600) | Реализовано (352) | Реализовано (934) | Реализовано (78/78) | Реализовано (большой CTest suite) |
| Активность разработки | Частично реализовано | Реализовано (32 issues, 50 PR) | Частично реализовано (HEAD 24.11) | Реализовано (PR #1426) | Частично реализовано (0 issues, 2 PR) | Реализовано (92 issues, 99 PR) |
| Удобство расширения | Частично реализовано | Частично реализовано | Частично реализовано | Реализовано | Частично реализовано | Частично реализовано |
| Лицензионные ограничения | GPL-3-or-later | GPL-3-or-later | GPL-3-or-later | GPL-2-only | Apache-2.0 | MIT |

Пояснения к оценкам:
- «Реализовано» для XCL GPU/API означает подтверждённые исходники и тесты, а не «GTA VI gameplay».
- Для BrVIdge «Реализовано (synthetic)» по загрузчику означает реальный `map_image` и релоки на синтетическом PE; TLS/SEH реальных игр отсутствуют.
- Производительность BrVIdge не измерена на игровом workloads — поэтому «Не удалось проверить», а не «плохо».

---

## 5. Лицензионный анализ

| Проект | Лицензия | Можно форкнуть | Можно перенести в BrVIdge (GPL-3) | Можно перенести в форк XCL (GPL-2) |
|---|---|---|---|---|
| BrVIdge | GPL-3.0-or-later | да | да | только наш код с перелицензией в GPL-2; требуется проверка |
| fxceless/victory-fork | GPL-3.0-or-later | да | да, с сохранением notices | **нет** (GPL-3 ↔ GPL-2 несовместимы) |
| VIctory upstream | GPL-3.0-or-later | да | да | **нет** |
| XCL | GPL-2.0-only | да, форк обязан остаться GPL-2 | **нет** (без смены лицензии BrVIdge) | да |
| scarlett-layer | Apache-2.0 | да | да с notices | **нет** (Apache-2 несовместим с GPL-2 без исключения) |
| Xbox2Windows | MIT | да | да с notices | да с notices |
| AMD ISA / Mesa addrlib / silicon-rans | MIT | да | да | да |
| LLVM tests | Apache-2 WITH LLVM-exception | да | да | да (по исключению) |
| FFmpeg | LGPL-2.1+ | да | да | да |
| AnyPS5 | GPL-2.0-only | да | **нет** | да |
| VIctory BCPack code inside XCL | GPL-3, special permission | только с подтверждением разрешения | только с подтверждением разрешения | требуется юридическая проверка |

Ограничения:
- GPL-2.0-only требует, чтобы производная работа оставалась GPL-2.0-only и сохраняла авторские уведомления; `THIRD_PARTY.md` XCL должен быть сохранён.
- GPL-3.0-or-later требует GPL-3 для производной; нельзя mechanically merging with GPL-2-only code.
- Apache-2.0 с GPL-2 несовместим (патентная оговорка), но совместим с GPL-3.
- MIT/LGPL-2.1+ совместимы с обоими вариантами.
- **Юридические вопросы, требующие отдельной проверки:** персональное разрешение автора VIctory на BCPack-код внутри XCL; статус AI-assisted кода scarlett-layer (clean-room не заявлен); лицензирование наших собственных BrVIdge-вкладов при переносе в GPL-2 форк; возможные patent claims; лицензии новых зависимостей.
- Юридических гарантий не даётся.

---

## 6. Совместимость с оборудованием

Конфигурация: Ryzen 5 5600 (6c/12t, AVX2), RTX 5060 8 ГБ, 32 ГБ RAM, Windows 10 Pro 22H2, SSD + HDD.

| Требование | BrVIdge | fxceless fork | XCL | scarlett-layer | Xbox2Windows |
|---|---|---|---|---|---|
| AVX2 | да (`victory`-совместимый код; BrVIdge тесты AVX2 не используют напрямую) | требуется (README VIctory) | требуется косвенно (Rust/x64) | MSVC x64, C++23 | C++/iced_x86, AVX2 fixtures |
| Shader Model | BrVIdge: `cs_6_0` через DXC | SM 6.6 + mesh shaders | SM 6.6 + mesh shaders | SM 6.6/D3D12 (WARP fallback) | SM 6.0 fallback + DXR опционально |
| Windows 10 22H2 | да | да, но inbox D3D12Core SM 6.5 → нужна Agility SDK | нужна Agility SDK на Win10 | поддерживается Win10/11 | поддерживается, в-box WARP SM 6.0 |
| Mesh shaders | не используется | обязательны для мира | обязательны для geometry/NGG | частично | есть fallback на SM 6.0 |
| VRAM 8 ГБ | suffice для прототипа | **риск**: 12 ГБ на тестовой машине, ~24 ГБ RAM; 8 ГБ-варианты имеют 2–3 fps и device loss в issue #18/#89 | **риск**: 16 ГБ VRAM на RX 9070, ~15 ГБ commit в меню | тест на RTX 5070 + WARP; WARP снимает VRAM-риск | прямой RTX 5060 report: 408 BVH queries passed; wave64 skipped |
| RAM 32 ГБ | достаточно | **на грани**: ~24 ГБ в игре + ОС/драйвер | ~15 ГБ commit в меню, дальше больше | 78/78 synth, легко | 14 GiB cap insufficient, 32 GiB cap проходит загрузку, затем BVH exception |
| NVIDIA wave32 | частично эмулируется в BrVIdge (не проверено в играх) | issue #75/#89/#18: wave64/relay проблемы | split wave64 медленнее, wave64 VS/PS могут отличаться | NVIDIA RTX 5070 + WARP | RTX 5060: native lanes 32, forced wave64 skipped |
| Драйверы | проверено только для synth | чувствителен: 621.14 device lost, 621.40 stable | DRED/Debug layer доступны | WARP fallback | hardware table с версиями драйверов |

Практический вывод: **не переносить** результаты RTX 4070 12 ГБ на RTX 5060 8 ГБ. Для fxceless fork это критично: на 8-ГБ картах уже есть сообщения о 2–3 fps и device loss. Для XCL наша конфигурация требует wave32-пути и Agility SDK. Для Xbox2Windows есть прямые отчёты по RTX 5060 — это надёжнее экстраполяции.

Тяжёлые тесты внешних проектов не запускались: только чтение исходников и лёгкий `cargo test --workspace` BrVIdge.

---

## 7. Технические риски

1. **Правовой:** нет легального способа получить Xbox Series X|S-образ GTA VI; все демонстрации основаны на утечке. BrVIdge не должен использовать такие файлы.
2. **Лицензионный:** GPL-2/GPL-3/Apache несовместимы при объединении; ошибка приведёт к потере права распространять.
3. **VRAM/RAM:** 8 ГБ VRAM и 32 ГБ RAM могут не вместить title memory GTA VI даже при успешной архитектуре.
4. **wave32:** консольный wave64 на NVIDIA требует эмуляции; часть шейдеров может отличаться.
5. **DXC/память:** большие translated shaders требуют гейтинга (XCL уже имеет `XCL_HUGE_PSO_*`).
6. **D3D12.X:** полный формат графических команд до сих пор не реконструирован ни одним проектом.
7. **Стабильность:** device loss/TDR на NVIDIA отмечены у VIctory fork, XCL и Xbox2Windows.
8. **Upstream churn:** XCL и Xbox2Windows очень активны; форк потребует постоянного merge.
9. **AI-assisted код:** scarlett-layer прямо не заявляет clean-room; крупные changes в victory-fork сделаны с Claude Code; требуется review.

---

## 8. Сравнение четырёх стратегий

| Вариант | Объём работы | Технические риски | Преимущества | Недостатки | Перспективы | Поддержка |
|---|---|---|---|---|---|---|
| **A. Продолжать BrVIdge (Phase 7)** | Очень большой: ~весь GPU/API/память с нуля | Высокие: нет реальных тестов, legal blocker | Полный контроль, Rust, GPL-3, свои доки/тесты | Отставание от XCL на годы; 81 synth тест против 934; нет GTA VI | Низкая в одиночку | Простая, но бесполезная без результата |
| **B. Форк fxceless/VIctory** | Средний: форк + адаптация под RTX 5060 8 ГБ | Высокие: 2–4 fps, 24 ГБ, тени, драйверы, GPU-specific | Единственный реальный GTA VI world render; GPL-3 совместим с BrVIdge | Узкая привязка к тайтлу/GPU; 32 issue; много `shortcut:` | Средняя для GTA VI, низкая для платформы | Трудно: upstream + локальные флаги |
| **C. Форк xcl-dev/xcl** | Средний: форк + GTA VI-слой clean-room | Средние: wave32, VRAM, GPL-2-only, BCPack permission | Лучшая архитектура, 934 теста, реальные игры, полный GPU/API | GTA VI gameplay не достигнут; нельзя merge GPL-3 код | **Высокая** | Требует upstream sync |
| **D. Гибрид из нескольких проектов** | Очень большой: интеграция C++/Rust/MIT/Apache | Высокие: несовместимость лицензий, интеграция | Можно взять MIT static recompiler и Apache shims | Теряется зрелость XCL; годами | Средняя | Сложная |

---

## 9. Рекомендуемая стратегия

**Вариант C: создать отдельный форк `xcl-dev/xcl` как основной репозиторий.**

Обоснование:

1. XCL — единственный проект с одновременно зрелой архитектурой **и** подтверждёнными тестами на реальных играх (`e2e.rs`, `gpu_pso_e2e.rs`, `console_dlls_e2e.rs`): 934 теста против 81 у BrVIdge.
2. GTA VI-специфичный слой всё равно придётся разрабатывать, но в XCL уже есть все foundation-компоненты: loader, resolver, UMA, D3D12.X, shader recompiler, XGameRuntime, audio/input, VFS, BCPack, текстуры.
3. Лицензионно форк XCL допустим при сохранении GPL-2.0-only и `THIRD_PARTY.md`. Наши собственные модули BrVIdge (`brv_config`, `brv_log`, `brv_diag`, тесты) могут быть перелицензированы в GPL-2, так как являются нашим кодом; GPL-3-код VIctory/форка переносить нельзя — только идеи и факты.
4. Вариант B заманчив из-за GTA VI, но это не архитектурная база: 2,5–4 fps на более сильном GPU, ~24 ГБ RAM, чёрные тени, 32 открытых issue, много title/GPU-specific `shortcut:`. По правилу «не считать наиболее продвинутый демо проект лучшей основой» он проигрывает.
5. Вариант D отдалённо эквивалентен C, но с потерей зрелого GPU-кода XCL; разумно использовать его как источник **отдельных** компонентов (MIT `Xbox2Windows` static recompiler tools, Apache `scarlett-layer` WARP-тесты), а не как основу.
6. Вариант A нерационален: это повторение пути XCL с нуля без легальных тестовых материалов.

---

## 10. План дальнейшей разработки

1. **Upstream и основной репозиторий:** `xcl-dev/xcl`, commit `e9e94ce`, новый fork, например `BrVIdge/XCL-BrVIdge`, лицензия GPL-2.0-only.
2. **Что сохранить из BrVIdge:** документация (`README/RESEARCH/ARCHITECTURE/FEASIBILITY/RISKS`), `docs/reports/`, модули `brv_config/brv_log/brv_diag` и synth-тесты как отдельные GPL-2-contributions; `brv_formats` переписать с MIT-таблиц AMD/Mesa, а не переносить как есть, если нужна GPL-2-совместимость.
3. **Что не переносить:** GPL-3-код VIctory/fork, Apache-2 scarlett-layer, `brv_gpu::console_list` (собственный формат), упрощённый tiling, 11-инструкционный translator — вместо них использовать соответствующие подсистемы XCL.
4. **Отдельно развивать:** GTA VI build identities/hooks (аналог `victory-game`) clean-room; NVIDIA wave32 path; бюджет VRAM/RAM под 8 ГБ; легальный synthetic harness; RH/DRED diagnostics.
5. **Структура будущего репозитория:** сохранить `crates/xcl` upstream layout; добавить `crates/gta6/` (идентификация билда, хуки), `crates/nvidia/` (wave32/VRAM), `docs/` (BrVIdge research archive), `tests/synthetic/`; поддержка `xtask`-аналога.
6. **Новая дорожная карта:**
   - P0: fork + legal review + CI (cargo test) на текущем железе.
   - P1: hardware baseline (RTX 5060, wave32, Agility SDK, DXC).
   - P2: GTA VI synthetic fixtures и build identity, без игровых файлов.
   - P3: port GPU/memory budget optimizations under GPL-2.
   - P4: integration и только затем попытка легального title-теста.
7. **Phase 7 BrVIdge:** оставить приостановленной; не удалять код; использовать как архив идей.

---

## 11. Список источников и проверенных коммитов

| Проект | Коммит | Дата | Источник |
|---|---|---|---|
| BrVIdge | нет `.git`; состояние 25.11.2026 | 2026-11-25 | `G:\Projects\Opencode\BrVIdge`, `cargo test` 81 passed |
| fxceless/victory-fork | `4e17478b9c27bb530643e5b0e16a767cf4419e08` | 2026-11-24 15:45 -0500 | https://github.com/fxceless/victory-fork |
| xcl-dev/xcl | `e9e94ce665d16bb0e2d2e4cb6b37e149f0d27721` | 2026-11-24 22:00 +0100 | https://github.com/xcl-dev/xcl |
| Marvisak/VIctory (upstream) | `ffe310eb2098e52169f4c44db5ffdabf43a72622` | 2026-11-24 10:53 +0100 | https://github.com/Marvisak/VIctory |
| jadeowaowa/scarlett-layer | `0e436b3e1c654765b193c1cb705918fbf101b10b` | 2026-11-23 19:30 -0500 | https://github.com/jadeowaowa/scarlett-layer |
| R0Y00/Xbox2Windows | `9881fef010d7275d6a3a9d3a5802b1c50b8d4646` | 2026-11-25 05:20 +0800 | https://github.com/R0Y00/Xbox2Windows |

Дополнительные источники: README/LICENSE/THIRD_PARTY.md каждого репозитория; `docs/specs/2026-11-20-xcl-design.md`; `docs/SPEC_SOURCES.md` scarlett-layer; issue `#870` Xbox2Windows; issues victory-fork `#13/#18/#75/#89`; `docs/reports/phase6.md`, `phase7.md`, `xcl-comparison.md` BrVIdge.

Все репозитории клонировались только для чтения; исходники и конфигурации не изменялись; исполняемые файлы проектов не запускались; игровые сборки не загружались.

---

## 12. Неизвестные и неподтверждённые моменты

- Реальные метрики BrVIdge на игровых workloads отсутствуют.
- Сравнение `fxceless/victory-fork` с upstream сделано по рабочим деревьям (shallow clones), а не по полной истории коммитов; детальный per-commit diff не проверялся.
- Claim форка «исправления написаны заново по XCL» — заявление автора, не независимо подтверждено.
- 496 тестов на RX 7600 в issue #29 victory-fork — сообщение пользователя, не Independent CI.
- 78/78 synthetic checks scarlett-layer подтверждены README/CI-бейджами, но не запускались в этом аудите.
- GTA VI gameplay не достигнут ни в одном проекте; лёгкость перехода от меню к gameplay не оценена.
- Точная совместимость лицензий BCPack от VIctory внутри XCL требует юридической проверки.
- Возможность получить легальную копию GTA VI для интеграционных тестов не установлена.

---

## FINAL DECISION

**Продолжать BrVIdge как основную реализацию? Нет.**

**Форкнуть существующий проект? Да — `xcl-dev/xcl`.**

**Почему:** XCL имеет наиболее зрелую и расширяемую архитектуру, 934 теста, реально работающие подсистемы CPU/GPU/API/памяти и подтверждённую совместимость с несколькими играми. BrVIdge — исследовательский прототип без игровых тестов; fxceless VIctory fork — лучший GTA VI-демо, но узкоспециализированный, медленный и нестабильный; scarlett-layer и Xbox2Windows — ценные источники компонентов, но не основа.

**Что делать дальше:**

1. Создать fork `xcl-dev/xcl` (GPL-2.0-only, commit `e9e94ce`), сохранить `THIRD_PARTY.md` и provenance.
2. Провести юридическую проверку лицензий до любого объединения кода (GPL-2 vs GPL-3 vs Apache-2).
3. Перенести в форк только наш собственный код/документацию BrVIdge, перелицензировав наши вклады в GPL-2; не переносить GPL-3-код VIctory.
4. Реализовать GTA VI-слой clean-room и адаптацию под RTX 5060 8 ГБ / wave32 / Windows 10 22H2 + Agility SDK.
5. Оставить BrVIdge как исследовательскую ветку, архив архитектурных решений и набор синтетических тестов; не удалять и не переписывать текущий код без отдельного решения.
6. Не запускать реализацию Phase 7 BrVIdge до принятия этого решения пользователем.
