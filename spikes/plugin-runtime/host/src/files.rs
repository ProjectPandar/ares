use anyhow::{Result, anyhow, bail};
use std::collections::BTreeMap;
use wasmi::{Caller, Engine, Linker, Memory, Module, Store, TypedFunc};

const WASI: &str = "wasi_snapshot_preview1";
const BADF: i32 = 8;
const FAULT: i32 = 21;
const NOENT: i32 = 44;
const NOSPC: i32 = 51;
const NOTCAPABLE: i32 = 76;
const QUOTA: usize = 1024;
const FIXTURE: &[u8] = b"virtual root specimen";

#[derive(Default)]
struct Root {
    snapshot: BTreeMap<String, Vec<u8>>,
    staged: BTreeMap<String, Vec<u8>>,
    fds: BTreeMap<i32, Open>,
}

enum Open {
    Read { bytes: Vec<u8>, offset: usize },
    Write { path: String, bytes: Vec<u8> },
}

fn namespace(path: &str) -> Result<(&str, &str), i32> {
    if path.contains(['\\', '\0', ':'])
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err(NOTCAPABLE);
    }
    match path.split_once('/') {
        Some((root @ ("package" | "data"), name)) => Ok((root, name)),
        _ => Err(NOTCAPABLE),
    }
}

impl Root {
    fn read(&self, path: &str) -> Result<&[u8], i32> {
        match namespace(path)? {
            ("package", "start.txt") => Ok(FIXTURE),
            ("data", name) => self.snapshot.get(name).map(Vec::as_slice).ok_or(NOENT),
            _ => Err(NOENT),
        }
    }

    fn write(&mut self, path: &str, bytes: Vec<u8>) -> Result<(), i32> {
        let (root, name) = namespace(path)?;
        if root != "data" {
            return Err(NOTCAPABLE);
        }
        let prior: usize = self
            .snapshot
            .iter()
            .filter(|(key, _)| key.as_str() != name && !self.staged.contains_key(*key))
            .map(|(_, value)| value.len())
            .sum();
        let writes: usize = self
            .staged
            .iter()
            .filter(|(key, _)| key.as_str() != name)
            .map(|(_, value)| value.len())
            .sum();
        if prior + writes + bytes.len() > QUOTA {
            return Err(NOSPC);
        }
        self.staged.insert(name.to_owned(), bytes);
        Ok(())
    }

    fn open(&mut self, path: &str, writable: bool) -> Result<i32, i32> {
        let open = if writable {
            if namespace(path)?.0 != "data" {
                return Err(NOTCAPABLE);
            }
            self.write(path, Vec::new())?;
            Open::Write {
                path: path.to_owned(),
                bytes: Vec::new(),
            }
        } else {
            Open::Read {
                bytes: self.read(path)?.to_vec(),
                offset: 0,
            }
        };
        let fd = (4..).find(|fd| !self.fds.contains_key(fd)).unwrap();
        self.fds.insert(fd, open);
        Ok(fd)
    }
}

fn memory(cx: &Caller<'_, Root>) -> Result<Memory, i32> {
    cx.get_export("memory")
        .and_then(wasmi::Extern::into_memory)
        .ok_or(FAULT)
}

fn read(cx: &Caller<'_, Root>, ptr: i32, len: i32) -> Result<Vec<u8>, i32> {
    let n = usize::try_from(len).map_err(|_| FAULT)?;
    let offset = usize::try_from(ptr).map_err(|_| FAULT)?;
    let mem = memory(cx)?;
    let end = offset.checked_add(n).ok_or(FAULT)?;
    Ok(mem.data(cx).get(offset..end).ok_or(FAULT)?.to_vec())
}

fn write(cx: &mut Caller<'_, Root>, pos: i32, data: &[u8]) -> Result<(), i32> {
    memory(cx)?
        .write(cx, pos as u32 as usize, data)
        .map_err(|_| FAULT)
}

fn path(cx: &Caller<'_, Root>, ptr: i32, len: i32) -> Result<String, i32> {
    if !(1..=256).contains(&len) {
        return Err(NOTCAPABLE);
    }
    String::from_utf8(read(cx, ptr, len)?).map_err(|_| NOTCAPABLE)
}

