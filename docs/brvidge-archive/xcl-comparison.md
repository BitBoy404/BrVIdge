# Архитектурное сравнение BrVIdge и XCL

Дата: 25 ноября 2026 года. Цель: решить, продолжать BrVIdge с нуля или
создать отдельный форк XCL.

## 1. Источники и метод

- XCL: `https://github.com/xcl-dev/xcl`, клон только для чтения в
  `H:\Temp\opencode\xcl-dev-src`, HEAD `e9e94ce665d16bb0e2d2e4cb6b37e149f0d27721`
  (2026-11-24 22:00 +0100). Использованы реальные файлы, а не только README.
- BrVIdge: `G:\Projects\Opencode\BrVIdge`, HEAD рабочего состояния — код
  phases 4–7 (81 тест). Код XCL в BrVIdge **не копировался**, репозиторий XCL
  **не изменялся**.
- Ссылки на XCL ниже даны на зафиксированный commit, чтобы утверждения можно
  было проверить: `https://github.com/xcl-dev/xcl/blob/e9e94ce.../<путь>`.

## 2. Подтверждённые факты по XCL (не из README только)

| Область | Файл / символ | Что реально есть |
|---|---|---|
| Архитектура | `crates/xcl/src/lib.rs` | модули `loader`, `pe`, `resolver`, `hle`, `gpu`, `vfs`, `host`, `report`, `rpf`, `disasm` и др.; один package `crates/xcl` + `tests/testtitle`, `tests/testgamedll` (`Cargo.toml`) |
| CPU | `loader.rs:60 map_image`, `:64 map_image_ex`, `:337 run_main`, `:248 run_tls_callbacks` | собственный PE loader, TLS, entry point в отдельном потоке; x64 выполняется нативно, без эмуляции |
| PE/импорты | `loader.rs`, `resolver.rs`, `stubs.rs`, `data/stubdb.toml` | relocations, imports по имени/ordinal/delay-load, 5 уровней resolver, auto-stub |
| Память | `gpu/uma.rs`: `MAX_SIZE = (4096 << 20) - GRAIN`, `WINDOW = 1<<32`, `Uma::new`, `OpenExistingHeapFromAddress` | до 4 окон по 4 GiB, общая CPU/GPU память без копий, first-fit allocator |
| HLE | `hle/kernelx.rs exports()` — 60 экспортов; `hle/xgameruntime/mod.rs exports()` — 127 экспортов; `hle/xgameruntime/gdk_table.rs` — 396 строк GDK | offline XUser/XGameSave/XPackage/XSystem, thunk-протокол, task queue |
| ФС | `vfs.rs`, `vfs_hooks.rs` (CreateFileW и др.), `config.rs` | `GAME:`, `G:`, `D:`, `S:`, `T:`, overlay, whiteouts |
| GPU-команды | `hle/d3d12x/console_list.rs`: `PIPE_MARKER 0x8C0F_E002`, `DISPATCH 0x4002_C400`, `LIST_SLOTS = 600`, `close()` | интерпретация compute-записей консольного списка, root arguments по RTS0 |
| GPU-бэкенд | `hle/d3d12x/real.rs`, `list_x.rs`, `device_x.rs`, `present.rs`, `guard.rs` | расширенные vtable, split по caller address, `PresentX` = слот 30 очереди |
| RDNA2 → DXIL | `gpu/rdna2.rs`, `gpu/rdna2_spec.rs` (AMD ISA, MIT), `gpu/recomp/mod.rs`: `translate`, `translate_mesh`, `translate_geometry`; `gpu/dxil/mod.rs` | 23 812 инструкций LLVM-тестов, все стадии VS/PS/CS/MS/GS, HLSL+DXC и прямой DXIL бэкенд |
| Текстуры/BCPack | `gpu/tiling.rs`: 14 2D-режимов (`MODES`), `detile`, `detile_level`, `detile_volume`, `image_layout`; `hle/bcpack/mod.rs`: `decode(mode, src, out_len)` | BC1/3/4/5/6H/7, mip tail, точное совпадение 909/912 текстур по данным README |
| RT/звук/ввод | `gpu/bvh.rs`, `hle/xaudio2.rs`, `hle/xma2/`, `hle/opus.rs`, `hle/gameinput.rs`, `hle/kbm.rs` | BVH в формате RDNA2, XMA/Opus, GameInput + KBM |
| Тесты | `crates/xcl/tests/`: 16 integration-файлов; статически 934 `#[test]`, 42 `#[ignore]`; `gpu_pso_e2e.rs::retail_vs_ps_through_the_pso_hook_render_and_read_back`, `e2e.rs::test_title_runs_and_game_folder_is_untouched`, `console_dlls_e2e.rs::bink2_console_dll_decodes_a_game_movie` | тесты на реальных DLL/шейдерах включаются через `XCL_SAMPLE_*`; GPU-тесты печатают `SKIPPED` явно |
| Ресурсы | `README.md`: «fits a 32 GB machine (~15 GB committed in the menus)», `XCL_HUGE_PSO_BYTES`, `XCL_HUGE_PSO_CONCURRENCY`, гейт на 3 GB commit | реальная забота о памяти и «штормах» компиляции PSO |

