use anyhow::Result;
#[cfg(any(feature = "interpreter", feature = "jit"))]
use anyhow::bail;
pub use {plugin_spike_api as api, plugin_spike_samples as samples, plugin_spike_wire as wire};
pub mod direct;
#[cfg(feature = "files")]
pub mod files;
#[cfg(feature = "metering")]
pub mod metering;

pub struct Hosted<'a> {
    pub guest: &'a mut dyn Guest,
    pub declarations: &'a [api::OptionDeclaration],
}

impl api::Plugin for Hosted<'_> {
    type Error = anyhow::Error;

    fn options(&self) -> &[api::OptionDeclaration] {
        self.declarations
    }

    fn perimeters(&mut self, layer: api::Layer<'_>) -> Result<Vec<api::Edit>> {
        let output = self.guest.perimeters(&wire::encode_layer(layer))?;
        wire::decode_edits(&output).ok_or_else(|| anyhow::anyhow!("invalid edit response"))
    }

    fn gcode_post_process(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        self.guest.gcode(input)
    }
}

#[cfg(any(feature = "interpreter", feature = "jit"))]
enum Hook {
    Perimeters,
    Gcode,
}

#[derive(Clone, Copy, Default)]
pub struct Budget {
    pub fuel: Option<u64>,
    pub memory: Option<usize>,
}

pub trait Guest {
    fn has_source(&self) -> bool;
    fn set_source(&mut self, bytes: &[u8]) -> Result<()>;
    fn perimeters(&mut self, bytes: &[u8]) -> Result<Vec<u8>>;
    fn gcode(&mut self, bytes: &[u8]) -> Result<Vec<u8>>;
}

// The compatibility imports expose no process state. The engines can read
// a fixed epoch for initialization; JavaScript has no clock or RNG API.
#[cfg(any(feature = "interpreter", feature = "jit"))]
macro_rules! denied_wasi {
    ($linker:ident, $exit:expr) => {{
        let wasi = "wasi_snapshot_preview1";
        let unavailable = 52_i32;
        $linker.func_wrap(
            wasi,
            "clock_time_get",
            |mut caller: Caller<'_, StoreLimits>, _: i32, _: i64, output: i32| {
                let Some(memory) = caller.get_export("memory").and_then(Extern::into_memory) else {
                    return 21;
                };
                if memory
                    .write(&mut caller, output as u32 as usize, &[0; 8])
                    .is_ok()
                {
                    0
                } else {
                    21
                }
            },
        )?;
        $linker.func_wrap(wasi, "random_get", move |_: i32, _: i32| unavailable)?;
        $linker.func_wrap(wasi, "fd_write", move |_: i32, _: i32, _: i32, _: i32| {
            unavailable
        })?;
        $linker.func_wrap(wasi, "environ_get", move |_: i32, _: i32| unavailable)?;
        $linker.func_wrap(wasi, "environ_sizes_get", move |_: i32, _: i32| unavailable)?;
        $linker.func_wrap(
            wasi,
            "poll_oneoff",
            move |_: i32, _: i32, _: i32, _: i32| unavailable,
        )?;
        $linker.func_wrap(wasi, "sched_yield", move || unavailable)?;
        $linker.func_wrap(wasi, "proc_exit", $exit)?;
        $linker.func_wrap("env", "__rquickjs_host_now_us", || 0.0_f64)?;
    }};
}

