//! Синтетический PE32+ с импортами консольных DLL.
//!
//! Образ создаётся целиком в памяти из собственных констант. Он не является
//! игровым файлом и не содержит кода GTA VI.

#![deny(missing_docs)]

/// RVA каталога импортов.
pub const DIRECTORY_IMPORT: usize = 1;
/// Число каталогов данных.
pub const DIRECTORY_COUNT: usize = 16;

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn put_text(bytes: &mut [u8], offset: usize, text: &str) {
    bytes[offset..offset + text.len()].copy_from_slice(text.as_bytes());
    bytes[offset + text.len()] = 0;
}

/// Максимум дескрипторов импорта в синтетическом образе.
pub const MAX_IMPORTS: usize = 8;
/// Максимальная длина имени DLL или функции.
pub const MAX_NAME: usize = 30;

/// Пара `(DLL, функция)` для синтетического импорта.
pub type ImportSpec = (&'static str, &'static str);

/// Импорты базового сценария.
pub fn boot_imports() -> &'static [ImportSpec] {
    &[
        ("kernelx.dll", "XMemAlloc"),
        ("xgameruntime.dll", "XUserGetXUID"),
        ("d3d12_xs.dll", "D3D12XboxCreateDevice"),
        ("unknown_x.dll", "NoImplementation"),
    ]
}

/// Импорты сценария Series S: добавлен неподдерживаемый FSR4-вызов.
pub fn series_s_imports() -> &'static [ImportSpec] {
    &[
        ("kernelx.dll", "XMemAlloc"),
        ("xgameruntime.dll", "XUserGetXUID"),
        ("d3d12_xs.dll", "D3D12XboxCreateDevice"),
        ("amd_fidelityfx_upscaler_xs.dll", "ffxCreateContext"),
        ("unknown_x.dll", "NoImplementation"),
    ]
}

/// Импорты сценария host/delay-load: системные DLL и одна заглушка.
pub fn host_imports() -> &'static [ImportSpec] {
    &[
        ("kernel32.dll", "GetTickCount"),
        ("ntdll.dll", "RtlAllocateHeap"),
        ("kernelx.dll", "XMemAlloc"),
        ("unknown_x.dll", "NoImplementation"),
    ]
}

/// Строит PE32+ AMD64 с заданным списком импортов.
///
/// Раскладка: дескрипторы `0x300`, имена DLL `0x400+`, thunks `0x500+`,
/// имена функций `0x580+`. RVA = file - 0x200 + 0x1000.
pub fn build_with_imports(imports: &[ImportSpec]) -> Result<Vec<u8>, String> {
    if imports.is_empty() || imports.len() > MAX_IMPORTS {
        return Err(format!("imports count {} out of range", imports.len()));
    }
    for (dll, function) in imports {
        if dll.len() > MAX_NAME || function.len() > MAX_NAME {
            return Err("import name too long".to_owned());
        }
    }

    let pe = 0x80usize;
    let coff = pe + 4;
    let optional = pe + 24;
    let section = optional + 240;
    let raw = 0x200usize;
    let va = 0x1000u32;
    let mut bytes = vec![0u8; 0x800];

    put_u16(&mut bytes, 0, 0x5A4D);
    put_u32(&mut bytes, 0x3C, pe as u32);
    put_u32(&mut bytes, pe, 0x0000_4550);
    put_u16(&mut bytes, coff, 0x8664);
    put_u16(&mut bytes, coff + 2, 1);
    put_u16(&mut bytes, coff + 16, 240);
    put_u16(&mut bytes, coff + 18, 0x0102);

    put_u16(&mut bytes, optional, 0x020B);
    put_u32(&mut bytes, optional + 16, va);
    put_u64(&mut bytes, optional + 24, 0x1400_0000);
    put_u32(&mut bytes, optional + 32, 0x1000);
    put_u32(&mut bytes, optional + 36, 0x200);
    put_u32(&mut bytes, optional + 56, 0x2000);
    put_u32(&mut bytes, optional + 60, 0x200);
    put_u32(&mut bytes, optional + 108, DIRECTORY_COUNT as u32);
    put_u32(
        &mut bytes,
        optional + 112 + DIRECTORY_IMPORT * 8,
        va + 0x100,
    );
    put_u32(
        &mut bytes,
        optional + 112 + DIRECTORY_IMPORT * 8 + 4,
        ((imports.len() + 1) * 20) as u32,
    );

    bytes[section..section + 5].copy_from_slice(b".text");
    put_u32(&mut bytes, section + 8, 0x600);
    put_u32(&mut bytes, section + 12, va);
    put_u32(&mut bytes, section + 16, 0x600);
    put_u32(&mut bytes, section + 20, raw as u32);
    put_u32(&mut bytes, section + 36, 0x6000_0020);

    for (index, (dll, function)) in imports.iter().enumerate() {
        let descriptor = 0x300 + index * 20;
        let dll_file = 0x400 + index * 0x20;
        // Каждый массив thunks: одна запись + нулевой терминатор (16 bytes).
        let thunk_file = 0x500 + index * 16;
        let func_file = 0x580 + index * 0x20;
        let dll_rva = va + (dll_file - raw) as u32;
        let thunk_rva = va + (thunk_file - raw) as u32;
        let func_rva = va + (func_file - raw) as u32;

        put_u32(&mut bytes, descriptor, thunk_rva);
        put_u32(&mut bytes, descriptor + 12, dll_rva);
        put_u32(&mut bytes, descriptor + 16, thunk_rva);
        put_u64(&mut bytes, thunk_file, u64::from(func_rva));
        put_u16(&mut bytes, func_file, 0);
        put_text(&mut bytes, func_file + 2, function);
        put_text(&mut bytes, dll_file, dll);
    }

    Ok(bytes)
}

