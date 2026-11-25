# Исследование существующих проектов

Дата исследования: 24 ноября 2026 года.
Статус: Phase 1 (Research). Документ описывает **только проверенные** факты
и явно помечает гипотезы.

---

## 0. Методология и источники

### Что изучалось

1. **Первичные исходники** (клонированы локально для анализа):
   - `github.com/Marvisak/VIctory` — commit `ffe310e` от 2026-11-24;
   - `github.com/ivall/xcl` — commit `c160d86` от 2026-11-24;
   - `github.com/boykopovar/AnyPS5` (для сравнения);
   - `github.com/xenia-project/xenia` — README, LICENSE, docs/building.md;
   - `github.com/WinDurango/WinDurango` — README.
2. **Собственная документация проектов**: README, `docs/architecture.md`,
   `HANDOFF.md`, `THIRD_PARTY.md`, `docs/specs/`, `Cargo.toml`, `LICENSE`.
3. **Публичные материалы о CGBJ** (код закрыт): PortWatchVI, XDA, DSOGaming
   (только заголовок/пересказ), fan-трекиеры.
4. **Открытые спецификации**: Wikipedia (Xbox Series X/S, GTA VI), сайт
   Rockstar, AMD Machine-Readable ISA, Mesa addrlib, LLVM/DXC (по упоминаниям
   в репозиториях).
5. **Игровые данные не использовались**. Ни один файл GTA VI не скачивался,
   не анализировался и не хранится.

### Правила

- Если утверждение есть только в README, но нет в коде — это помечено как
  «заявление проекта», а не «проверено в коде».
- Для закрытых проектов (CGBJ) изучаются только публично доступные сообщения.
- Никакие возможности не приписываются проектам «по умолчанию».

---

## 1. VIctory

### 1.1 Общая информация

| Параметр | Значение |
|---|---|
| Репозиторий | `github.com/Marvisak/VIctory` |
| Автор | Marvisak |
| Язык | Rust, edition 2024, `rust-version = 1.88` (`Cargo.toml`) |
| Лицензия | GPL-3.0-or-later (`LICENSE`, `Cargo.toml`) |
| Тип | слой совместимости Xbox Series X|S → Windows для GTA VI |
| Создан | 23 ноября 2026 (по данным fan-трекера; не подтверждено кодом) |
| Статус по README (23.11.2026) | **Not playable yet** |
| Тестовая машина разработчика | Windows 11, AMD RX 6700 XT |

### 1.2 Архитектура

Из `docs/architecture.md` и `Cargo.toml` (workspace: `crates/*`, `apps/*`,
`xtask`, отдельно `tests/fixtures`):

| Crate | Роль |
|---|---|
| `victory-core` | конфигурация, asset-policy guard, логирование |
| `victory-pe` | только чтение PE32+ и import-library analysis |
| `victory-loader` | подготовка, отображение, связывание и запуск образа |
| `victory-hle` | реализации консольных API, политика pass-through на хост |
| `victory-gpu` | объекты D3D12.X, списки команд, гостевая память GPU → host D3D12 |
| `victory-shader` | консольные контейнеры шейдеров → хост-шейдеры (DXIL) |
| `victory-formats` | раскладки текстур GPU (sizes, tiling, detiling), аудиокодеки; BCPack — plan |
| `victory-pipeline` | trait конвертера, контент-адресуемый кэш |
| `victory-game` | специфичные для GTA VI форматы, identity сборок, хуки |
| `apps/victory` | CLI |
| `xtask` | asset-policy check, git hooks |
| `tests/fixtures/hle-probe` | собственная подставная «игра» для тестов loader |

Всё, кроме `victory-game`, позиционируется как platform-level и тестируемо
с любой игрой Xbox Series (по утверждению документации).

Поток (по `docs/architecture.md`):

```text
файлы пользователя → victory-pipeline/victory-game → cache_dir
исполняемый файл → victory-loader → код игры (native x86-64)
                         ↓ imports
                    victory-hle → host Windows
                         ↓ D3D12.X
                    victory-gpu → host D3D12
                         ↓ shaders at pipeline creation
                    victory-shader ↔ cache_dir
```

### 1.3 Подход к исполнению CPU-кода

**Эмуляции CPU нет.** Код wine/Proton-like запускается внутри процесса Windows.

Import resolution (порядок зафиксирован в `docs/architecture.md`):

1. реализация в `victory-hle::modules` (всегда wins);
2. собственная функция хоста, если `HostPolicy` разрешает DLL;
3. экспорт DLL игры, лежащей рядом с исполняемым файлом;
4. stub, называющий импорт: `--unimplemented trap` (breakpoint + abort)
   или `zero` (log один раз, вернуть 0).

Дополнительно:

- `LoadLibrary*` консольной DLL возвращает pseudo-handle; `GetProcAddress`
  находит функции hle; `LoadLibrary*` DLL игры отображает её образ.
- `QueryApiImpl` выдаёт runtime-объекты по provider + interface id;
  online-области отвечают как offline-консоль.
- Static TLS: exe получает слот 0; при отображении как DLL — другой слот,
  loader перенаправляет обращения (`victory-loader::tls`).
- Crash reporter (`victory-loader::crash`) называет сбой как `module+rva`
  и разворачивает стек; логи пишутся в `crashes.log`.
