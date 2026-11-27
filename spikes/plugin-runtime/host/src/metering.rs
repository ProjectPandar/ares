use anyhow::{Context, Result, bail, ensure};
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, Encode, ExportKind, GlobalType, Instruction, MemorySection,
    MemoryType, Module, RawSection, ValType,
};
use wasmparser::{
    BinaryReader, CodeSectionReader, Operator, Parser, Payload, TypeRef, Validator, WasmFeatures,
};

pub const FUEL_EXPORT: &str = "__ares_fuel";

pub struct MeteredModule {
    pub bytes: Vec<u8>,
    pub segments: usize,
    pub code_size: (usize, usize),
}

pub fn instrument(wasm: &[u8], max_pages: u32) -> Result<MeteredModule> {
    // Validate first so original indices cannot name the added global.
    Validator::new_with_features(WasmFeatures::WASM2 | WasmFeatures::EXTENDED_CONST)
        .validate_all(wasm)
        .context("validate core guest")?;
    let mut module = Module::new();
    let mut fuel_index = 0;
    let mut needs_global = true;
    let mut needs_export = true;
    let mut has_memory = false;
    let mut segments = 0;
    let mut code_size = (0, 0);
    for payload in Parser::new(0).parse_all(wasm) {
        let payload = payload?;
        let Some((id, range)) = payload.as_section() else {
            continue;
        };
        let (start, end) = (range.start as usize, range.end as usize);
        if id >= 7 && needs_global {
            append_global(&mut module, 0, &[]);
            needs_global = false;
        }
        if id >= 8 && needs_export {
            append_export(&mut module, 0, &[], fuel_index);
            needs_export = false;
        }
        match payload {
            Payload::ImportSection(reader) => {
                fuel_index = imported_globals(reader)?;
                module.section(&RawSection {
                    id,
                    data: &wasm[start..end],
                });
            }
            Payload::MemorySection(reader) => {
                ensure!(reader.count() == 1, "guest must define one memory");
                let mut memory = MemorySection::new();
                for entry in reader {
                    let ty = entry?;
                    let cap = u64::from(max_pages).min(ty.maximum.unwrap_or(65_536));
                    ensure!(ty.initial <= cap, "initial memory exceeds guest cap");
                    memory.memory(MemoryType {
                        minimum: ty.initial,
                        maximum: Some(cap),
                        memory64: false,
                        shared: false,
                        page_size_log2: None,
                    });
                }
                module.section(&memory);
                has_memory = true;
            }
            Payload::GlobalSection(reader) => {
                fuel_index += reader.count();
                append_global(
                    &mut module,
                    reader.count(),
                    &wasm[reader.original_position() as usize..end],
                );
                needs_global = false;
            }
            Payload::ExportSection(reader) => {
                let (count, items) = (reader.count(), reader.original_position() as usize);
                for export in reader {
                    ensure!(
                        export?.name != FUEL_EXPORT,
                        "reserved fuel export already exists"
                    );
                }
                append_export(&mut module, count, &wasm[items..end], fuel_index);
                needs_export = false;
            }
            Payload::StartSection { .. } => bail!("guest start function is not supported"),
            Payload::CodeSectionStart { .. } => {
                let mut code = CodeSection::new();
                let reader =
                    CodeSectionReader::new(BinaryReader::new(&wasm[start..end], start as u64))?;
                code_size.0 = end - start;
                let mut body = Vec::new();
                for function in reader {
                    let function = function?;
                    let mut ops = function.get_operators_reader()?;
                    let begin = ops.original_position() as usize;
                    body.clear();
                    body.extend_from_slice(&wasm[function.range().start as usize..begin]);
                    segments += meter_body(&mut body, &mut ops, wasm, fuel_index)?;
                    code.raw(&body);
                }
                let mut count = Vec::new();
                code.len().encode(&mut count);
                code_size.1 = code.byte_len() + count.len();
                module.section(&code);
            }
            _ => {
                module.section(&RawSection {
                    id,
                    data: &wasm[start..end],
                });
            }
        }
    }
    ensure!(has_memory, "guest must define one memory");
    if needs_global {
        append_global(&mut module, 0, &[]);
    }
    if needs_export {
        append_export(&mut module, 0, &[], fuel_index);
    }
    Ok(MeteredModule {
        bytes: module.finish(),
        segments,
        code_size,
    })
}

fn imported_globals(reader: wasmparser::ImportSectionReader<'_>) -> Result<u32> {
    let mut count = 0;
    for import in reader.into_imports() {
        let kind = import?.ty;
        ensure!(
            !matches!(kind, TypeRef::Memory(_)),
            "imported guest memory is not supported"
        );
        count += u32::from(matches!(kind, TypeRef::Global(_)));
    }
    Ok(count)
}

fn append_global(module: &mut Module, count: u32, items: &[u8]) {
    let mut data = Vec::new();
    (count + 1).encode(&mut data);
    data.extend_from_slice(items);
    GlobalType {
        val_type: ValType::I64,
        mutable: true,
        shared: false,
    }
    .encode(&mut data);
    ConstExpr::i64_const(0).encode(&mut data);
    module.section(&RawSection { id: 6, data: &data });
}

fn append_export(module: &mut Module, count: u32, items: &[u8], fuel_index: u32) {
    let mut data = Vec::new();
    (count + 1).encode(&mut data);
    data.extend_from_slice(items);
    FUEL_EXPORT.encode(&mut data);
    ExportKind::Global.encode(&mut data);
    fuel_index.encode(&mut data);
    module.section(&RawSection { id: 7, data: &data });
}

fn meter_body(
    output: &mut Vec<u8>,
    ops: &mut wasmparser::OperatorsReader<'_>,
    wasm: &[u8],
    fuel_index: u32,
) -> Result<usize> {
    let mut start = ops.original_position() as usize;
    let mut cost = 0;
    let mut segments = 0;
    while !ops.eof() {
        let pos = ops.original_position() as usize;
        let op = ops.read()?;
        let end = ops.original_position() as usize;
        // A branch to a loop enters after its header. A branch out of a block
        // or around an if arm enters after its delimiter.
        match op {
            Operator::Block { .. } | Operator::Loop { .. } | Operator::Else | Operator::End => {
                segments += debit(output, fuel_index, cost, &wasm[start..pos]);
                output.extend_from_slice(&wasm[pos..end]);
            }
            Operator::If { .. }
            | Operator::Br { .. }
            | Operator::BrIf { .. }
            | Operator::BrTable { .. }
            | Operator::Call { .. }
            | Operator::CallIndirect { .. }
            | Operator::Return
            | Operator::Unreachable => {
                segments += debit(output, fuel_index, cost + 1, &wasm[start..end]);
            }
            _ => {
                cost += 1;
                continue;
            }
        }
        cost = 0;
        start = end;
    }
    Ok(segments)
}

fn debit(output: &mut Vec<u8>, fuel: u32, cost: i64, original: &[u8]) -> usize {
    if cost == 0 {
        return 0;
    }
    for instruction in [
        Instruction::GlobalGet(fuel),
        Instruction::I64Const(cost),
        Instruction::I64LtU,
        Instruction::If(BlockType::Empty),
        Instruction::Unreachable,
        Instruction::End,
        Instruction::GlobalGet(fuel),
        Instruction::I64Const(cost),
        Instruction::I64Sub,
        Instruction::GlobalSet(fuel),
    ] {
        instruction.encode(output);
    }
    output.extend_from_slice(original);
    1
}
