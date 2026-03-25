use super::*;

use linear_map::LinearMap;

pub struct InterpolationCache {
    cache: LinearMap<Movement, Cached>,
}

struct Cached {
    baked: Interpolation<TransformLight>,
    relevant: bool,
}

impl InterpolationCache {
    pub fn new() -> Self {
        Self {
            cache: LinearMap::new(),
        }
    }

    /// Reset relevancy status of all cached elements.
    /// Call at the start of the frame before using the cache.
    pub fn update(&mut self) {
        for (_, cached) in self.cache.iter_mut() {
            cached.relevant = false;
        }
    }

    pub fn get_or_bake(&mut self, movement: &Movement) -> &Interpolation<TransformLight> {
        let cached = self
            .cache
            .entry(movement.clone())
            .or_insert_with(|| Cached {
                baked: movement.bake(),
                relevant: false,
            });
        cached.relevant = true;
        &cached.baked
    }

    /// Removes unused entries from the cache.
    /// Call at the end of the frame to keep the cached info relevant.
    pub fn clear_irrelevant(&mut self) {
        self.cache.retain(|_, cached| cached.relevant)
    }
}

impl Default for InterpolationCache {
    fn default() -> Self {
        Self::new()
    }
}