- Команда `victory census` классифицирует импорты **без запуска** кода.

### 1.4 Подход к GPU-командам

- `victory-gpu` отображает консольные объекты D3D12.X на host D3D12.
- По README: draw/compute/present на host D3D12; в хостовую память
  копируются **только изменившиеся** области title memory.
- Мир использует indirect draws; arguments ограничиваются (clamped).
- Конвейеры создаются worker-потоками и сохраняются на диске.
- Дескрипторные lifetimes и shader-written buffers перечислены как
  проблемные области.
- Есть диагностический режим `VICTORY_D3D12_WARP=1` (software adapter),
  потому что вход в мир на реальном GPU может вызвать сброс драйвера.
- DRED/GBV/debug layer доступны в `diag`-сборке.

**Графический формат списков команд D3D12.X в коде реконструирован частично**
(судя по открытым описаниям и оставшимся ограничениям); здесь не делается
утверждение о полной поддержке.

### 1.5 Системные API

- `victory-hle::modules`: один вошедший offline-пользователь, лицензии,
  пакеты, сейвы (сохраняются при kill на любом этапе), текстовый диалог.
- Speech в диалоге — silent; диалог и перезапуск не проверены вживую.
- Сеть: offline by design, игра не имеет доступа к сети.
- Файлы: `victory-hle::modules::files` перехватывает все path-taking imports
  kernel32/C CRT, переписывает пути виртуального диска на папку хоста;
  persistent local storage — отдельный диск над папкой в `VICTORY_DATA_DIR`.
- Ввод: XInput-геймпады, клавиатура, клавиатура как замена геймпада;
  вибрация не работает.
- Аудио: меню, видео, загрузка; XMA-голоса и Opus-потоки декодируются;
  реверб silent, Opus не сверен с игрой.
- Неподдерживаемые API → stub с именем (trap/zero), что важно для diagnostics.

### 1.6 Память

- Работа с «title memory» игры; копируются только изменённые страницы.
- Для запуска требуется несколько ГБ доступного Windows commit;
  при Windows error 1455 README рекомендует закрыть программы или
  увеличить page file.
- Точная модель guest address space в открытой документации не описана —
  здесь не выдумывается.

### 1.7 Трансляция шейдеров

По `docs/architecture.md`:

- `victory_shader::ir::build`: нативная программа → SSA, одна machine
  instruction на инструкцию IR.
- `victory_shader::structure`: убирает EXEC — lane masks становятся
  per-lane boolean; wave-wide проверки масок → per-lane branches; vector-инструкции
  сохраняют guard только там, где lane может быть с сброшенным EXEC.
- `victory_shader::resource`: трассировка каждого дескриптора и scalar memory
  address до root parameter (table, heap, root descriptor, constant).
- Waterfall-цикл читает дескриптор из first lane (`v_readfirstlane_b32`);
  транслятор прослеживает его до per-lane load/choice и использует
  non-uniform heap index.
- Трансляция происходит **в момент создания PSO**, из байтов, которые
  передала игра (exe, archive, memory); AOT-конвертация — только разогрев кэша.
- CLI: `victory inspect`, `census`, `lib-exports`, `shader-info`,
  `translate [--validate] [--pipelines]`, `run`.

Статистика из README (23.11.2026):

| Категория | Переведено |
|---|---|
| Меню | все |
| Мир: mesh | 8 191 / 8 506 |
| Мир: vertex/pixel | 10 617 / 12 346 |
| Мир: compute | 1 643 / 2 539 |

Не переведены: 64-bit image atomics, merged geometry stages, шейдеры
ray-tracing builder; часть compute требует GPU с 64-lane waves.

### 1.8 Поддерживаемые форматы

- PE32+ и import libraries (`victory-pe`, `victory lib-exports`).
- Консольные контейнеры шейдеров, SGD5-файлы (`victory shader-info`,
  `victory translate`; по `docs/architecture.md`).
- Тайлинг текстур и detiling для режимов, используемых игрой.
- zlib и BCPack — parallel decode (BCPack в `victory-formats` помечен как planned
  по архитектурному документу; README утверждает decoded — расхождение
  документации, фиксируем как неопределённость).
- Аудио: XMA, Opus.
- XVC/MSIXVC не расшифровываются; игра запускается из файлов пользователя.

### 1.9 Ограничения (по README)

- Boot/меню/видео/загрузка работают; мир «reached and runs», но сцена
  после титр тёмная; 0.5–1.8 с/кадр.
- Вход в мир на реальном GPU может повесить его → сброс драйвера Windows.
- Frame pacing дрейфует на одном экране в release-сборке.
- Нет: real queries, depth bounds, HDR, scaling, reverb, rumble,
  speech в диалоге.
- Series S: проходит меню до экрана загрузки, мир не работает.
- Тестировалось на одной машине (RX 6700 XT); воспроизводимость сторонними
  не подтверждена.

### 1.10 Лицензия и повторное использование

- GPL-3.0-or-later. Исходники доступны, но прямое копирование в BrVIdge
  (GPL-3-or-later) возможно только с соблюдением GPL-3 и авторских прав;
  BrVIdge выбирает clean-room approach.
