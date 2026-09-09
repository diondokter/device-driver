//! Interner based on https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html

use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    mem,
    ops::Deref,
    sync::{LazyLock, Mutex},
};

static GLOBAL_INTERNER: LazyLock<Mutex<Interner>> =
    LazyLock::new(|| Mutex::new(Interner::with_capacity(1024 * 32)));

pub struct Interner {
    map: HashMap<&'static str, u32>,
    vec: Vec<&'static str>,
    buf: String,
    full: Vec<String>,
}

impl Interner {
    fn with_capacity(cap: usize) -> Interner {
        let cap = cap.next_power_of_two();
        let mut interner = Interner {
            map: HashMap::default(),
            vec: Vec::new(),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        };

        // Allocate a default Istr at 0
        let default_istr = interner.intern("");
        debug_assert_eq!(default_istr.0, 0);

        interner
    }

    pub fn intern(&mut self, name: &str) -> Istr {
        if let Some(&id) = self.map.get(name) {
            return Istr(id);
        }
        let name = unsafe { self.alloc(name) };
        let id = self.map.len() as u32;
        self.map.insert(name, id);
        self.vec.push(name);

        debug_assert!(self.lookup(Istr(id)) == name);
        debug_assert!(self.intern(name) == Istr(id));

        Istr(id)
    }

    pub fn lookup(&self, id: Istr) -> &'static str {
        self.vec[id.0 as usize]
    }

    unsafe fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();
        if cap < self.buf.len() + name.len() {
            let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }

        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            &self.buf[start..]
        };

        unsafe { &*(interned as *const str) }
    }
}

impl Drop for Interner {
    fn drop(&mut self) {
        panic!("Interner must not be dropped since it has given out static references")
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct Istr(u32);

impl Istr {
    pub fn as_str(self) -> &'static str {
        let interner = GLOBAL_INTERNER.lock().unwrap();
        interner.lookup(self)
    }
}

impl std::fmt::Debug for Istr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "({}){:?}", self.0, self.as_str())
        } else {
            write!(f, "{:?}", self.as_str())
        }
    }
}

impl std::fmt::Display for Istr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Deref for Istr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Istr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Istr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Hash for Istr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'a> From<&'a str> for Istr {
    fn from(value: &'a str) -> Self {
        value.intern()
    }
}

pub trait StrExt {
    fn intern(&self) -> Istr;
}
impl<T: AsRef<str>> StrExt for T {
    fn intern(&self) -> Istr {
        GLOBAL_INTERNER.lock().unwrap().intern(self.as_ref())
    }
}
