# Отчёт о Phase 7 (GPU Backend), промежуточный

Дата: 25 ноября 2026 года. Статус: **часть компонентов реализована и
проверена, ворота этапа пока не пройдены**.

## 1. Проверка Phase 6 перед началом

- Исходники `crates/brv_exec/src/lib.rs` содержат реальные Win32-вызовы
  (`VirtualAlloc`, `VirtualProtect`, `CreateThread`, affinity/priority).
- `cargo test --workspace`: все 59 тестов Phase 6 зелёные.
- `cargo clippy --workspace --all-targets -- -D warnings`: без замечаний.
- `brv native` и `brv run` подтверждают нативный запуск и JSON-отчёт.

## 2. Реализованные компоненты Phase 7

| Компонент | Что сделано | Тесты |
|---|---|---|
| D3D12 device/queues/SwapChain | `Presentation`: скрытое окно, `IDXGIFactory2`, `CreateSwapChainForHwnd` с **command queue**, 2 RTV, текущий backbuffer index, Present | `swapchain_presents_frames` |
| Списки команд | `console_list`: 9 тегов, builder + interpreter, явные ошибки на обрезанных/неизвестных записях | 4 теста |
| Pipeline manager | `manager::PipelineCache` по FNV-ключу (blob + root signature) | 3 теста |
| Root signatures/descriptors | `manager::DescriptorAllocator`: allocate/release/reuse, контроль исчерпания | 2 теста |
| RDNA2 → DXIL | декодер расширен: `s_sub_u32`, `s_and_b32`, `v_mul_f32`, `v_lshlrev_b32`; HLSL-emitter использует `asuint/asfloat` для сдвига | 1 новый тест декодера + DXC-компиляция |
| Shader cache | `brv_shader_cache`: ключ = версия + содержимое, atomic write+rename, hits/misses | 4 теста |
| Тайлинг | `brv_formats::tiling`: 256B XOR-раскладка, `tile_8bpp`/`detile_8bpp` | round-trip и валидация |
| BC1 | `brv_formats::bc1::decode_block`: RGBA8 4×4, 2- и 3-цветные режимы | 3 теста |
| BCPack | **не реализован**: `decode()` возвращает явную ошибку с указанием причин | тест на явный отказ |

## 3. Результаты проверки

```
cargo xtask all                         → 81 тест, все зелёные
cargo clippy --workspace --all-targets -- -D warnings → чисто
brv present                             → SwapChain создан, показано 2 кадра
brv gpu                                 → треугольник, проверка пикселей пройдена
brv shader-cache --dir ...              → попадание 1, промах 1, запись найдена
brv formats                             → тайлинг round-trip ПРОЙДЕН, BC1 ок, BCPack «не реализовано»
brv dxil                                → DXIL валидирован DXC
```

## 4. Явные ограничения (не выдаются за реализованные)

- Формат записей `console_list` — **собственный синтаксис BrVIdge**,
  а не реконструированный D3D12.X.
- Тайлинг — упрощённая модель, несовместимая с реальными GFX10-паттернами
  без эталонных текстур.
- BCPack требует rANS-таблиц silicon-rans и легальных потоков; функция
  намеренно возвращает ошибку, а не подставной результат.
- Не сделаны: wave64-on-wave32, 64-битные atomics, ограничение DXC-памяти
  через Job Object, полный набор инструкций RDNA2, PresentX консоли.

## 5. Вывод

Phase 7 выполнена частично. Все заявленные в отчёте функции реально
реализованы и покрыты тестами; незавершённые части помечены явно.
Переход к Phase 8 отложен до закрытия ворот Phase 7.