## 3. Подтверждённые факты по BrVIdge

| Область | Файл / символ | Что реально есть |
|---|---|---|
| Архитектура | `Cargo.toml`: 14 crates, edition 2024; `unsafe` только в `brv_gpu` и `brv_exec` | модульная структура, строгие lints, русские комментарии |
| CPU | `brv_exec::map_image`, `run_entry`, `run_entry_in_thread`, `capture_crash` | реальное PE-отображение через Win32, релоки `DIR64`/`HIGHLOW`, нативный запуск, потоки с priority/affinity |
| PE | `brv_pe::PeHeaders::parse`, `imports`, `rva_to_offset`, `synthetic::build_minimal_pe32plus`, `build_native_pe32plus` | YAML-подобных фикстур нет; образы генерируются кодом |
| Память | `brv_config::ConsoleProfile::series_x/series_s`, `brv_exec` использует `VirtualAlloc` только под PE; UMA guest-модели **нет** | бюджет 4096 MiB в конфиге — пока только число, не реальная модель |
| HLE | `brv_hle::Registry::with_builtin`, `HostPolicy`, `census`; `brv_runtime::run_title`, `SyntheticTitle` | 3 HLE-функции в in-process title, журнал вызовов; XGameRuntime offline **не реализован** |
| ФС | `brv_diag::check_dir` | проверка активов репозитория, а не виртуальные диски игры |
| GPU | `brv_gpu::render_triangle`, `Presentation::new/present_clear`, `adapter_info` | оффскрин-треугольник и отдельный windowed swapchain; PresentX **не реализован** |
| Команды | `brv_gpu::console_list::interpret`, 9 тегов | собственный формат записей, **не** D3D12.X |
| Менеджеры | `brv_gpu::manager::{PipelineCache, DescriptorAllocator}` | кэш по FNV и allocator индексов; реальные PSO не кэшируются в D3D12 |
| RDNA2 → DXIL | `brv_shader::decode_u32` (11 инструкций), `emit_compute_hlsl`, `brv_dxc::compile_rdna2_subset` | один compute-shader, валидирован DXC |
| Кэш | `brv_shader_cache::ShaderCache::{get,put,key}` | ключ = версия + содержимое; работает на файлах |
| Текстуры | `brv_formats::tiling::tile_8bpp/detile_8bpp`, `bc1::decode_block`, `bcpack::decode` | собственная 256B-модель, полный BC1; BCPack явно не реализован |
| Тесты | `cargo xtask all`: 81 тест, 0 ignored; `brv run` пишет `data/report.json` | всё на синтетических данных; код явно помечает нереализованное |

## 4. Сравнительная таблица