fn io_slices(cx: &Caller<'_, Root>, ptr: i32, len: i32) -> Result<Vec<(i32, i32)>, i32> {
    if !(0..=32).contains(&len) {
        return Err(FAULT);
    }
    Ok(read(cx, ptr, len * 8)?
        .as_chunks::<8>()
        .0
        .iter()
        .map(|buf| {
            (
                i32::from_le_bytes(buf[..4].try_into().unwrap()),
                i32::from_le_bytes(buf[4..].try_into().unwrap()),
            )
        })
        .collect())
}

fn custom(linker: &mut Linker<Root>) -> Result<()> {
    linker.func_wrap(
        "ares",
        "file_read",
        |mut cx: Caller<'_, Root>, ptr, len, out, capacity: i32| -> i32 {
            let mut work = || {
                let path = path(&cx, ptr, len)?;
                let data = cx.data().read(&path)?.to_vec();
                if capacity < data.len() as i32 {
                    return Err(FAULT);
                }
                write(&mut cx, out, &data)?;
                Ok(data.len() as i32)
            };
            (work)().unwrap_or_else(|error| -error)
        },
    )?;
    linker.func_wrap(
        "ares",
        "file_write",
        |mut cx: Caller<'_, Root>, ptr, len, input, size: i32| -> i32 {
            let mut work = || {
                let path = path(&cx, ptr, len)?;
                if size > QUOTA as i32 {
                    return Err(NOSPC);
                }
                let data = read(&cx, input, size)?;
                cx.data_mut().write(&path, data)?;
                Ok(size)
            };
            (work)().unwrap_or_else(|error| -error)
        },
    )?;
    Ok(())
}

fn status(f: impl FnOnce() -> Result<(), i32>) -> i32 {
    f().err().unwrap_or(0)
}

fn fd_read(mut cx: Caller<'_, Root>, fd: i32, ptr: i32, len: i32, out: i32) -> Result<(), i32> {
    let mut total = 0_u32;
    for (ptr, len) in io_slices(&cx, ptr, len)? {
        let Open::Read { bytes, offset } = cx.data_mut().fds.get_mut(&fd).ok_or(BADF)? else {
            return Err(BADF);
        };
        let size = (len as u32 as usize).min(bytes.len() - *offset);
        let buf = bytes[*offset..*offset + size].to_vec();
        *offset += size;
        write(&mut cx, ptr, &buf)?;
        total += size as u32;
    }
    write(&mut cx, out, &total.to_le_bytes())
}

fn fd_write(mut cx: Caller<'_, Root>, fd: i32, ptr: i32, len: i32, out: i32) -> Result<(), i32> {
    let mut added = Vec::new();
    for (ptr, len) in io_slices(&cx, ptr, len)? {
        if len < 0 || added.len() + len as usize > QUOTA {
            return Err(NOSPC);
        }
        added.extend(read(&cx, ptr, len)?);
    }
    let Open::Write { path, bytes } = cx.data().fds.get(&fd).ok_or(BADF)? else {
        return Err(BADF);
    };
    let path = path.clone();
    let mut updated = bytes.clone();
    updated.extend_from_slice(&added);
    cx.data_mut().write(&path, updated.clone())?;
    let Some(Open::Write { bytes, .. }) = cx.data_mut().fds.get_mut(&fd) else {
        unreachable!()
    };
    *bytes = updated;
    write(&mut cx, out, &(added.len() as u32).to_le_bytes())
}