- Внешние crates из `Cargo.toml`: `anyhow`, `thiserror`, `clap`, `tracing`,
  `tracing-subscriber`, `serde`, `toml`, `object`, `windows-sys`, `windows`,
  `blake3`, `memchr`, `miniz_oxide`, `walkdir`, `tempfile`, `opus-pure`
  (BSD-3-Clause) и др.
- Зависимости собраны с lints: `unsafe_op_in_unsafe_fn = deny`,
  `unused_must_use = deny`, clippy warnings.

---

### 1.11 Уточнения по реальному коду (аудит исходников)

Дополнительно к README был прочитан код crates. Уточнения, меняющие оценку:

- **Rust toolchain зафиксирован**: `rust-toolchain.toml` → `1.98.1`,
  target `x86_64-pc-windows-msvc`, rustfmt + clippy. CI: `cargo clippy
  --workspace --all-targets --locked -- -D warnings` и `cargo test --workspace
  --locked` на `windows-latest`.
- **Динамической рекомпиляции x86-64 нет** (`victory-loader/src/lib.rs`):
  образ копируется, патчится в заголовке (DLL-флаг, entry=0, пустая
  import directory, сброс TLS-указателя и FORCE_INTEGRITY), после чего
  загружается через `LoadLibraryExW`. `.text` не меняется.
- Что есть вместо JIT x86:
  - 32-байтные trampoline-стабы на каждый нереализованный импорт
    (`victory-loader/src/thunks.rs`): `mov rcx, site; mov rax, handler; jmp rax`;
    политики `Trap` (DebugBreak + abort) и `ReturnZero`;
  - патч TLS-доступов `mov r64, gs:[0x58]` → `call thunks[r]; nop`
    (`victory-loader/src/tls.rs`): в GTA VI найдено 53 246 совпадений;
  - точечная эмуляция AMD-инструкций через vectored exception handler
    (`victory-loader/src/crash.rs`): `clzero`, `monitorx/mwaitx`, `clwb`,
    SSE4a `insertq/extrq`; после эмуляции RIP сдвигается.
- **Реестр HLE**: 15 модулей в `victory-hle/src/modules/mod.rs`, 133 экспорта
  `hle_export!`, 12 областей `QueryApiImpl` (xasync, xuser, xsystem, xgame,
  storage, xnetworking, xpackage, display, xgamesave, xstore, xgameui, speech).
- **D3D12.X**: лимиты inlined-пакетов в собственных списках держатся нулевыми,
  поэтому каждая команда приходит вызовом метода таблицы из 160 слотов
  (`victory-gpu/src/list.rs`). Косвенные draw/dispatch аргументы
  ограничиваются compute-шейдером (`victory-gpu/src/indirect.rs`).
- **Память**: `XMemVirtualAlloc` оборачивает `VirtualAlloc2` с выравниванием
  2 MiB; 64 KiB — «страница консоли»; грязные страницы ищутся через
  `GetWriteWatch`, для physical views — `PAGE_GUARD` + VEH. Модель консоли
  из `victory-gpu/src/console.rs`: Series X — 10 GiB GPU-optimal +
  3.5 GiB standard, Series S — 7776 MiB standard.
- **Шейдеры**: DXIL пишется **собственным** writer-ом LLVM 3.7 bitcode
  (`victory-shader/src/emit/bitcode.rs`), без DXC в runtime; DXC-подобный
  валидатор `dxil.dll` используется только в тестах. Поддерживаются только
  `Vs|Ps|Cs|Ms`; 64-bit image atomics дают ошибку; merged geometry stages
  не читаются; `KNOWN_BUILDS` в `victory-game` **пуст** — ни одной build identity
  не зарегистрировано.
- **BCPack уже реализован** в `victory-formats/src/bcpack.rs` (decode-only,
  12/16 rANS lanes, 128 моделей), хотя `docs/architecture.md` всё ещё
  помечает его как planned. Аналогично устарел комментарий «planned: present» —
  `Presenter` в `victory-gpu/src/host.rs` существует.
- **Производительность**: GTA VI создаёт около 60 000 pipeline за первые
  20 секунд; бюджет ожидания pipeline — 0.5 с на кадр, неготовые draw дропаются.
- **Тесты**: 352 атрибута `#[test]`; `#[ignore]` нет; игровые данные тестам
  не нужны; есть end-to-end тесты с собственной fixture `hle-probe` и
  тест asset-policy для всего репозитория.
- **Форматы**: парсера XVC/MSIXVC в репозитории нет — эти расширения только
  в `.gitignore`. Поддерживаются DXBC, Xbox DXIL/XDAT, XBCH/FSSM, SGD5,
  zlib/deflate, BCPack, XMA2, Opus, GFX10 tiling (swizzle 0..27).

Эти детали усиливают вывод: VIctory — зрелый платформенный слой, но с
незавершённой графикой и без легального механизма получения игровых данных.

---

## 2. XCL

### 2.1 Общая информация

| Параметр | Значение |
|---|---|
| Репозиторий | `github.com/ivall/xcl` |
| Язык | Rust, edition 2021 (`Cargo.toml`) |
| Лицензия | **GPL-2.0-only** |
| Тип | compatibility layer Xbox Series X|S → Windows, в духе Wine/Proton |
| Разработка | RX 9070 (RDNA4), Rust 1.96, Windows 11, VS 2022 |
| Статус | несколько игр fully playable; GTA VI — «a newer retail game», меню, gameplay не достигнут |

