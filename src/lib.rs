mod combined_getters;

#[doc(hidden)]
pub mod __private {
    pub use pastey::paste;
}

#[derive(thiserror::Error, Debug)]
#[error("Unique contraint violation on {field} for value {value:?}")]
pub struct UniqueContraintViolation<T> {
    pub field: &'static str,
    pub value: T,
}

/// Macro to define a multi-index map with unique and non-unique indexes
#[macro_export]
macro_rules! multi_index_container {
    (
        $(#[$meta:meta])*
        $vis:vis $map_name:ident<$value_type:ty> {
            $(unique $unique_name:ident: $unique_key_type:ty => |$unique_param:ident| $unique_expr:expr,)*
            $(non_unique $non_unique_name:ident: $non_unique_key_type:ty => |$non_unique_param:ident| $non_unique_expr:expr,)*
            $(unique_ordered $unique_ordered_name:ident: $unique_ordered_key_type:ty => |$unique_ordered_param:ident| $unique_ordered_expr:expr,)*
            $(non_unique_ordered $non_unique_ordered_name:ident: $non_unique_ordered_key_type:ty => |$non_unique_ordered_param:ident| $non_unique_ordered_expr:expr,)*
        }
    ) => {
        use multi_index_container::__private::paste;

        paste! {
            /// A typed index into the underlying storage of [`$map_name`].
            ///
            /// Wraps a `usize` to provide type safety, preventing accidental mix-ups
            /// between indices belonging to different maps.
            ///
            /// Derived traits:
            /// - [`Clone`], [`Copy`] — cheap to duplicate; pass by value freely.
            /// - [`PartialEq`], [`Eq`], [`Hash`] — usable as a map key or in sets.
            /// - [`Default`] — initialises to index `0`.
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            $vis struct [<$map_name StorageIndex>] (usize);

            impl [<$map_name StorageIndex>] {
                /// Returns the immediately following index.
                ///
                /// Useful when advancing through storage slots sequentially,
                /// for example when inserting a new entry at the next available position.
                ///
                /// # Panics
                /// Panics in debug mode if the inner `usize` overflows. In release mode
                /// the value wraps silently.
                pub fn next(&self) -> Self {
                    Self ( self.0 + 1 )
                }
            }
        }

        $(#[$meta])*
        #[doc = concat!("A map for storing `", stringify!($value_type), "` indexed by `", stringify!($storage_key_type), "`.")]
        $(
            #[doc = concat!("- Unique index `", stringify!($unique_name), "`: `", stringify!($unique_key_type), "`")]
        )*
        $(
            #[doc = concat!("- Non-unique index `", stringify!($non_unique_name), "`: `", stringify!($non_unique_key_type), "`")]
        )*
        $(
            #[doc = concat!("- Unique ordered index `", stringify!($unique_ordered_name), "`: `", stringify!($unique_ordered_key_type), "`")]
        )*
        $(
            #[doc = concat!("- Non-unique ordered index `", stringify!($non_unique_ordered_name), "`: `", stringify!($non_unique_ordered_key_type), "`")]
        )*
        $vis struct $map_name {


            #[doc = "Storage keys that have been freed and can be reused."]
            freed_storage_keys: Vec< paste! { [<$map_name StorageIndex>] }>,

            #[doc = "The next storage key to be assigned when no freed keys are available."]
            next_storage_key: paste! { [<$map_name StorageIndex>] },
            
            #[doc = concat!("Primary storage mapping each index to its `", stringify!($value_type), "`.")]
            storage: std::collections::HashMap<paste! { [<$map_name StorageIndex>] }, $value_type>,
            
            $(
                #[doc = concat!("Unique index `", stringify!($unique_name), "` mapping `", stringify!($unique_key_type), "` to a single storage index.")]
                $unique_name: std::collections::HashMap<$unique_key_type, paste! { [<$map_name StorageIndex>] }>,
            )*
            $(
                #[doc = concat!("Non-unique index `", stringify!($non_unique_name), "` mapping `", stringify!($non_unique_key_type), "` to a set of storage indices.")]
                $non_unique_name: std::collections::HashMap<$non_unique_key_type, std::collections::HashSet< paste! { [<$map_name StorageIndex>] } >>,
            )*
            $(
                #[doc = concat!("Unique ordered index `", stringify!($unique_ordered_name), "` mapping `", stringify!($unique_ordered_key_type), "` to a single storage index.")]
                $unique_ordered_name: std::collections::BTreeMap<$unique_ordered_key_type, paste! { [<$map_name StorageIndex>] } >,
            )*
            $(
                #[doc = concat!("Non-unique ordered index `", stringify!($non_unique_ordered_name), "` mapping `", stringify!($non_unique_ordered_key_type), "` to a set of storage indices.")]
                $non_unique_ordered_name: std::collections::BTreeMap<$non_unique_ordered_key_type, std::collections::HashSet< paste! { [<$map_name StorageIndex>] } >>,
            )*
        }

        impl $map_name {
            
            #[doc = concat!("Initialise an empty ", stringify!($value_type))]
            pub fn new() -> Self {
                Self {
                    freed_storage_keys: Vec::new(),
                    next_storage_key: Default::default(),
                    storage: std::collections::HashMap::new(),
                    $(
                        $unique_name: std::collections::HashMap::new(),
                    )*
                    $(
                        $non_unique_name: std::collections::HashMap::new(),
                    )*
                    $(
                        $unique_ordered_name: std::collections::BTreeMap::new(),
                    )*
                    $(
                        $non_unique_ordered_name: std::collections::BTreeMap::new(),
                    )*
                }
            }

            #[doc = "Get a reference to a value in the store based on the storage index."]
            pub fn get(&self, storage_key: &paste! { [<$map_name StorageIndex>] }) -> Option<&$value_type> {
                self.storage.get(storage_key)
            }

            #[doc = concat!("Inserts a `", stringify!($value_type), "` and update all indexes with the new type.")]
            pub fn insert(&mut self, value: $value_type) -> Result<(), multi_index_container::UniqueContraintViolation<$value_type>> {
                // Check unique constraints
                $(
                    let $unique_param = &value;
                    let unique_key = $unique_expr;
                    if self.$unique_name.contains_key(&unique_key) {
                        return Err(multi_index_container::UniqueContraintViolation { field: stringify!($unique_name), value } );
                    }
                )*
                $(
                    let $unique_ordered_param = &value;
                    let unique_key = $unique_ordered_expr;
                    if self.$unique_ordered_name.contains_key(&unique_key) {
                        return Err(multi_index_container::UniqueContraintViolation { field: stringify!($unique_ordered_name), value } );
                    }
                )*

                let storage_key = self.freed_storage_keys.pop().unwrap_or_else(
                    || {
                        let key = self.next_storage_key;
                        self.next_storage_key = self.next_storage_key.next();
                        key
                    }
                );

                // Insert into storage
                let storage_key_clone = storage_key;
                self.storage.insert(storage_key, value);

                // Update indexes
                let stored_value = self.storage.get(&storage_key_clone).unwrap();

                $(
                    let $unique_param = stored_value;
                    let unique_key = $unique_expr;
                    self.$unique_name.insert(unique_key, storage_key_clone);
                )*

                $(
                    let $non_unique_param = stored_value;
                    let non_unique_key = $non_unique_expr;
                    self.$non_unique_name
                        .entry(non_unique_key)
                        .or_default()
                        .insert(storage_key_clone);
                )*
                $(
                    let $unique_ordered_param = stored_value;
                    let unique_ordered_key = $unique_ordered_expr;
                    self.$unique_ordered_name.insert(unique_ordered_key, storage_key_clone);
                )*
                $(
                    let $non_unique_ordered_param = stored_value;
                    let non_unique_ordered_key = $non_unique_ordered_expr;
                    self.$non_unique_ordered_name
                        .entry(non_unique_ordered_key)
                        .or_default()
                        .insert(storage_key_clone);
                )*

                Ok(())
            }

            #[doc = concat!("Inserts a `", stringify!($value_type), "` overwriting any existing entries that conflict on unique indexes.")]
            pub fn insert_or_overwrite(&mut self, value: $value_type) {
                $(
                    let $unique_param = &value;
                    let unique_key = $unique_expr;
                    if let Some(&conflicting_storage_key) = self.$unique_name.get(&unique_key) {
                        self.remove(&conflicting_storage_key);
                    }
                )*
                $(
                    let $unique_ordered_param = &value;
                    let unique_ordered_key = $unique_ordered_expr;
                    if let Some(&conflicting_storage_key) = self.$unique_ordered_name.get(&unique_ordered_key) {
                        self.remove(&conflicting_storage_key);
                    }
                )*

                let storage_key = self.freed_storage_keys.pop().unwrap_or_else(
                    || {
                        let key = self.next_storage_key;
                        self.next_storage_key = self.next_storage_key.next();
                        key
                    }
                );

                let storage_key_clone = storage_key;
                self.storage.insert(storage_key, value);

                let stored_value = self.storage.get(&storage_key_clone).unwrap();

                $(
                    let $unique_param = stored_value;
                    let unique_key = $unique_expr;
                    self.$unique_name.insert(unique_key, storage_key_clone);
                )*

                $(
                    let $non_unique_param = stored_value;
                    let non_unique_key = $non_unique_expr;
                    self.$non_unique_name
                        .entry(non_unique_key)
                        .or_default()
                        .insert(storage_key_clone);
                )*
                $(
                    let $unique_ordered_param = stored_value;
                    let unique_ordered_key = $unique_ordered_expr;
                    self.$unique_ordered_name.insert(unique_ordered_key, storage_key_clone);
                )*
                $(
                    let $non_unique_ordered_param = stored_value;
                    let non_unique_ordered_key = $non_unique_ordered_expr;
                    self.$non_unique_ordered_name
                        .entry(non_unique_ordered_key)
                        .or_default()
                        .insert(storage_key_clone);
                )*
            }

            $(
                paste! {
                    #[doc = concat!("Get a single value, if it exist, by indexing by the unique key `", stringify!($unique_name), "` .")]
                    pub fn [<get_by_ $unique_name>](&self, $unique_name: &$unique_key_type) -> Option<&$value_type> {
                        self.$unique_name
                            .get($unique_name)
                            .and_then(|storage_key| self.storage.get(storage_key))
                    }

                    #[doc = concat!("Get a mutable modifier for a value, if it exist, by indexing by the unique key `", stringify!($unique_name), "` .")]
                    pub fn [<get_mut_by_ $unique_name>](&mut self, $unique_name: &$unique_key_type) -> Option<[<$map_name MutEntry>]> {
                        let storage_key = self.$unique_name.get($unique_name)?;

                        Some([<$map_name MutEntry>] {
                            entry: *storage_key,
                            hashmap: self
                        })
                    }
                }
            )*

            $(
                paste! {
                    #[doc = concat!("Get a single value, if it exist, by indexing by the unique ordered key `", stringify!($unique_ordered_name), "` .")]
                    pub fn [<get_by_ $unique_ordered_name>](&self, $unique_ordered_name: &$unique_ordered_key_type) -> Option<&$value_type> {
                        self.$unique_ordered_name
                            .get($unique_ordered_name)
                            .and_then(|storage_key| self.storage.get(storage_key))
                    }

                    #[doc = concat!("Get a mutable modifier for a value, if it exist, by indexing by the unique ordered key `", stringify!($unique_name), "` .")]
                    pub fn [<get_mut_by_ $unique_ordered_name>](&mut self, $unique_ordered_name: &$unique_ordered_key_type) -> Option<[<$map_name MutEntry>]> {
                        let storage_key = self.$unique_ordered_name.get($unique_ordered_name)?;
                        Some([<$map_name MutEntry>] {
                            entry: *storage_key,
                            hashmap: self
                        })
                    }

                    $vis fn [<first_by_ $unique_ordered_name>](&self) -> Option<&$value_type> {
                        self.$unique_ordered_name
                            .first_key_value()
                            .and_then(|(_, sk)| self.storage.get(sk))
                    }

                    $vis fn [<last_by_ $unique_ordered_name>](&self) -> Option<&$value_type> {
                        self.$unique_ordered_name
                            .last_key_value()
                            .and_then(|(_, sk)| self.storage.get(sk))
                    }

                    $vis fn [<get_by_ $unique_ordered_name _range>]<R>(&self, range: R) -> impl Iterator<Item = &$value_type>
                    where
                        R: std::ops::RangeBounds<$unique_ordered_key_type>,
                    {
                        self.$unique_ordered_name
                            .range(range)
                            .filter_map(|(_, sk)| self.storage.get(sk))
                    }
                }
            )*

            $(
                paste! {
                    #[doc = concat!("Get the values by indexing by the non unique key `", stringify!($non_unique_name), "` .")]
                    pub fn [<get_by_ $non_unique_name>]<'a>(&'a self, $non_unique_name: &$non_unique_key_type) -> impl Iterator<Item = &'a $value_type> {
                        self.$non_unique_name
                            .get($non_unique_name)
                            .into_iter()
                            .flat_map(|storage_keys| {
                                storage_keys
                                    .iter()
                                    .filter_map(|sk| self.storage.get(sk))
                            })
                    }

                    #[doc = concat!("Get a mutable modifier for a value, if it exist, by indexing by the non unique key `", stringify!($non_unique_name), "` .")]
                    pub fn [<get_mut_by_ $non_unique_name>](&mut self, $non_unique_name: &$non_unique_key_type) -> [<$map_name MutEntries>] {
                        let storage_keys = match self.$non_unique_name.get($non_unique_name) {
                            Some(s) => s,
                            None => return [<$map_name MutEntries>] {
                                entries: vec![].into_iter(),
                                hashmap: self,
                            },
                        };

                        [<$map_name MutEntries>] {
                            entries: storage_keys.clone().into_iter().collect::<Vec<_>>().into_iter(),
                            hashmap: self,
                        }
                    }
                }
            )*

            $(
                paste! {
                    #[doc = concat!("Get the values by indexing by the non unique ordered key `", stringify!($non_unique_ordered_name), "` .")]
                    pub fn [<get_by_ $non_unique_ordered_name>]<'a>(&'a self, $non_unique_ordered_name: &$non_unique_ordered_key_type) -> impl Iterator<Item = &'a $value_type> {
                        self.$non_unique_ordered_name
                            .get($non_unique_ordered_name)
                            .into_iter()
                            .flat_map(|storage_keys| {
                                storage_keys
                                    .iter()
                                    .filter_map(|sk| self.storage.get(sk))
                            })
                    }

                    #[doc = concat!("Get a mutable modifier for a value, if it exist, by indexing by the non unique ordered key `", stringify!($non_unique_ordered_name), "` .")]
                    pub fn [<get_mut_by_ $non_unique_ordered_name>](&mut self, $non_unique_ordered_name: &$non_unique_ordered_key_type) -> [<$map_name MutEntries>] {
                        let storage_keys = match self.$non_unique_ordered_name.get($non_unique_ordered_name) {
                            Some(s) => s,
                            None => return [<$map_name MutEntries>] {
                                entries: vec![].into_iter(),
                                hashmap: self,
                            },
                        };

                        [<$map_name MutEntries>] {
                            entries: storage_keys.clone().into_iter().collect::<Vec<_>>().into_iter(),
                            hashmap: self,
                        }
                    }

                    $vis fn [<first_by_ $non_unique_ordered_name>](&self) -> impl Iterator<Item = &$value_type> {
                        self.$non_unique_ordered_name
                            .first_key_value()
                            .into_iter()
                            .flat_map(|(_, storage_keys)| {
                                storage_keys
                                    .iter()
                                    .filter_map(|sk| self.storage.get(sk))
                            })
                    }

                    $vis fn [<last_by_ $non_unique_ordered_name>](&self) -> impl Iterator<Item = &$value_type> {
                        self.$non_unique_ordered_name
                            .last_key_value()
                            .into_iter()
                            .flat_map(|(_, storage_keys)| {
                                storage_keys
                                    .iter()
                                    .filter_map(|sk| self.storage.get(sk))
                            })
                    }

                    $vis fn [<get_by_ $non_unique_ordered_name _range>]<R>(&self, range: R) -> impl Iterator<Item = &$value_type>
                    where
                        R: std::ops::RangeBounds<$non_unique_ordered_key_type>,
                    {
                        self.$non_unique_ordered_name
                            .range(range)
                            .flat_map(|(_, storage_keys)| {
                                storage_keys
                                    .iter()
                                    .filter_map(|sk| self.storage.get(sk))
                            })
                    }
                }
            )*

            $crate::__multimap_combined_getters!(@generate_combos
                $vis $map_name<$value_type>,
                all_fields: [
                    $(($non_unique_name: $non_unique_key_type))*
                    $(($non_unique_ordered_name: $non_unique_ordered_key_type))*
                ],
                selected: [],
                remaining: [
                    $($non_unique_name: $non_unique_key_type,)*
                    $($non_unique_ordered_name: $non_unique_ordered_key_type,)*
                ]
            );

            $crate::__multimap_combined_ordered_getters!(@generate_combos
                $vis $map_name<$value_type>,
                eq_fields: [
                    $($non_unique_name: $non_unique_key_type,)*
                    $($non_unique_ordered_name: $non_unique_ordered_key_type,)*
                ],
                ordered_fields: [
                    $($non_unique_ordered_name: $non_unique_ordered_key_type,)*
                ]
            );

            #[doc = concat!("Remove entry from store by unique key.")]
            pub fn remove(&mut self, storage_key: &paste! { [<$map_name StorageIndex>] }) -> Option<$value_type> {
                let value = self.storage.remove(storage_key)?;

                // Remove from unique indexes
                $(
                    let $unique_param = &value;
                    let unique_key = $unique_expr;
                    self.$unique_name.remove(&unique_key);
                )*

                // Remove from non-unique indexes
                $(
                    let $non_unique_param = &value;
                    let non_unique_key = $non_unique_expr;
                    if let Some(keys) = self.$non_unique_name.get_mut(&non_unique_key) {
                        keys.retain(|k| k != storage_key);
                        if keys.is_empty() {
                            self.$non_unique_name.remove(&non_unique_key);
                        }
                    }
                )*

                // Remove from unique ordered indexes
                $(
                    let $unique_ordered_param = &value;
                    let unique_ordered_key = $unique_ordered_expr;
                    self.$unique_ordered_name.remove(&unique_ordered_key);
                )*
                // Remove from non-unique ordered indexes
                $(
                    let $non_unique_ordered_param = &value;
                    let non_unique_ordered_key = $non_unique_ordered_expr;
                    if let Some(keys) = self.$non_unique_ordered_name.get_mut(&non_unique_ordered_key) {
                        keys.remove(storage_key);
                        if keys.is_empty() {
                            self.$non_unique_ordered_name.remove(&non_unique_ordered_key);
                        }
                    }
                )*

                self.freed_storage_keys.push(*storage_key);

                Some(value)
            }
            
            #[doc = "Get the number of values in the store"]
            #[inline]
            pub fn len(&self) -> usize {
                self.storage.len()
            }
            
            #[doc = "Check if the map is empty"]
            #[inline]
             pub fn is_empty(&self) -> bool {
                 self.storage.is_empty()
            }
            
             #[doc = "Extend the map with multiple values, returning a vector of all the unique contraint errors."]
             pub fn extend<I>(&mut self, iter: I) -> Vec<multi_index_container::UniqueContraintViolation<$value_type>>
             where
                 I: IntoIterator<Item = $value_type>,
             {
                 let mut errors = Vec::new();
                 for value in iter {
                     if let Err(e) = self.insert(value) {
                         errors.push(e);
                     }
                 }
                 errors
             }
            
             #[doc = "Iterate over all the values in the map."]
             pub fn values(&self) -> std::collections::hash_map::Values<'_, paste! { [<$map_name StorageIndex>] }, $value_type> {
                 self.storage.values()
             }
        }

        impl Default for $map_name {

            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }

        paste! {
            #[doc = concat!("An iterator likey type of mutable entry handles into [`", stringify!($map_name), "`], produced by a multi-key lookup.")]
            #[doc = ""]
            #[doc = "Holds exclusive mutable access to the map for its lifetime `'map`, ensuring"]
            #[doc = "no other mutable borrows can exist while entries are being traversed or modified."]
            #[doc = ""]
            #[doc = "Obtained from methods that look up multiple keys at once (e.g. `get_many_mut`)."]
            #[doc = concat!("Each item yielded is a [`", stringify!($map_name), "MutEntry`], which allows in-place mutation or removal of that individual entry.")]
            #[doc = ""]
            #[doc = "# Notes"]
            #[doc = "- Entries are yielded in the same order as the keys passed to the originating lookup."]
            #[doc = "- Indices that no longer exist in the map at iteration time are silently skipped"]
            #[doc = "  by consuming methods such as `remove_all` and `collect_values`."]
            #[doc = ""]
            #[doc = "# Lifetimes"]
            #[doc = concat!("- `'map` — ties this iterator to the mutable borrow of the underlying [`", stringify!($map_name), "`], preventing any other access to the map until the iterator is dropped.")]
            $vis struct [<$map_name MutEntries>]<'map> {
                entries: std::vec::IntoIter<[<$map_name StorageIndex>]>,
                hashmap: &'map mut $map_name,
            }

            impl<'map> [<$map_name MutEntries>]<'map> {
                /// Creates an iterator which uses a closure to determine if an element should be yielded.
                ///
                /// Given an element the closure must return true or false. The returned iterator will yield only the elements for which the closure returns true.
                pub fn filter<F>(self, mut predicate: F) -> Self
                where
                    F: FnMut(&$value_type) -> bool,
                {
                    let filtered: Vec<_> = self
                        .entries
                        .filter(|index| {
                            self.hashmap
                                .storage
                                .get(index)
                                .map(|v| predicate(v))
                                .unwrap_or(false)
                        })
                        .collect();

                    Self {
                        entries: filtered.into_iter(),
                        hashmap: self.hashmap,
                    }
                }
                
                /// Get a value from the yielded results.
                pub fn first(self) -> Option<[<$map_name MutEntry>]<'map>> {
                    let hashmap = self.hashmap;
                    self.entries
                        .into_iter()
                        .next()
                        .map(|entry| [<$map_name MutEntry>] { entry, hashmap })
                }
                
                /// Calls a closure on each element of an iterator.
                pub fn for_each<F>(self, mut f: F)
                where
                    F: for<'entry> FnMut([<$map_name MutEntry>]<'entry>),
                {
                    let hashmap = self.hashmap;
                    for entry in self.entries {
                        f([<$map_name MutEntry>] { entry, hashmap });
                    }
                }
                
                /// Searches for an element of an iterator that satisfies a predicate.
                pub fn find<F>(self, mut predicate: F) -> Option<[<$map_name MutEntry>]<'map>>
                where
                    F: FnMut(&$value_type) -> bool,
                {
                    let hashmap = self.hashmap;
                    let found = self.entries
                        .into_iter()
                        .find(|index| {
                            hashmap
                                .storage
                                .get(index)
                                .map(|v| predicate(v))
                                .unwrap_or(false)
                        });

                    found.map(|entry| [<$map_name MutEntry>] { entry, hashmap })
                }
                
                /// Removes all entries in this multi-entry selection from the map, returning their values.
                ///
                /// # Returns
                /// A `Vec` containing the removed values, in iteration order.
                pub fn remove_all(self) -> Vec<$value_type> {
                    let hashmap = self.hashmap;
                    self.entries
                        .into_iter()
                        .filter_map(|entry| {
                            hashmap
                                .storage
                                .get(&entry)
                                .is_some()
                                .then(|| [<$map_name MutEntry>] { entry, hashmap }.remove())
                        })
                        .collect()
                }

                /// Returns the last entry in this selection as a mutable entry handle, if any.
                ///
                /// # Returns
                /// `Some(MutEntry)` for the last entry, or `None` if the selection is empty.
                pub fn last(self) -> Option<[<$map_name MutEntry>]<'map>> {
                    let hashmap = self.hashmap;
                    self.entries
                        .into_iter()
                        .last()
                        .map(|entry| [<$map_name MutEntry>] { entry, hashmap })
                }

                /// Returns the entry at position `n` in this selection as a mutable entry handle, if any.
                ///
                /// # Parameters
                /// - `n`: Zero-based index into the selection.
                ///
                /// # Returns
                /// `Some(MutEntry)` for the nth entry, or `None` if `n` is out of bounds.
                pub fn nth(self, n: usize) -> Option<[<$map_name MutEntry>]<'map>> {
                    let hashmap = self.hashmap;
                    self.entries
                        .into_iter()
                        .nth(n)
                        .map(|entry| [<$map_name MutEntry>] { entry, hashmap })
                }

                /// Returns the number of entries in this selection.
                #[inline]
                pub fn count(self) -> usize {
                    self.entries.len()
                }

                /// Returns `true` if this selection contains no entries.
                #[inline]
                pub fn is_empty(self) -> bool {
                    self.entries.len() == 0
                }

                /// Returns `true` if this selection contains at least one entry.
                #[inline]
                pub fn is_not_empty(self) -> bool {
                    self.entries.len() != 0
                }
                /// Collects shared references to the values of all entries in this selection.
                ///
                /// Entries that no longer exist in the map are silently skipped.
                ///
                /// # Returns
                /// A `Vec` of references to the values, in iteration order.
                pub fn collect_values(self) -> Vec<&'map $value_type> {
                    let hashmap = self.hashmap;
                    self.entries
                        .into_iter()
                        .filter_map(|index| hashmap.storage.get(&index))
                        .collect()
                }
            }
        }

        paste! {
            $vis struct [<$map_name MutEntry>] <'map> {
                entry: [<$map_name StorageIndex>],
                hashmap: &'map mut $map_name,
            }

            impl<'map> [<$map_name MutEntry>]<'map> {

                /// Remove the value from the hashmap
                pub fn remove(&mut self) -> $value_type {
                    self.hashmap.remove(&self.entry).expect("Expected the value to exist given it is guaranteed by the mutable pointer to the hashmap while this reference is initialised.")
                }

                pub fn get(&self) -> &$value_type {
                    self.hashmap.get(&self.entry).expect("Expected the value to exist given it is guaranteed by the mutable pointer to the hashmap while this reference is initialised.")
                }

                #[doc = concat!("Modify a `", stringify!($value_type), "` such that indexes are kept up to date.")]
                #[doc = "If a modified value would fail to be inserted, the original value remains in place while the new value gets returned as part of the error. This means it is cloned as part of this function. To avoid this clone you can use the modify_or_remove function"]
                pub fn modify<F>(&mut self, f: F) -> Result<(), multi_index_container::UniqueContraintViolation<$value_type>>
                where
                    F: for<'entry> FnOnce(&'entry mut $value_type),
                {
                    let value = self.remove();
                    let mut modifiable_value = value.clone();
                    f(&mut modifiable_value);

                    if let Err(error) = self.hashmap.insert(modifiable_value) {
                        self.hashmap.insert(value).expect("Expected to be able to insert the value we just removed.");

                        return Err(error)
                    }

                    Ok(())
                }

                #[doc = concat!("Modify a `", stringify!($value_type), "` such that indexes are kept up to date.")]
                #[doc = "If a modified value would fail to be inserted, the original value is lost."]
                pub fn modify_or_remove<F>(&mut self, f: F) -> Result<(), multi_index_container::UniqueContraintViolation<$value_type>>
                where
                    F: for<'entry> FnOnce(&'entry mut $value_type),
                {
                    let mut modifiable_value = self.remove();
                    f(&mut modifiable_value);

                    self.hashmap.insert(modifiable_value)
                }
            }
        }
    };
}