fn wasi_p1(linker: &mut Linker<Root>) -> Result<()> {
    linker.func_wrap(
        WASI,
        "fd_prestat_get",
        |mut cx: Caller<'_, Root>, fd: i32, out| {
            status(|| {
                if fd != 3 {
                    return Err(BADF);
                }
                write(&mut cx, out, &[0, 0, 0, 0, 1, 0, 0, 0])
            })
        },
    )?;
    linker.func_wrap(
        WASI,
        "fd_prestat_dir_name",
        |mut cx: Caller<'_, Root>, fd: i32, out, len: i32| {
            status(|| {
                if fd != 3 || len < 1 {
                    return Err(BADF);
                }
                write(&mut cx, out, b".")
            })
        },
    )?;
    linker.func_wrap(
        WASI,
        "path_open",
        |mut cx: Caller<'_, Root>,
         fd: i32,
         _: i32,
         ptr,
         len,
         _: i32,
         rights: i64,
         _: i64,
         _: i32,
         out| {
            status(|| {
                if fd != 3 {
                    return Err(BADF);
                }
                let path = path(&cx, ptr, len)?;
                let opened = cx.data_mut().open(&path, rights & (1 << 6) != 0)?;
                write(&mut cx, out, &opened.to_le_bytes())
            })
        },
    )?;
    linker.func_wrap(WASI, "fd_close", |mut cx: Caller<'_, Root>, fd: i32| {
        status(|| cx.data_mut().fds.remove(&fd).map(|_| ()).ok_or(BADF))
    })?;
    linker.func_wrap(
        WASI,
        "fd_filestat_get",
        |mut cx: Caller<'_, Root>, fd: i32, out| {
            status(|| {
                let size = match cx.data().fds.get(&fd).ok_or(BADF)? {
                    Open::Read { bytes, .. } | Open::Write { bytes, .. } => bytes.len() as u64,
                };
                let mut stat = [0; 64];
                stat[16] = 4;
                stat[32..40].copy_from_slice(&size.to_le_bytes());
                write(&mut cx, out, &stat)
            })
        },
    )?;
    linker.func_wrap(
        WASI,
        "fd_fdstat_get",
        |mut cx: Caller<'_, Root>, fd: i32, out| {
            status(|| {
                let kind = if fd == 3 {
                    3
                } else if cx.data().fds.contains_key(&fd) {
                    4
                } else {
                    return Err(BADF);
                };
                let mut result = [0xff; 24];
                result[..8].fill(0);
                result[0] = kind;
                write(&mut cx, out, &result)
            })
        },
    )?;
    linker.func_wrap(
        WASI,
        "fd_read",
        |cx: Caller<'_, Root>, fd, ptr, len, out| status(|| fd_read(cx, fd, ptr, len, out)),
    )?;
    linker.func_wrap(
        WASI,
        "fd_write",
        |cx: Caller<'_, Root>, fd, ptr, len, out| status(|| fd_write(cx, fd, ptr, len, out)),
    )?;
    linker.func_wrap(
        WASI,
        "environ_sizes_get",
        |mut cx: Caller<'_, Root>, count, size| {
            status(|| {
                write(&mut cx, count, &[0; 4])?;
                write(&mut cx, size, &[0; 4])
            })
        },
    )?;
    linker.func_wrap(WASI, "environ_get", |_: i32, _: i32| 0_i32)?;
    linker.func_wrap(WASI, "proc_exit", |code: i32| -> Result<(), wasmi::Error> {
        Err(wasmi::Error::i32_exit(code))
    })?;
    Ok(())
}

pub struct FileProbe {
    store: Store<Root>,
    hook: TypedFunc<(), i32>,
}

impl FileProbe {
    pub fn new(wasm: &[u8], wasi: bool, prior: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let mut root = Root::default();
        root.snapshot.insert("result.txt".into(), prior.to_vec());
        let mut store = Store::new(&engine, root);
        let mut linker = Linker::new(&engine);
        if wasi {
            wasi_p1(&mut linker)?;
        } else {
            custom(&mut linker)?;
        }
        let module = Module::new(&engine, wasm)?;
        let instance = linker.instantiate_and_start(&mut store, &module)?;
        let hook = instance.get_typed_func(&store, "read_write")?;
        Ok(Self { store, hook })
    }

    pub fn run(&mut self, iterations: usize) -> Result<()> {
        let expected: i32 = FIXTURE.iter().map(|b| i32::from(*b)).sum();
        for _ in 0..iterations {
            if self.hook.call(&mut self.store, ())? != expected {
                bail!("file round trip gave a different checksum");
            }
        }
        Ok(())
    }

    pub fn finish(&self, prior: &[u8]) -> Result<Vec<u8>> {
        let root = self.store.data();
        if root.read("data/result.txt") != Ok(prior)
            || root.read("/package/start.txt").is_ok()
            || root.read("data/../package/start.txt").is_ok()
        {
            bail!("invalid virtual root visibility");
        }
        let staged = root
            .staged
            .get("result.txt")
            .ok_or_else(|| anyhow!("no staged file"))?;
        if staged != FIXTURE {
            bail!("staged file does not match package bytes");
        }
        Ok(staged.clone())
    }
}