| Область | Что в BrVIdge уже реализовано | Что в XCL уже реализовано | Что в BrVIdge лучше | Что в XCL лучше | Можно переиспользовать при соблюдении лицензии | Придётся разрабатывать самостоятельно |
|---|---|---|---|---|---|---|
| **Архитектура** | 14 std-only crates, edition 2024, `unsafe` изолирован в 2 crates, `xtask`, русские доки | Один package `crates/xcl` + test fixtures; 163 `.rs`, ~121k строк; архитектура описана в `docs/specs/2026-11-20-xcl-design.md` | Модульная гигиена, явные статусы, строгие lints, asset-policy | Зрелость, объём, проверенность на реальных играх | Собственную структуру crates и документацию BrVIdge можно перенести в форк как свои GPL-3→GPL-2 материалы (при желании) | Организацию upstream-синхронизации форка |
| **CPU / x64** | `brv_exec::map_image`, `run_entry`, `run_entry_in_thread`, `capture_crash`; нативный запуск, релоки | `loader.rs::map_image_ex/run_main/run_tls_callbacks`, `resolver.rs`, `stubs.rs`; нативный запуск + TLS + SEH + delay-load | Тесты на настоящем machine-коде в PE, детерминированные фикстуры | TLS, C++-исключения, delay-load, 16 консольных DLL, crash reporter игры | Идеи и тестовую фикстуру `build_native_pe32plus` (собственный код BrVIdge) | Интеграцию с реальными TLS/SEH GTA VI в форке |
| **PE Loader и память** | `brv_pe::PeHeaders::parse/imports/rva_to_offset`, `brv_loader::prepare`; UMA **нет** | `loader.rs`, `gpu/uma.rs` (4095 MiB, 4×4 GiB окна, `OpenExistingHeapFromAddress`), `kernelx` физические страницы, VFS overlay | Чистый std-only парсер, понятные ошибки | Реальная модель памяти консоли и zero-copy CPU/GPU, overlay ФС | MIT-таблицы AMD/Mesa, идеи UMA из XCL-документации (не код GPL-2) в BrVIdge; в форке — весь код XCL | UMA, бюджет 8 ГБ VRAM под RTX 5060, physical pages, `GAME:`/`G:`/`D:`/`S:`/`T:` |
| **GPU / RDNA2 → DXIL** | `render_triangle`, `Presentation`, `console_list::interpret`, 11 инструкций, `compile_rdna2_subset`, DXC-валидация | `gpu/recomp/*` (VS/PS/CS/MS/GS, atomics, LDS, wave ops, RT), `gpu/dxil/*`, 26 698/26 698 шейдеров, 0 ошибок DXC | Честная маркировка «собственный формат списка», версионированный кэш | Полноценный транслятор, 100% coverage, particle GPU-тесты, прямой DXIL | В BrVIdge — только MIT-факты AMD ISA; в GPL-2 форке — весь `gpu/recomp/` и `gpu/dxil/` (с provenance AnyPS5 GPL-2) | Полный набор инструкций, wave64-on-wave32, 64-bit atomics, D3D12.X graphic records |
| **Системные Xbox API** | `brv_hle::Registry`, `HostPolicy`, `census`, 3 HLE-функции в `brv_runtime` | 60 экспортов kernelx, 127 XGameRuntime + 396 GDK-слотов, XAudio2/Opus/XMA, GameInput, DirectStorage, netoff | Явный отчёт о неподдерживаемых API | Реальные offline-сервисы, чья работа проверена на играх | В BrVIdge — архитектурные идеи и MIT libHttpClient-контракт; в форке — весь `hle/` XCL | XUser/XGameSave/XPackage для GTA VI, audio/input, сетевые offline-заглушки |
| **Runtime / ресурсы** | `brv_config`, `brv_log`, `brv_diag`, `brv_runtime`, `data/report.json`; PSO-кэш и allocator как прототипы | `host.rs`, `launcher.rs`, `report.rs`, Job Object memory cap, `XCL_HUGE_PSO_*`, `XCL_DSTORAGE_*`, frame dumps | Понятная конфигурация и честная диагностика | Управление commit/paging, PSO concurrency, DRED, подробный frame pacing | Свои конфиг/логгер/диагностику можно перенести в форк как GPL-2 совместимый вклад | Адаптация под 6 ядер и 8 ГБ VRAM, gating DXC |
| **Тестовая инфраструктура** | 81 тест, `cargo xtask all`, `brv selftest`, `asset-policy`, `SKIPPED` только у GPU-теста | 934 `#[test]`, 16 integration-файлов, `XCL_SAMPLE_*`, GPU-тесты, real-DLL oracle, `scripts/test.ps1` | Ноль тестов требует игровых файлов; policy проверяет репозиторий | Глубина и покрытие реальными данными | Тестовые подходы (`XCL_SAMPLE_*`) и собственные фикстуры BrVIdge | Легальные синтетические тесты для GTA VI без утечки |
| **Производительность** | Прототип: один кадр, 64×64, readback; 2 кадра SwapChain | 30 fps в меню ритейл-игры, ~15 ГБ commit, 26k шейдеров, million draws без пропусков; узкие места: DXC superlinear, wave64 на NVIDIA, VRAM | Нет; ограничения VRAM ещё не измерены | Измеренные ограничения и обходные пути (`XCL_HUGE_PSO_*`, split wave64) | Данные о поведении драйвера и флаги XCL как спецификация | Профилирование на RTX 5060 8 ГБ, бюджет VRAM, wave32-путь |
| **Лицензии** | GPL-3.0-or-later; зависимости MIT/Apache-2.0/windows | GPL-2.0-only; third-party: AnyPS5 GPL-2, FFmpeg LGPL-2.1+, AMD/Mesa/silicon-rans MIT, LLVM Apache-with-exception, XCL→VIctory BCPack по персональному разрешению | GPL-3 совместим с MIT/BSD/LGPL-2.1+/Apache-exception | GPL-2 форк получает весь код XCL и AnyPS5-логику | В BrVIdge: только MIT/BSD/LGPL-2.1+/Apache-exception куски и идеи; в GPL-2 форке: весь XCL + AnyPS5 + LGPL-2.1 FFmpeg | Юридическая проверка персонального разрешения на BCPack от VIctory в форке |