/// Строит базовый синтетический PE32+.
pub fn build_title() -> Vec<u8> {
    build_with_imports(boot_imports()).expect("valid boot imports")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_pe_has_expected_headers() {
        let bytes = build_title();
        assert_eq!(bytes[0], b'M');
        assert_eq!(bytes[1], b'Z');
        assert_eq!(&bytes[0x80..0x84], b"PE\0\0");
        assert_eq!(&bytes[0x188..0x18D], b".text");
    }

    #[test]
    fn synthetic_pe_contains_console_import_names() {
        let bytes = build_title();
        let text: Vec<u8> = bytes.iter().copied().filter(|b| *b != 0).collect();
        let joined = String::from_utf8_lossy(&text);
        assert!(joined.contains("kernelx.dll"));
        assert!(joined.contains("XMemAlloc"));
        assert!(joined.contains("xgameruntime.dll"));
        assert!(joined.contains("XUserGetXUID"));
        assert!(joined.contains("d3d12_xs.dll"));
        assert!(joined.contains("D3D12XboxCreateDevice"));
    }

    #[test]
    fn xcl_pe_parser_reads_synthetic_imports() {
        let bytes = build_title();
        let pe = xcl::pe::Pe::parse(&bytes, xcl::pe::Layout::File).unwrap();
        let imports = pe.imports().unwrap();
        assert_eq!(imports.len(), 4);
        assert_eq!(imports[0].dll, "kernelx.dll");
        assert_eq!(imports[0].entries[0].what.to_string(), "XMemAlloc");
        assert_eq!(imports[1].dll, "xgameruntime.dll");
        assert_eq!(imports[1].entries[0].what.to_string(), "XUserGetXUID");
        assert_eq!(imports[2].dll, "d3d12_xs.dll");
        assert_eq!(
            imports[2].entries[0].what.to_string(),
            "D3D12XboxCreateDevice"
        );
        assert_eq!(imports[3].dll, "unknown_x.dll");
        assert_eq!(imports[3].entries[0].what.to_string(), "NoImplementation");
    }

    #[test]
    fn parsed_imports_map_to_offline_routes() {
        let bytes = build_title();
        let pe = xcl::pe::Pe::parse(&bytes, xcl::pe::Layout::File).unwrap();
        let routes = crate::routes_from_xcl_imports(&pe.imports().unwrap());
        assert_eq!(routes.len(), 4);
        assert_eq!(
            routes.iter().filter(|(_, kind)| *kind == crate::RouteKind::Hle).count(),
            3
        );
        assert_eq!(
            routes.iter().filter(|(_, kind)| *kind == crate::RouteKind::Stub).count(),
            1
        );
    }

    #[test]
    fn scenarios_are_parsed_by_xcl_and_have_expected_routes() {
        for (imports, expected) in [
            (series_s_imports(), (3usize, 0usize, 2usize)),
            (host_imports(), (1usize, 2usize, 1usize)),
        ] {
            let bytes = build_with_imports(imports).unwrap();
            let pe = xcl::pe::Pe::parse(&bytes, xcl::pe::Layout::File).unwrap();
            let routes = crate::routes_from_xcl_imports(&pe.imports().unwrap());
            assert_eq!(routes.len(), imports.len());
            for (kind, count) in [
                (crate::RouteKind::Hle, expected.0),
                (crate::RouteKind::Host, expected.1),
                (crate::RouteKind::Stub, expected.2),
            ] {
                assert_eq!(
                    routes.iter().filter(|(_, k)| *k == kind).count(),
                    count
                );
            }
        }
    }

    #[test]
    fn invalid_import_lists_are_refused() {
        assert!(build_with_imports(&[]).is_err());
        let long: Vec<ImportSpec> = (0..(MAX_IMPORTS + 1))
            .map(|_| ("kernel32.dll", "GetTickCount"))
            .collect();
        assert!(build_with_imports(&long).is_err());
        assert!(build_with_imports(&[("a_very_long_dll_name_that_does_not_fit.dll", "f")]).is_err());
    }

    #[test]
    fn synthetic_pe_is_not_a_real_executable() {
        let bytes = build_title();
        assert_eq!(bytes.len(), 0x800);
        assert_ne!(bytes, crate::synthetic_image(0));
    }
}
