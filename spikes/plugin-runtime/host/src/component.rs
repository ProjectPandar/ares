use anyhow::Result;
use std::{fs, path::Path};
use wasmtime::{
    Engine, Store,
    component::{Component, Linker},
};

const WRAPPER: &str = r#"
(component
  (core module $m
    (memory (export "memory") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      i32.const 1024)
    (func (export "perimeters") (param i32 i32) (result i32)
      local.get 0 i32.const BYTE i32.store8
      i32.const 8 local.get 0 i32.store
      i32.const 12 local.get 1 i32.store
      i32.const 8))
  (core instance $i (instantiate $m))
  (alias core export $i "memory" (core memory $mem))
  (alias core export $i "realloc" (core func $alloc))
  (alias core export $i "perimeters" (core func $perimeters))
  (type $bytes (list u8))
  (func $hook (param "layer" $bytes) (result $bytes)
    (canon lift (core func $perimeters) (memory $mem) (realloc $alloc)))
  (export "perimeters" (func $hook)))
"#;

const LIMITS: &str = r#"
(module
  MEMORY
  (export "memory" (memory 0))
  (func (export "advance") (param $remaining i32)
    (block $end (loop $next
      local.get $remaining i32.eqz br_if $end
      i32.const 0 i32.const 0 i32.load i32.const 1 i32.add i32.store
      local.get $remaining i32.const 1 i32.sub local.set $remaining
      br $next)))
  (func (export "grow") (param i32) (result i32)
    local.get 0 memory.grow))
"#;

pub fn build(out: &Path) -> Result<()> {
    let limits = wat::parse_str(LIMITS.replace("MEMORY", "(memory 1)"))?;
    let budget = plugin_spike_host::Budget {
        memory: Some(2 * 65536),
        fuel: Some(1_000),
    };
    let jit = plugin_spike_host::jit::Host::new(budget).probe(&limits)?;
    let interpreter = plugin_spike_host::interpreter::Host::new(budget).probe(&limits)?;
    for [trapped, steps, grow] in [jit, interpreter] {
        assert_eq!((trapped, grow), (1, -1));
        assert!((1..100_000).contains(&steps));
    }
    fs::write(
        out.join("native-limits.json"),
        serde_json::to_vec(&serde_json::json!({"jit": jit, "interpreter": interpreter}))?,
    )?;
    fs::write(out.join("limits.wasm"), limits)?;
    fs::write(
        out.join("limits-browser.wasm"),
        wat::parse_str(LIMITS.replace("MEMORY", r#"(import "ares" "memory" (memory 1 2))"#))?,
    )?;
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    for (name, byte) in [("component.wasm", 7), ("component-alternate.wasm", 9)] {
        let wasm = wat::parse_str(WRAPPER.replace("BYTE", &byte.to_string()))?;
        let module = Component::new(&engine, &wasm)?;
        let instance = linker.instantiate(&mut store, &module)?;
        let hook = instance.get_typed_func::<(Vec<u8>,), (Vec<u8>,)>(&mut store, "perimeters")?;
        assert_eq!(hook.call(&mut store, (vec![0, 1, 2],))?.0, [byte, 1, 2]);
        fs::write(out.join(name), wasm)?;
    }
    Ok(())
}
