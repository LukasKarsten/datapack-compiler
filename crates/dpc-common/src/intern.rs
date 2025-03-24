use std::{
    fmt,
    hash::{BuildHasher, BuildHasherDefault},
    num::NonZeroU32,
};

use hashbrown::{HashMap, hash_map::RawEntryMut};
use rustc_hash::FxHasher;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(pub NonZeroU32);

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol(0x{:04X})", self.0)
    }
}

pub trait Interner {
    fn intern(&mut self, string: &str) -> Symbol;
    fn resolve(&self, symbol: Symbol) -> Option<&str>;
    unsafe fn resolve_unchecked(&self, symbol: Symbol) -> &str {
        self.resolve(symbol).unwrap()
    }
}

struct Buffer {
    current: Box<[u8]>,
    length: usize,
    full: Vec<Box<[u8]>>,
}

impl Buffer {
    fn new() -> Self {
        Self {
            current: vec![0; 4096].into_boxed_slice(),
            length: 0,
            full: Vec::new(),
        }
    }

    fn insert(&mut self, string: &str) -> BufferSlice {
        let len = string.len();

        if self.current.len() - self.length < len {
            let new_size = std::cmp::max(self.current.len() + 1, len).next_power_of_two();
            let new_buffer = vec![0; new_size].into_boxed_slice();
            let old_buffer = std::mem::replace(&mut self.current, new_buffer);
            self.full.push(old_buffer);
            self.length = 0;
        }

        unsafe {
            let ptr = self.current.as_mut_ptr().add(self.length);
            ptr.copy_from(string.as_ptr(), len);
            self.length += len;

            BufferSlice { ptr, len }
        }
    }
}

#[derive(Clone, Copy)]
struct BufferSlice {
    ptr: *const u8,
    len: usize,
}

unsafe impl Send for BufferSlice {}
unsafe impl Sync for BufferSlice {}

impl BufferSlice {
    unsafe fn as_str(self) -> &'static str {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

pub struct StaticInterner<H = BuildHasherDefault<FxHasher>> {
    build_hasher: H,
    symbols: HashMap<(Symbol, BufferSlice), (), ()>,
    entries: Vec<BufferSlice>,
    buffer: Buffer,
}

impl<H: Default> Default for StaticInterner<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: Default> StaticInterner<H> {
    pub fn new() -> Self {
        Self::with_hasher(H::default())
    }
}

impl<H> StaticInterner<H> {
    pub fn with_hasher(build_hasher: H) -> Self {
        Self {
            build_hasher,
            symbols: HashMap::with_hasher(()),
            entries: Vec::new(),
            buffer: Buffer::new(),
        }
    }
}

impl<H: BuildHasher> Interner for StaticInterner<H> {
    fn intern(&mut self, string: &str) -> Symbol {
        let hash = self.build_hasher.hash_one(string);
        let entry = self
            .symbols
            .raw_entry_mut()
            .from_hash(hash, |(_, view)| unsafe { string == view.as_str() });
        match entry {
            RawEntryMut::Occupied(occupied) => occupied.key().0,
            RawEntryMut::Vacant(vacant) => {
                let string_view = self.buffer.insert(string);

                self.entries.push(string_view);

                let id: u32 = self
                    .entries
                    .len()
                    .try_into()
                    .expect("too many strings interned");
                let symbol = Symbol(
                    NonZeroU32::new(id).expect("entries.len() must be greater than 0 after push"),
                );

                vacant.insert_with_hasher(hash, (symbol, string_view), (), |&(_, other)| {
                    self.build_hasher.hash_one(unsafe { other.as_str() })
                });

                symbol
            }
        }
    }

    fn resolve(&self, symbol: Symbol) -> Option<&str> {
        (usize::try_from(symbol.0.get()).unwrap() <= self.entries.len())
            .then(|| unsafe { self.resolve_unchecked(symbol) })
    }

    unsafe fn resolve_unchecked(&self, symbol: Symbol) -> &str {
        // Safety: The lifetime of the returned string is shortened to the &self lifetime, for
        // which the string is always valid.
        unsafe {
            self.entries
                .get_unchecked(usize::try_from(symbol.0.get() - 1).unwrap())
                .as_str()
        }
    }
}