### 2.2 Архитектура

Workspace: `crates/xcl`, `tests/testtitle`, `tests/testgamedll`.
Основные модули (по `crates/xcl/src/` и `HANDOFF.md`):

| Подсистема | Файлы/модули | Роль |
|---|---|---|
| PE/loader | `loader.rs`, `pe.rs`, `eh.rs`, `tls.rs` | mapping, relocations, imports, delay-load, TLS, C++ exceptions |
| Resolver | `resolver.rs`, `hle/mod.rs`, `stubs.rs`, `data/stubdb.toml` | HLE → game DLL → Windows → stub |
| kernelx | `hle/kernelx.rs` | title memory, physical pages, threads, modules |
| VFS | `vfs.rs`, `vfs_hooks.rs` | `GAME:`, `G:`, `D:`, `S:\`, `T:\`, overlay, whiteouts |
| XGameRuntime | `hle/xgameruntime/` | GDK thunk protocol (396 functions), XUser, XGameSave, PLS, XPackage, XSystem |
| Host | `host.rs`, `launcher.rs`, `report.rs` | процесс, командная строка, `report.json` |
| D3D12.X | `hle/d3d12x/real.rs`, `null.rs`, `guard.rs`, `present.rs`, `console_list.rs` | реальные объекты, console vtable, записи консольных списков, PresentX |
| GPU | `gpu/rdna2.rs`, `gpu/rdna2_xbox.rs`, `gpu/shader.rs`, `gpu/xbox.rs`, `gpu/desc.rs`, `gpu/UMA.rs` | decoder, container parse, descriptors, UMA |
| Recompiler | `gpu/recomp/` (HLSL/DXC), `gpu/dxil/` (direct DXIL) | RDNA2 → HLSL → DXIL и прямой DXIL backend |
| Formats | `gpu/tiling.rs`, `hle/bcpack/`, `hle/xma2/`, `hle/opus.rs`, `gpu/bvh.rs`, `rpf.rs` | тайлинг, BCPack, XMA/Opus, BVH, RPF |
| Audio/Input | `hle/xaudio2.rs`, `hle/gameinput` | XAudio2 2.9, XMA/Opus, GameInput |

Используется консольный интерфейс D3D12.X из `d3d12_xs.dll`.

### 2.3 CPU-код

Эмуляции x86 нет. Код игры выполняется нативно. Ключевые детали из HANDOFF:

- PE loader: relocations, imports by name/ordinal/delay-load, DllMain/TLS,
  game DLLs, data-driven routing (HLE / Windows / stub).
- Static TLS: сгенерированная DLL-носитель; на Windows 11 слот выделяется
  только для DLL с импортами, `SizeOfZeroFill` игнорируется.
- C++ exceptions: hooks на `RtlPcToFileHeader` (retour) и `RtlAddFunctionTable`.
- 16 консольных DLL из игры загружаются с 0 unresolved imports; CRT работает,
  включая `std::mutex` с двумя потоками (40 000 инкрементов).
- Физические страницы: Allocate/Free/MapTitlePhysicalPages в UMA, 64 KiB
  страницы; mapping по адресу, выбранному игрой, или разрозненных страниц
  отклоняется.

### 2.4 GPU-команды

- Реальный D3D12-бэкенд использует расширенные vtable; discovering:
  системные vtable имеют внутренние virtuals после публичных (Device 125
  против 104), поэтому нужны guards по caller address.
- Слот 62 консольного Device = `CreateComputePipelineState` с двумя
  дополнительными аргументами; на PC это CreateStateObject — сделан split по
  месту вызова.
- **Console command list**: гостевой код пишет во внутренние поля объекта
  (`+0x18` write pointer, `+0x30` end, `+0x140` dirty, root params at `+0x344`
  и т.д.). Простая подмена vtable недостаточна — создан собственный объект
  со «shadow»-областью и интерпретатором записей при Close/Execute.
- Compute-формат записей восстановлен на примере `XboxBvhBuilder.dll`:
  pipeline 28 B, root parameters, Dispatch (`0x4002C400`), barrier, indirect
  dispatch (`0x4003CC00`) — последний ещё не реализован.
- Результат: реальный `XboxBvhBuilder.dll` строит BLAS из 64 треугольников
  на RX 9070 через XCL; BLAS/TLAS проверены трассировкой луча против brute force.
- Графические записи draw, по данным HANDOFF, ещё предстоит определять на игре.
- PresentX и frame pacing в XCL window реализованы; presentation untested on a game.

### 2.5 Системные API

- `kernelx`: title memory (бюджет из `MicrosoftGame.Config`), потоки на
 processor хоста, модули, OS/console info; физические страницы.
- `XGameRuntime`: 396-функциональная таблица thunk protokol; offline XUser,
  XGameSave, PLS, XPackage, XSystem, task queues/async; часть вызовов
  forwarding в системный `xgameruntime.dll`.
- VFS: `GAME:`, `G:`, `D:` → папка игры; записи в overlay; папка игры
  остаётся byte-identical; `S:\`/`T:\` в data-папке.
- DirectStorage: file/memory sources, zlib, BCPack для BC1/3/4/5/6H/7;
  hardware swizzle и BC2 отклоняются.
- XAudio2 → system XAudio2.9, Opus через `MSOpusDecoder.dll` или libopus,
  XMA1/XMA2 — pure Rust (порт FFmpeg под LGPL-2.1+).
- GameInput → **собственные** консольные COM-интерфейсы в `hle/gameinput.rs`
  (один геймпад из XInput); клавиатура и мышь как геймпад — `hle/kbm.rs`
  через Raw Input. Системная `GameInput.dll` в XCL не используется.
- Online-часть — offline-заглушки; сетевые вызовы XCL не эмулирует полностью.

### 2.6 Память

- UMA: единый регион (2048 MB default, максимум 4095 MB), 4 GiB-aligned,
  fully committed, открывается через `ID3D12Device3::OpenExistingHeapFromAddress`.
  Младшие 32 бита консольного адреса = offset в буфере; GPU видит те же данные
  без копий. First-fit allocator (64 KiB, coalescing).
- Title graphics memory размещается в UMA при `VirtualAlloc(MEM_GRAPHICS)`.
- Ограничение: 4 GiB region rejected, partial commit rejected (измерено).

### 2.7 Трансляция шейдеров

**Форматы (HANDOFF, §5.7a):** DXIL (bitcode `NOBC`) → XDAT (+0x18) →
XBCH v3 (zlib) → FSSM v0x101; code в u64 +0xC0/+0xC8; stage descriptors
`0xC0_0002_AAHH` (HH: 2 ES, 3 GS, 4 VS, 5 PS, 6 CS, 7 MS/NGG);
data blocks отдельно. PSV0/ISG1/OSG1/RTS0 используются для интерфейсов
и раскладки root arguments.

**Декодер:** `gpu/rdna2_spec.rs` сгенерирован из AMD Machine-Readable ISA
(MIT); 23 812 инструкций из LLVM-тестов — 100% имени и длины;
консольные расширения `v_mac_f32`, `v_madmk_f32`, `v_madak_f32`, `v_mad_f32`;
SDWA/DPP; 395 mnemonics на полном паке шейдеров.

**HLSL-бэкенд (§5.7b и сессии 3–5):**

- switch/if chain over PC, SGPR uniform, VGPR per-lane, EXEC через lanebit,
  cross-lane через `WaveReadLaneAt`/`WaveActiveBallot`.
- Поддержаны: SALU/VALU f32/u32/i32, f16, 64-bit, atomics (включая fmin/fmax
  через CAS), LDS, MUBUF/MTBUF с 78 форматами, image/sampler, mesh (two-pass),
  legacy GS (ES+GS в один compute + mesh passthrough), BVH intersection,
  wave32-on-wave64 и wave64-on-wave32 (часть).
- Важное открытие: branch-free HLSL (`select()` вместо `?:`) радикально
  ускоряет DXC и снижает потребление памяти.
- DXC: `-O0` для очень больших HLSL; встроенный in-process компилятор;
  кэш DXIL в `%LOCALAPPDATA%\xcl\dxil`.

**Прямой DXIL-бэкенд (docs/specs/2026-11-24-direct-dxil.md):** в ветке
`shaders/direct-dxil`, слои `bitstream`, `module`, `shader`, `container`,
`validate`; цель — убрать HLSL и DXC из runtime; фазы 1–3 сделаны,
4–6 частично; coverage и живые прогоны не измерены.

**Coverage:** 24 064 / 24 064 программ одного retail-пака переводятся
полностью (по HANDOFF); 370+ автотестов; GPU-тесты сравнивают инструкции.

### 2.8 Поддерживаемые форматы

PE32+, XDAT/XBCH/FSSM/SGD5-подобные контейнеры, DXIL/DXBC, RPF9 (shader
packs, audio, textures), TTPF, BK2 (Bink 2), AWC, `*.bvh4`, BCPack, XMA1/2,
Opus, GFX10 tiling (14 2D + 6 3D режимов), AMD BVH (box16/box32, instance
nodes), FSR4 console DLL (политика: не эмулировать модель, а HLE FFX API).

XVC/MSIXVC XCL не расшифровывает; работает только с распакованными файлами
пользователя.

### 2.9 Ограничения

- Игры: Minecraft Preview и Unicorn Overlord fully playable; Fortnite и
  XDefiant — насколько позволяет отсутствие серверов; GTA VI — меню/settings,
  gameplay не достигнут, minimap mesh pipelines отклоняются.
- wave64 на NVIDIA (32-lane) — две половины, медленнее; wave64 VS/PS с чтением
  lane ≥ 32 могут отличаться.
- Графические записи D3D12.X и дескрипторные таблицы для графики — не завершены.
- Индиректный dispatch (`0x4003CC00`) не реализован.
- Тайлинг: _X-режимы требуют GB_ADDR_CONFIG, не определен на игре; mip tail
  не реализован.
- Один инцидент: компиляция DXC съела 38 ГБ committed и убила сессию;
  добавлены `-O0` пороги и memory cap в тестах.
- Часть кода XCL имеет стороннее происхождение: порт кода VIctory под
  GPL-2.0-only сделан с персонального разрешения автора — BrVIdge не может
  предполагать наличие такого разрешения.

### 2.10 Лицензия

GPL-2.0-only; `THIRD_PARTY.md` содержит подробную таблицу:
AnyPS5 GPL-2.0-only, FFmpeg LGPL-2.1+, AMD ISA MIT, Mesa addrlib MIT,
VIctory GPL-3 (special permission), LLVM Apache-2.0 WITH LLVM-exception,
WinDurango LGPL-3.0 (reference only), libHttpClient MIT, DXC NCSA/Microsoft,
Rust crates MIT/BSD/Zlib/Unlicense, libopus BSD-3 (не распространяется).
Политика совместимости: с GPL-2.0-only совместимы MIT/BSD/Zlib/ISC/Unlicense/
LGPL-2.1+/GPL-2.0+/MPL-2.0; несовместимы Apache-2.0 без альтернативы,
GPL-3, LGPL-3, AGPL.

---

### 2.11 Уточнения по реальному коду (аудит исходников)

Дополнительно к README и HANDOFF был прочитан код (292 файла, из них
160 `.rs`, около 121 000 строк Rust). Ключевые уточнения:

- **Только Windows x64.** Файла `rust-toolchain` в репозитории нет; версия
  Rust 1.96 зафиксирована только в `HANDOFF.md`. `build.rs` добавляет
  `/HIGHENTROPYVA:NO`, чтобы адреса тайтла оставались < 2^40 (требуется
  BVH-билдеру ритейл-игры).
- **Запуск:** launcher запускает тот же `xcl.exe` дочерним процессом с
  `XCL_HOST=<boot.json>`; в host-режиме инициализируются UMA, VFS-хуки,
  `RtlPcToFileHeader`, crash handler, PE-образ мапится и entry point
  вызывается в отдельном потоке с размером стека из PE.
- **Resolver** имеет 5 уровней: aliases → HLE (с fallback в системные DLL)
  → DLL игры → системная Windows DLL → auto-stub. Stub — 48-байтовый
  x64-thunk в RWX-арене, первый вызов пишется в лог; значения по умолчанию
  хранятся в `data/stubdb.toml`.
- **TLS-carrier:** генерируется минимальная DLL с импортом, чтобы Windows
  выделила TLS-слот; `SizeOfZeroFill` игнорируется (эмпирически).
- **HLE в цифрах:** `kernelx.dll` — 59 экспортов; `xgameruntime.dll` —
  127 name-экспортов и таблица из **396** GDK-функций; `d3d12_xs.dll` —
  12 экспортов. Из 396 GDK-слотов, по данным HANDOFF, XCL отвечает на 123,
  системный runtime — на 111, 109 отклонены, 39 не реализованы.
- **D3D12.X:** расширенные vtable реальных PC-объектов (публичные методы,
  guard-туннели по caller address, консольные слоты). device #62 =
  CreateComputePipelineState с двумя лишними аргументами; queue #30 =
  PresentX. Модуль `console_list.rs` — интерпретатор **только compute** и
  **opt-in** (`XCL_CONSOLE_LISTS=1`); по комментарию в исходнике — «until the
  graphics records are known». Неизвестная запись → warning + пропуск.
- **Интерпретатор записей:** собственные маркеры XCL (`PIPE_MARKER`,
  `RS_MARKER`, `BARRIER_MARKER`, …), 600 слотов vtable, чанки 1 МБ,
  root arguments по RTS0: параметры идут в user SGPRs s4…s15 (compute) /
  s8…s31 (graphics), остальное — в таблицу по `s[0:1]`.
- **UMA:** GRAIN 64 KiB, `MAX_SIZE = 4096 MiB - 64 KiB`, WINDOW 4 GiB,
  до 4 окон; base 16 GiB-aligned; first-fit allocator. Пул XCL резервируется
  заранее, иначе тайтл, коммитящий почти всю UMA (2046 из 2048 MB в Unicorn
  Overlord), не оставляет места под аргументы draw.
- **Транслятор:** отдельного HLSL IR в коде нет — `emit.rs` сразу порождает
  текст HLSL (например, 5738 строк); IR в классическом смысле есть только
  в прямом DXIL-бэкенде (`gpu/dxil/{ssa,cfg,lanes,module}.rs`). Включение
  прямого DXIL: `XCL_DIRECT_DXIL=1|all`; `XCL_TRANSLATOR_SOURCE_ID` из хэша
  12 исходников входит в ключ кэша.
- **Форматы:** RPF9 (`9FPR`) — только сканер записей; TOC зашифрован и не
  парсится. Нет собственных парсеров/декодеров XVC/MSIXVC, BK2, AWC
  (runtime), BVH4; для них тесты вызывают DLL игры. TTPF разбирается только
  в тестовом модуле. GFX10 tiling: 14 2D + 6 3D режимов, но MSAA не реализован.
- **Известные пробелы по коду:** VFS не перехватывает прямые `Nt*`-syscalls;
  `Deserialize*PipelineStateX` → `E_NOTIMPL`; SHAPE-микшер APU не
  запускается; Frame clock не привязан к vblank; GPU-записи в title-texture
  память не видны XCL; parent pointers в BVH не пишутся.
- **Тесты:** статически 900 атрибутов `#[test]` (README заявляет более
  старые «370+»), 42 `#[ignore]`; часть GPU-тестов при отсутствии
  адаптера/DXC печатает `SKIPPED` и считается пройденной; 15 integration-файлов.
  Тесты на реальных данных включаются переменными `XCL_SAMPLE_*` и без них
  ничего не проверяют. В репозитории только синтетические данные.