#[cfg(any(feature = "interpreter", feature = "jit"))]
macro_rules! backend {
    ($runtime:ident, $instantiate:ident, $exit:expr) => {
        use $runtime::{
            Caller, Config, Engine, Extern, Linker, Memory, Module, Store, StoreLimits,
            StoreLimitsBuilder, TypedFunc,
        };
        #[cfg(feature = "metering")]
        use $runtime::{Global, Val};

        pub struct Host {
            engine: Engine,
            budget: Budget,
            #[cfg(feature = "metering")]
            portable: Option<(u32, i64)>,
        }
        impl Default for Host {
            fn default() -> Self {
                Self::new(Budget::default())
            }
        }

        impl Host {
            pub fn new(budget: Budget) -> Self {
                let mut config = Config::default();
                config.consume_fuel(budget.fuel.is_some());
                let engine = $runtime::Engine::new(&config);
                Self {
                    engine: engine_result(engine),
                    budget,
                    #[cfg(feature = "metering")]
                    portable: None,
                }
            }

            #[cfg(feature = "metering")]
            pub fn portable(max_pages: u32, fuel: i64) -> Self {
                Self {
                    portable: Some((max_pages, fuel)),
                    ..Self::default()
                }
            }

            fn prepare(&self, wasm: &[u8]) -> Result<(Store<StoreLimits>, $runtime::Instance)> {
                #[cfg(feature = "metering")]
                let instrumented = self
                    .portable
                    .map(|(pages, _)| metering::instrument(wasm, pages))
                    .transpose()?;
                #[cfg(feature = "metering")]
                let wasm = instrumented
                    .as_ref()
                    .map_or(wasm, |module| module.bytes.as_slice());
                let module = Module::new(&self.engine, wasm)?;
                let mut limits = StoreLimitsBuilder::new();
                if let Some(memory) = self.budget.memory {
                    limits = limits.memory_size(memory).memories(1);
                }
                let mut store = Store::new(&self.engine, limits.build());
                if self.budget.memory.is_some() {
                    store.limiter(|limits| limits);
                }
                if let Some(fuel) = self.budget.fuel {
                    store.set_fuel(fuel)?;
                }
                let mut linker = Linker::new(&self.engine);
                denied_wasi!(linker, $exit);
                let exports = linker.$instantiate(&mut store, &module)?;
                Ok((store, exports))
            }

            pub fn probe(&self, wasm: &[u8]) -> Result<[i32; 3]> {
                let (mut store, exports) = self.prepare(wasm)?;
                let advance = exports.get_typed_func::<i32, ()>(&mut store, "advance")?;
                let memory = exports.get_memory(&mut store, "memory").unwrap();
                let trapped = advance.call(&mut store, 100_000).is_err();
                let mut steps = [0; 4];
                memory.read(&store, 0, &mut steps)?;
                if self.budget.fuel.is_some() {
                    store.set_fuel(1_000)?;
                }
                let grow = exports.get_typed_func::<i32, i32>(&mut store, "grow")?;
                let growth = grow.call(&mut store, 2)?;
                Ok([i32::from(trapped), i32::from_le_bytes(steps), growth])
            }

            #[cfg(feature = "metering")]
            fn fuel_global(
                &self,
                store: &mut Store<StoreLimits>,
                exports: $runtime::Instance,
            ) -> Result<Option<(Global, i64)>> {
                self.portable
                    .map(|(_, fuel)| {
                        let global = exports
                            .get_global(store, metering::FUEL_EXPORT)
                            .ok_or_else(|| anyhow::anyhow!("no guest fuel global"))?;
                        Ok((global, fuel))
                    })
                    .transpose()
            }

            #[cfg(feature = "metering")]
            pub fn portable_probe(&self, wasm: &[u8]) -> Result<[i32; 4]> {
                let (mut store, exports) = self.prepare(wasm)?;
                let (global, fuel) = self
                    .fuel_global(&mut store, exports)?
                    .ok_or_else(|| anyhow::anyhow!("no portable fuel budget"))?;
                let advance = exports.get_typed_func::<i32, ()>(&mut store, "advance")?;
                let memory = exports.get_memory(&mut store, "memory").unwrap();
                global.set(&mut store, Val::I64(fuel))?;
                let trapped = advance.call(&mut store, 100_000).is_err();
                let mut steps = [0; 4];
                memory.read(&store, 0, &mut steps)?;
                let grow = exports.get_typed_func::<i32, i32>(&mut store, "grow")?;
                global.set(&mut store, Val::I64(fuel))?;
                let at_cap = grow.call(&mut store, 1)?;
                let above_cap = grow.call(&mut store, 1)?;
                Ok([
                    i32::from(trapped),
                    i32::from_le_bytes(steps),
                    at_cap,
                    above_cap,
                ])
            }

            pub fn load(&self, wasm: &[u8], source: Option<&[u8]>) -> Result<Instance> {
                let (mut store, exports) = self.prepare(wasm)?;
                let memory = exports
                    .get_memory(&mut store, "memory")
                    .ok_or_else(|| anyhow::anyhow!("no guest memory"))?;
                let reserve = exports.get_typed_func::<u32, u32>(&mut store, "reserve_input")?;
                let perimeters = exports.get_typed_func::<(), u64>(&mut store, "perimeters")?;
                let gcode = exports.get_typed_func::<(), u64>(&mut store, "gcode_post_process")?;
                let source_hook = exports
                    .get_typed_func::<(), ()>(&mut store, "set_source")
                    .ok();
                #[cfg(feature = "metering")]
                let portable = self.fuel_global(&mut store, exports)?;
                let mut instance = Instance {
                    store,
                    memory,
                    reserve,
                    perimeters,
                    gcode,
                    source: source_hook,
                    fuel: self.budget.fuel,
                    #[cfg(feature = "metering")]
                    portable,
                };
                if let Some(source) = source {
                    instance.set_source(source)?;
                }
                Ok(instance)
            }
        }

        pub struct Instance {
            store: Store<StoreLimits>,
            fuel: Option<u64>,
            #[cfg(feature = "metering")]
            portable: Option<(Global, i64)>,
            memory: Memory,
            reserve: TypedFunc<u32, u32>,
            perimeters: TypedFunc<(), u64>,
            gcode: TypedFunc<(), u64>,
            source: Option<TypedFunc<(), ()>>,
        }

        impl Instance {
            fn input(&mut self, bytes: &[u8]) -> Result<()> {
                if let Some(fuel) = self.fuel {
                    self.store.set_fuel(fuel)?;
                }
                #[cfg(feature = "metering")]
                if let Some((global, fuel)) = self.portable {
                    global.set(&mut self.store, Val::I64(fuel))?;
                }
                let pos = self
                    .reserve
                    .call(&mut self.store, bytes.len().try_into()?)?;
                self.memory.write(&mut self.store, pos as usize, bytes)?;
                Ok(())
            }

            fn hook(&mut self, bytes: &[u8], hook: Hook) -> Result<Vec<u8>> {
                self.input(bytes)?;
                let function = match hook {
                    Hook::Perimeters => &self.perimeters,
                    Hook::Gcode => &self.gcode,
                };
                let packed = function.call(&mut self.store, ())?;
                let len = (packed >> 32) as usize;
                let start = packed as u32 as usize;
                let bytes = self.memory.data(&self.store);
                let output = bytes
                    .get(start..)
                    .and_then(|tail| tail.get(..len))
                    .ok_or_else(|| anyhow::anyhow!("guest returned an invalid buffer"))?;
                Ok(output.to_vec())
            }
        }

        impl Guest for Instance {
            fn has_source(&self) -> bool {
                self.source.is_some()
            }

            fn set_source(&mut self, bytes: &[u8]) -> Result<()> {
                self.input(bytes)?;
                let Some(source) = &self.source else {
                    bail!("no source export")
                };
                source.call(&mut self.store, ())?;
                Ok(())
            }

            fn perimeters(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
                self.hook(bytes, Hook::Perimeters)
            }

            fn gcode(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
                self.hook(bytes, Hook::Gcode)
            }
        }
    };
}

#[cfg(feature = "interpreter")]
pub mod interpreter {
    use super::*;
    fn engine_result(engine: wasmi::Engine) -> wasmi::Engine {
        engine
    }
    backend!(wasmi, instantiate_and_start, |code: i32| -> Result<
        (),
        wasmi::Error,
    > {
        Err(wasmi::Error::i32_exit(code))
    });
}

#[cfg(all(not(target_arch = "wasm32"), feature = "jit"))]
pub mod jit {
    use super::*;
    fn engine_result(engine: wasmtime::Result<wasmtime::Engine>) -> wasmtime::Engine {
        engine.unwrap()
    }
    backend!(wasmtime, instantiate, |code: i32| -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(format!("guest proc_exit: {code}")))
    });
}