## 5. Рекомендация

**Рекомендую отдельный форк XCL как основу для GTA VI, а BrVIdge оставить
исследовательским полигоном и источником архитектурной документации.**
Причины:

1. **Принципиальная проверенность.** XCL уже запускает игры целиком
   (`e2e.rs::test_title_runs_and_game_folder_is_untouched`), рисует их через
   переведённые PSO (`gpu_pso_e2e.rs::retail_vs_ps_through_the_pso_hook_render_and_read_back`),
   запускает консольные DLL (`console_dlls_e2e.rs::bink2_console_dll_decodes_a_game_movie`).
   У BrVIdge нет ни одного реального игрового теста, и по условиям проекта его
   нельзя получить легально.
2. **Стоимость догоняющего развития.** BrVIdge имеет 81 синтетический тест
   против 934 тестов XCL; разрыв по GPU/API/памяти измеряется годами работы.
3. **Лицензионная совместимость.** BrVIdge GPL-3.0-or-later не может включать
   GPL-2-only код XCL/AnyPS5. Форк XCL остаётся GPL-2.0-only и получает всю
   кодовую базу законно, с сохранением `THIRD_PARTY.md` и provenance-заметок.
4. **Целевая игра.** GTA VI-специфичный слой (аналог `victory-game`) всё равно
   придётся писать; в форке XCL для этого уже есть PE loader, HLE, shader
   recompiler и D3D12.X, а в BrVIdge — только их зачатки.
5. **Ограничения «железа» пользователя.** RTX 5060 (8 ГБ VRAM, wave32) и
   Ryzen 5 5600 (6 ядер) требуют доработки XCL, но это локальные задачи
   (wave64-on-wave32, бюджет UMA, PSO gating) на готовом foundation, а не
   переписывание с нуля.

Условия форка:

- не использовать утечки GTA VI, ключи и образы; тесты только синтетические
  или легально предоставленные пользователем;
- сохранить GPL-2.0-only, авторские уведомления и `THIRD_PARTY.md`;
- отдельно подтвердить статус BCPack-кода, переданного в XCL от автора VIctory
  по персональному разрешению (2026-11-24), или заменить его clean-room;
- синхронизироваться с upstream `xcl-dev/xcl`, не выполняя массовый merge в BrVIdge;
- перенести в форк только те наработки BrVIdge, которые являются нашим
  собственным кодом/документацией (конфиг, логгер, диагностика, policy-тесты).

**Итог:** продолжать BrVIdge с нуля невыгодно. Форк XCL даёт реально
проверенную основу; BrVIdge сохраняет ценность как исследование и как источник
компонентов, которые можно законно перенести в форк.


