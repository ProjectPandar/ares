use boa_engine::{
    Context, NativeFunction, Source, js_string,
    object::builtins::{AlignedVec, JsArrayBuffer, JsUint8Array},
};

pub struct Script(Context);

pub fn probe() -> [i32; 3] {
    let mut context = Context::default();
    context.runtime_limits_mut().set_loop_iteration_limit(1_000);
    let trapped = context
        .eval(Source::from_bytes(
            "var count = 0; while (count < 100000) count++;",
        ))
        .is_err();
    let count = context
        .eval(Source::from_bytes("count"))
        .unwrap()
        .as_number()
        .unwrap() as i32;
    [i32::from(trapped), count, -1]
}

impl Script {
    pub fn new(source: &str) -> Self {
        let mut context = Context::default();
        context
            .register_global_builtin_callable(
                js_string!("__plugin_math"),
                3,
                NativeFunction::from_fn_ptr(|_, args, context| {
                    let code = args[0].to_i32(context)?;
                    let x = args[1].to_number(context)?;
                    let y = args[2].to_number(context)?;
                    Ok(super::math(code, x, y).into())
                }),
            )
            .unwrap();
        context.eval(Source::from_bytes(source)).unwrap();
        Self(context)
    }

    pub fn run(&mut self, name: &str, bytes: &[u8]) -> Vec<u8> {
        let c = &mut self.0;
        let mut block = AlignedVec::with_capacity(64, bytes.len());
        block.extend_from_slice(bytes);
        let buffer = JsArrayBuffer::from_byte_block(block, c).unwrap();
        let input = JsUint8Array::from_array_buffer(buffer, c).unwrap();
        let callable = c.global_object().get(js_string!(name), c).unwrap();
        let func = callable.as_callable().unwrap();
        let value = func.call(&Default::default(), &[input.into()], c).unwrap();
        let array = JsUint8Array::from_object(value.as_object().unwrap()).unwrap();
        let offset = array.byte_offset(c).unwrap();
        let len = array.byte_length(c).unwrap();
        let buffer = array.buffer(c).unwrap().as_object().unwrap();
        let buffer = JsArrayBuffer::from_object(buffer).unwrap();
        buffer.data().unwrap()[offset..offset + len].to_vec()
    }
}
