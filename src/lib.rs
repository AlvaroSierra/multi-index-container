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
macro_rules! multi_index_map {
    (
        $(#[$meta:meta])*
        $vis:vis $map_name:ident<$value_type:ty> {
            $(unique $unique_name:ident: $unique_key_type:ty => |$unique_param:ident| $unique_expr:expr,)*
            $(non_unique $non_unique_name:ident: $non_unique_key_type:ty => |$non_unique_param:ident| $non_unique_expr:expr,)*
        }
    ) => {
        use multi_index_hashmap::__private::paste;

        paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            $vis struct [<$map_name StorageIndex>] (usize);

            impl [<$map_name StorageIndex>] {
                pub fn next(&self) -> Self {
                    Self ( self.0 + 1 )
                }
            }
        }

        $(#[$meta])*
        #[doc = ""]
        #[doc = concat!("A map for storing `", stringify!($value_type), "` indexed by `", stringify!($storage_key_type), "`.")]
        $(
            #[doc = concat!("- Unique index `", stringify!($unique_name), "`: `", stringify!($unique_key_type), "`")]
        )*
        $(
            #[doc = concat!("- Non-unique index `", stringify!($non_unique_name), "`: `", stringify!($non_unique_key_type), "`")]
        )*
        $vis struct $map_name {
            freed_storage_keys: Vec< paste! { [<$map_name StorageIndex>] }>,
            next_storage_key: paste! { [<$map_name StorageIndex>] },
            storage: std::collections::HashMap<paste! { [<$map_name StorageIndex>] }, $value_type>,
            $(
                $unique_name: std::collections::HashMap<$unique_key_type, paste! { [<$map_name StorageIndex>] }>,
            )*
            $(
                $non_unique_name: std::collections::HashMap<$non_unique_key_type, std::collections::HashSet< paste! { [<$map_name StorageIndex>] } >>,
            )*
        }

        impl $map_name {
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
                }
            }

            pub fn get(&self, storage_key: &paste! { [<$map_name StorageIndex>] }) -> Option<&$value_type> {
                self.storage.get(storage_key)
            }

            #[doc = concat!("Inserts a `", stringify!($value_type), "` and update all indexes with the new type.")]
            pub fn insert(&mut self, value: $value_type) -> Result<(), multi_index_hashmap::UniqueContraintViolation<$value_type>> {
                // Check unique constraints
                $(
                    let $unique_param = &value;
                    let unique_key = $unique_expr;
                    if self.$unique_name.contains_key(&unique_key) {
                        return Err(multi_index_hashmap::UniqueContraintViolation { field: stringify!($unique_name), value } );
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

                Ok(())
            }

            $(
                paste! {
                    #[doc = concat!("Get a single value, if it exist, by indexing by the unique key `", stringify!($unique_name), "` .")]
                    pub fn [<get_by_ $unique_name>](&self, key: &$unique_key_type) -> Option<&$value_type> {
                        self.$unique_name
                            .get(key)
                            .and_then(|storage_key| self.storage.get(storage_key))
                    }


                    pub fn [<get_mut_by_ $unique_name>](&mut self, key: &$unique_key_type) -> Option<[<$map_name MutEntry>]> {
                        let storage_key = self.$unique_name.get(key)?;

                        Some([<$map_name MutEntry>] {
                            entry: *storage_key,
                            hashmap: self
                        })
                    }
                }
            )*

            $(
                paste! {
                    #[doc = concat!("Get the values by indexing by the non unique key `", stringify!($non_unique_name), "` .")]
                    pub fn [<get_by_ $non_unique_name>](&self, key: &$non_unique_key_type) -> Vec<&$value_type> {
                        self.$non_unique_name
                            .get(key)
                            .map(|storage_keys| {
                                storage_keys
                                    .iter()
                                    .filter_map(|sk| self.storage.get(sk))
                                    .collect()
                            })
                            .unwrap_or_default()
                    }

                    pub fn [<get_mut_by_ $non_unique_name>](&mut self, key: &$non_unique_key_type) -> Option<[<$map_name MutEntries>]> {
                        let storage_keys = self.$non_unique_name.get(key)?;

                        Some([<$map_name MutEntries>] {
                            entries: storage_keys.clone().into_iter().collect::<Vec<_>>().into_iter(),
                            hashmap: self,
                        })
                    }
                }
            )*

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

                self.freed_storage_keys.push(*storage_key);

                Some(value)
            }

            pub fn len(&self) -> usize {
                self.storage.len()
            }

             pub fn is_empty(&self) -> bool {
                 self.storage.is_empty()
            }
 
             pub fn extend<I>(&mut self, iter: I) -> Vec<multi_index_hashmap::UniqueContraintViolation<$value_type>>
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
            $vis struct [<$map_name MutEntries>]<'map> {
                entries: std::vec::IntoIter<[<$map_name StorageIndex>]>,
                hashmap: &'map mut $map_name,
            }

            impl<'map> [<$map_name MutEntries>]<'map> {

                pub fn for_each<F>(self, mut f: F)
                where
                    F: for<'entry> FnMut([<$map_name MutEntry>]<'entry>),
                {
                    let hashmap = self.hashmap;
                    for entry in self.entries {
                        f([<$map_name MutEntry>] { entry, hashmap });
                    }
                }

            }
        }

        paste! {
            $vis struct [<$map_name MutEntry>] <'map> {
                entry: [<$map_name StorageIndex>],
                hashmap: &'map mut $map_name,
            }

            impl<'map> [<$map_name MutEntry>]<'map> {
                pub fn remove(&mut self) -> $value_type {
                    self.hashmap.remove(&self.entry).expect("Expected the value to exist given it is guaranteed by the mutable pointer to the hashmap while this reference is initialised.")
                }

                pub fn get(&self) -> &$value_type {
                    self.hashmap.get(&self.entry).expect("Expected the value to exist given it is guaranteed by the mutable pointer to the hashmap while this reference is initialised.")
                }
                
                #[doc = concat!("Modify a `", stringify!($value_type), "` such that indexes are kept up to date.")]
                #[doc = "If a modified value would fail to be inserted, the original value remains in place while the new value gets returned as part of the error. This means it is cloned as part of this function. To avoid this clone you can use the modify_or_remove function"]
                pub fn modify<F>(&mut self, f: F) -> Result<(), multi_index_hashmap::UniqueContraintViolation<$value_type>>
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
                pub fn modify_or_remove<F>(&mut self, f: F) -> Result<(), multi_index_hashmap::UniqueContraintViolation<$value_type>>
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