Эти детали подтверждают: XCL — самый зрелый по объёму кода референс, но
даже в нём графический D3D12.X и точные ABI ритейл-игры остаются
недоопределёнными.

---

## 3. CGBJ (только публичные материалы)

| Параметр | Значение |
|---|---|
| Природа | закрытая (private) сборка/проект |
| База | по сообщениям — VIctory + часть наработок XCL |
| Исходный код | **недоступен**, independent verification невозможна |
| Публичный статус (24.11.2026) | перемещение по миру, разговоры с NPC, инвентарь/меню; сцена в плоских неосвещённых цветах, очень медленно |

Источники:

- PortWatchVI, 24.11.2026: «CGBJ gets GTA VI in game, walking and talking in
  an unlit world»; видео 77 с; 72% по их шкале; code private.
- XDA, 24.11.2026: подтверждает со ссылкой на PortWatch, отмечает, что
  коммерчески проект не проверен и что трекеры могут преувеличивать прогресс.
- DSOGaming: заголовок о достижении in-game (страница недоступна для чтения).

**Выводы для BrVIdge:**

1. Изучать реализацию CGBJ невозможно — доступны только описания, и те
  потенциально неточные. Любые детали о его архитектуре были бы выдумкой.
2. Сам факт «in-game, но без освещения» совпадает с проблемой VIctory
   («сцена после титр тёмная») — это усиливает гипотезу, что главный блокер —
   графический конвейер, а не загрузка игры.
