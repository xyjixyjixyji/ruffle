use gc_arena::Collect;

use crate::avm2::{dynamic_map::DynamicKey, Value};

const LOCAL_PROP_CACHE_SIZE: usize = 8;

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct LocalPropertyCache<'gc> {
    /// Cache for local properties.
    cache: [(DynamicKey<'gc>, Value<'gc>); LOCAL_PROP_CACHE_SIZE],

    /// Current cache size.
    cache_size: usize,

    /// The next cache (to evict) slot pointer. It is used once the cache is full.
    next_slot: usize,
}

impl<'gc> LocalPropertyCache<'gc> {
    pub fn new() -> Self {
        Self {
            cache: [(DynamicKey::Uint(0), Value::Null); 8],
            cache_size: 0,
            next_slot: 0,
        }
    }

    #[inline(always)]
    fn lookup_index(&self, key: DynamicKey<'gc>) -> Option<usize> {
        for i in 0..self.cache_size {
            if self.cache[i].0 == key {
                return Some(i);
            }
        }
        None
    }

    #[inline]
    pub fn lookup(&self, key: &DynamicKey<'gc>) -> Option<Value<'gc>> {
        self.lookup_index(*key).map(|i| self.cache[i].1)
    }

    #[inline]
    pub fn insert(&mut self, key: DynamicKey<'gc>, value: Value<'gc>) {
        if let Some(i) = self.lookup_index(key) {
            self.cache[i].1 = value;
            return;
        }

        // Simple FIFO after it's filled.
        let idx = if self.cache_size < LOCAL_PROP_CACHE_SIZE {
            let ret = self.cache_size;
            self.cache_size += 1;
            ret
        } else {
            let ret = self.next_slot;
            self.next_slot = (self.next_slot + 1) % LOCAL_PROP_CACHE_SIZE;
            ret
        };

        self.cache[idx] = (key, value);
    }

    #[inline]
    pub fn delete(&mut self, key: &DynamicKey<'gc>) -> bool {
        if let Some(i) = self.lookup_index(*key) {
            self.cache.swap(i, self.cache_size - 1);
            self.cache_size -= 1;
            return true;
        }

        false
    }
}
