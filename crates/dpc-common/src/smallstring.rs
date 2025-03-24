use std::{fmt, mem::ManuallyDrop, ops::Deref};

#[cfg(target_pointer_width = "64")]
const MAX_INLINE_LEN: usize = 15;

#[cfg(target_pointer_width = "32")]
const MAX_INLINE_LEN: usize = 7;

#[derive(Clone, Copy)]
#[repr(C)]
struct InlineString {
    len: u8,
    bytes: [u8; MAX_INLINE_LEN],
}

#[derive(Clone, Copy)]
#[repr(C)]
struct HeapString {
    len: usize,
    ptr: *mut u8,
}

#[repr(C)]
pub union SmallString {
    inline: InlineString,
    heap: HeapString,
}

unsafe impl Send for SmallString {}
unsafe impl Sync for SmallString {}

impl SmallString {
    fn is_inline(&self) -> bool {
        unsafe { self.inline.len > 127 }
    }
}

impl Default for SmallString {
    fn default() -> Self {
        Self {
            inline: InlineString {
                len: 0x80,
                bytes: [0; MAX_INLINE_LEN],
            },
        }
    }
}

impl Clone for SmallString {
    fn clone(&self) -> Self {
        if self.is_inline() {
            Self {
                inline: unsafe { self.inline },
            }
        } else {
            unsafe {
                let HeapString { len, ptr } = self.heap;
                let slice = std::slice::from_raw_parts_mut(ptr, len);
                let mut boxed = ManuallyDrop::new(Vec::from(slice).into_boxed_slice());
                let len = boxed.len();
                let ptr = boxed.as_mut_ptr();
                Self {
                    heap: HeapString { len, ptr },
                }
            }
        }
    }
}

impl From<&str> for SmallString {
    fn from(value: &str) -> Self {
        if value.len() <= MAX_INLINE_LEN {
            let mut bytes = [0; MAX_INLINE_LEN];
            bytes[..value.len()].copy_from_slice(value.as_bytes());
            Self {
                inline: InlineString {
                    len: 0x80 | value.len() as u8,
                    bytes,
                },
            }
        } else {
            assert!(value.len() <= usize::MAX >> 1);
            let mut boxed = ManuallyDrop::new(String::from(value).into_boxed_str());
            let len = boxed.len();
            let ptr = boxed.as_mut_ptr();
            Self {
                heap: HeapString { len, ptr },
            }
        }
    }
}

impl From<String> for SmallString {
    fn from(value: String) -> Self {
        if value.len() <= MAX_INLINE_LEN {
            let mut bytes = [0; MAX_INLINE_LEN];
            bytes[..value.len()].copy_from_slice(value.as_bytes());
            Self {
                inline: InlineString {
                    len: 0x80 | value.len() as u8,
                    bytes,
                },
            }
        } else {
            assert!(value.len() <= usize::MAX >> 1);
            let mut boxed = ManuallyDrop::new(value.into_boxed_str());
            let len = boxed.len();
            let ptr = boxed.as_mut_ptr();
            Self {
                heap: HeapString { len, ptr },
            }
        }
    }
}

impl Deref for SmallString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe {
            if self.is_inline() {
                let InlineString { len, ref bytes } = self.inline;
                std::str::from_utf8_unchecked(&bytes[..((len & 0x7F) as usize)])
            } else {
                let HeapString { len, ptr } = self.heap;
                let slice = std::slice::from_raw_parts(ptr, len);
                std::str::from_utf8_unchecked(slice)
            }
        }
    }
}

impl fmt::Display for SmallString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl fmt::Debug for SmallString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl Drop for SmallString {
    fn drop(&mut self) {
        if !self.is_inline() {
            unsafe {
                let HeapString { len, ptr } = self.heap;
                let slice = std::slice::from_raw_parts_mut(ptr, len);
                drop(Box::from_raw(slice));
            }
        }
    }
}