3. CGBJ не является источником кода или лицензии для BrVIdge.

---

## 4. Xenia

| Параметр | Значение |
|---|---|
| Репозиторий | `github.com/xenia-project/xenia` |
| Назначение | эмулятор Xbox 360 |
| Язык | C++ (сборка через `xb`, premake; VS 2017–2022, Python 3.6+) |
| Лицензия | BSD-2-Clause (copyright Ben Vanik, 2015) |
| CPU | PowerPC Xenon → JIT-рекомпилятор (в docs/building.md упомянут JIT-код ~0xA0000000 и `--emit_source_annotations`) |
| GPU | Xenos; в документации упоминается Vulkan-бэкенд; SDL2 и Vulkan — в зависимостях Linux-сборки |
| Статус | «Quite a few real games run. Quite a few don't.» (README) |

**Что полезно BrVIdge:**

- культура research/education и жёсткое разделение с illegal activity;
- модульная структура CPU/GPU/systems и game-compatibility tracking;
- логи и отладочные.flags (JIT-annotations).

**Почему не основа:**

- другое поколение консоли (PowerPC, Xenos, Xbox 360 OS); для Series X|S
  CPU-эмуляция не нужна, а GPU и API принципиально другие;
- код C++ под BSD-2 совместим по лицензии, но архитектурно не переносим.

