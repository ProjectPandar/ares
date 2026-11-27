use rquickjs::{Context, Function, Runtime, TypedArray};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[unsafe(no_mangle)]
extern "C" fn __rquickjs_host_now_us() -> f64 {
    0.0
}

fn runtime() -> Runtime {
    let runtime = Runtime::new().unwrap();
    // The default 1 MiB bound can underflow the stack pointer of a core Wasm guest.
    runtime.set_max_stack_size(128 * 1024);
    runtime
}

pub fn probe() -> [i32; 3] {
    let runtime = runtime();
    let context = Context::full(&runtime).unwrap();
    let polls = std::cell::Cell::new(0);
    runtime.set_interrupt_handler(Some(Box::new(move || {
        polls.set(polls.get() + 1);
        polls.get() >= 4
    })));
    let trapped = context.with(|ctx| {
        ctx.eval::<(), _>("var count = 0; while (count < 100000) count++;")
            .is_err()
    });
    runtime.set_interrupt_handler(None);
    let count = context.with(|ctx| ctx.eval::<i32, _>("count").unwrap());
    runtime.set_memory_limit(1 << 20);
    let rejected = context.with(|ctx| ctx.eval::<(), _>("new ArrayBuffer(8 << 20)").is_err());
    [i32::from(trapped), count, i32::from(rejected)]
}

pub struct Script {
    context: Context,
    _runtime: Runtime,
}

impl Script {
    pub fn new(source: &str) -> Self {
        let runtime = runtime();
        let context = Context::full(&runtime).unwrap();
        context.with(|ctx| {
            let math = Function::new(ctx.clone(), super::math).unwrap();
            ctx.globals().set("__plugin_math", math).unwrap();
            ctx.eval::<(), _>(source).unwrap();
        });
        Self {
            context,
            _runtime: runtime,
        }
    }

    pub fn run(&mut self, name: &str, bytes: &[u8]) -> Vec<u8> {
        self.context.with(|ctx| {
            let input = TypedArray::<u8>::new_copy(ctx.clone(), bytes).unwrap();
            let func: Function = ctx.globals().get(name).unwrap();
            let output: TypedArray<u8> = func.call((input,)).unwrap();
            // The owned copy is made before any further JS call or buffer mutation.
            unsafe { output.as_bytes() }.unwrap().to_vec()
        })
    }
}
