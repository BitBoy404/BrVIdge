# Отчёт о завершении Phase 6 (CPU Backend)

Дата проверки: 24 ноября 2026 года.

## 1. Проверка по исходному коду

- `crates/brv_exec/src/lib.rs` содержит реальные вызовы `VirtualAlloc`,
  `VirtualFree`, `VirtualProtect`, `CreateThread`, `SetThreadPriority`,
  `SetThreadAffinityMask`, `WaitForSingleObject`, `GetExitCodeThread`.
- `map_image()` копирует заголовки и секции в выделенную область, применяет
  `DIR64`/`HIGHLOW`-релокации и меняет защиту страниц на `PAGE_EXECUTE_READ`.
- `run_entry()` и `run_entry_in_thread()` получают указатель функции из
  `base + entry_rva` и выполняют гостевой x86-64 код процессором хоста.
  Эмуляции инструкций нет.
- `crates/brv_pe/src/lib.rs::synthetic::build_native_pe32plus()` создаёт
  собственный PE32+ с инструкцией `mov eax, imm32; ret` и таблицей релокаций.
- `capture_crash()` формирует `module + 0xRVA` и стек через `std::backtrace`.
- `unsafe` изолирован в `brv_gpu` и `brv_exec`; в остальных crates он запрещён.

## 2. Результаты тестов

```
cargo test --workspace   → все 59 тестов пройдено, 0 failed
cargo clippy --workspace --all-targets -- -D warnings → без предупреждений
cargo xtask policy       → нарушений нет
brv native               → entry point вернул 42, релок DIR64 применён
brv native --thread --exit-code 17 → поток вернул 17
brv run                  → код 0, 3 API-вызова в data/logs/brv.log и data/report.json
```

## 3. Границы подтверждения

Подтверждено тестами: PE32+-отображение, релокации, нативный запуск, потоки
с приоритетом и affinity, отсутствие дедлоков, формат crash-отчёта,
наличие вызовов в журнале и JSON-отчёте.

Не подтверждено и не заявлено как реализованное: TLS-колбэки, delay-load,
реальный VEH/SEH-перехват, загрузка настоящих игровых PE, совместимость с GTA VI.

Вывод: ворота Phase 6 пройдены в объёме синтетического прототипа.