Глубокий аудит исходников Xenia в этой фазе не проводился — изучены README,
LICENSE и docs/building.md. Все утверждения выше ограничены этими файлами.

---

## 5. Другие релевантные проекты и источники

### 5.1 WinDurango

- `github.com/WinDurango/WinDurango`, Xbox One translation layer поверх UWP,
  C++/CMake/vcpkg, реализует KernelX, Xbox WinRT interfaces, D3D11.X.
- Список игр в README: Sonic Mania, Minecraft XBE, LIMBO — playable;
  Forza Horizon 2, Forza Motorsport 5, Peggle 2 — boot.
- Лицензия LGPL-3.0 (по `THIRD_PARTY.md` XCL).
- Для XCL/XVI serves только как reference (номера экспортов d3d12_x.dll,
  без переноса кода). Для BrVIdge — пример более раннего поколения ERA-слоя.

### 5.2 AnyPS5

- `github.com/boykopovar/AnyPS5`, C++, GPL-2.0-only.
- Relinker + реализации system PRX-библиотек; без эмуляции и отдельного
  runtime-процесса; shader recompiler → SPIR-V; поддержка SDL-геймпадов.
- XCL ported отдельные части логики рекомпилятора в Rust/HLSL с пометками
  provenance (GPL-2.0 §2a). Для BrVIdge является примером подхода
  «native execution + relink + shader recompile», но **не источником кода**.

### 5.3 Смежные открытые спецификации (по THIRD_PARTY.md XCL)

- AMD Machine-Readable ISA (MIT) — машиночитаемые таблицы RDNA2.
- Mesa addrlib (MIT) — swizzle-паттерны GFX10.
- LLVM (Apache-2.0 WITH LLVM-exception) и DXC (NCSA/Microsoft) — тестовые
  вектора, формат DXIL, валидатор.
- AMD GPURT (MIT) — формат BVH-узлов.
- FFmpeg (LGPL-2.1+) — XMA1/XMA2.
- libHttpClient (MIT), docs.xbox.com — контракты и документация API.
- Windows XGameRuntime — системный компонент; class GUIDs и слоты thunk
  protocol определены анализом, API forwarding в системную DLL.

### 5.4 Проекты-однодневки и трекеры

- Leonida — публичный проект, удалён 23.11.2026 (репозиторий и Discord).
- TuxBox, omonkey — упоминаются в публичных трекерах как другие проекты;
  исходники и результаты не проверялись, поэтому в BrVIdge не используются
  как источник фактов.
- Fan-трекиеры (PortWatchVI, victorygta6.wiki, gta6recomp.com,
  gta.mtwildwood.net) — полезны как указатели на первичные источники,
  но их метрики и проценты не являются доказательством работоспособности.

---

## 6. Сводная сравнительная таблица

| Критерий | VIctory | XCL | CGBJ | Xenia |
|---|---|---|---|---|
| Архитектура | native+HLE+D3D12.X | native+HLE+D3D12.X | неизвестна (закрыт) | полная эмуляция Xbox 360 |
| Язык | Rust 2024 | Rust 2021 | неизвестен | C++ |
| CPU | без эмуляции | без эмуляции | неизвестно | PowerPC JIT |
| GPU | D3D12.X → D3D12, indirect draws | D3D12.X + console list interpreter → D3D12 | неизвестно | Xenos → Vulkan/D3D |
| Системные API | HLE, offline, VFS | HLE + GDK thunks + system DLL | неизвестно | эмуляция Xbox 360 API |
| Память | title memory, изменённые страницы | UMA, окна 4 GiB, физические страницы | неизвестно | guest RAM Xenon |
| Шейдеры | RDNA2 → SSA → DXIL, structure+resource | RDNA2 → HLSL → DXIL; direct DXIL в разработке | неизвестно | shader conversion для Xenos |
| Форматы | PE, SGD5, XMA/Opus, tiling | PE, RPF9, TTPF, BK2, BVH4, BCPack, XMA/Opus | неизвестно | XEX/XDB и др. |
| Тесты | синтетические + CLI (проект утверждает) | 370+/100% corpus/GPU-тесты | нет данных | game compatibility list |
| Лицензия | GPL-3.0-or-later | GPL-2.0-only | закрыт | BSD-2-Clause |
| Повторное использование | архитектурные идеи | архитектурные идеи | нет | немного (culture) |

Оценки в таблице основаны на документации и коде репозиториев; для CGBJ —
только на публичных сообщениях.

---

## 7. Главные выводы исследования

1. Для Xbox Series X|S **эмуляция CPU не нужна**: все три актуальных проекта
   используют native x86-64 + HLE. BrVIdge должен следовать этому пути.
2. **Самый твёрдый технический долг — GPU**: консольные шейдеры RDNA2
   требуют полноценной трансляции, а формат D3D12.X для графики до сих пор
   не полностью восстановлен даже в лидирующих проектах.
3. **XCL даёт наиболее зрелую инженерную базу** (тесты, UMA, рекомпилятор,
   compute-списки), но под GPL-2.0-only; **VIctory — ближе к GTA VI**
   (специфичные форматы, offline-сервисы, таргетная отладка), но под GPL-3.
   Код ни одного из них не может быть механически перенесён без соблюдения
   лицензии; BrVIdge реализует идеи самостоятельно.
4. **CGBJ не может быть изучен**: закрытый код и непроверенные публичные
   сообщения. Его прогресс диктует осторожность в оценках, но не является
   источником технических решений.
5. **Ключевой нетехнический блокер** — отсутствие легального тестового
   материала. Все достигнутые публичные результаты получены на утекшей
   сборке; BrVIdge не должен её использовать и поэтому на старте не сможет
   подтвердить совместимость с реальной игрой.
6. **Целевая машина** (6 ядер, 8 ГБ VRAM, wave32) отличается от машин
   разработчиков референсных проектов (RX 6700 XT / RX 9070), поэтому
   переносимость и экономия памяти должны быть целями с первого прототипа.

---

## 8. Источники (список)

- `https://github.com/Marvisak/VIctory` — README, `docs/architecture.md`,
  `Cargo.toml`, `LICENSE`, код crates (локальный клон, commit ffe310e).
- `https://github.com/ivall/xcl` — README, `HANDOFF.md`, `THIRD_PARTY.md`,
  `Cargo.toml`, `docs/specs/2026-11-20-xcl-design.md`,
  `docs/specs/2026-11-24-direct-dxil.md`, код (commit c160d86).
- `https://github.com/boykopovar/AnyPS5` — README, LICENSE.
- `https://github.com/xenia-project/xenia` — README, LICENSE, docs/building.md.
- `https://github.com/WinDurango/WinDurango` — README.
- `https://portwatch-vi.xyz/news/2026-11-24-cgbj-in-game` — публичный отчёт о CGBJ.
- `https://www.xda-developers.com/unofficial-gta-6-pc-port-making-progress-developers-have-first-in-game-visuals/` — публичный обзор, правовая рамка.
- `https://www.rockstargames.com/VI/` — официальные платформы и дата релиза.
- `https://en.wikipedia.org/wiki/Xbox_Series_X_and_Series_S` — публичные спецификации железа.
- `https://en.wikipedia.org/wiki/Grand_Theft_Auto_VI` — публичная информация об игре, RAGE, утечках.
- `https://gta.mtwildwood.net/gta-6-pc/` — трекер PR/статусов (только как указатель).

