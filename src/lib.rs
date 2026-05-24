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
            storage_key: $storage_key_type:ty => |$storage_param:ident| $storage_expr:expr,
            $(unique $unique_name:ident: $unique_key_type:ty => |$unique_param:ident| $unique_expr:expr,)*
            $(non_unique $non_unique_name:ident: $non_unique_key_type:ty => |$non_unique_param:ident| $non_unique_expr:expr,)*
        }
    ) => {
        use std::collections::{HashMap, HashSet};
        use multi_index_hashmap::__private::paste;

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
            storage: HashMap<$storage_key_type, $value_type>,
            $(
                $unique_name: HashMap<$unique_key_type, $storage_key_type>,
            )*
            $(
                $non_unique_name: HashMap<$non_unique_key_type, HashSet<$storage_key_type>>,
            )*
        }

        impl $map_name {
            pub fn new() -> Self {
                Self {
                    storage: HashMap::new(),
                    $(
                        $unique_name: HashMap::new(),
                    )*
                    $(
                        $non_unique_name: HashMap::new(),
                    )*
                }
            }

            #[doc = concat!("Inserts a `", stringify!($value_type), "` and update all indexes with the new type.")]
            pub fn insert(&mut self, value: $value_type) -> Result<(), multi_index_hashmap::UniqueContraintViolation<$value_type>> {
                let $storage_param = &value;
                let storage_key = $storage_expr;

                // Check unique constraints
                $(
                    let $unique_param = &value;
                    let unique_key = $unique_expr;
                    if self.$unique_name.contains_key(&unique_key) {
                        return Err(multi_index_hashmap::UniqueContraintViolation { field: stringify!($unique_name), value } );
                    }
                )*

                // Insert into storage
                let storage_key_clone = storage_key.clone();
                self.storage.insert(storage_key.clone(), value);

                // Update indexes
                let stored_value = self.storage.get(&storage_key_clone).unwrap();

                $(
                    let $unique_param = stored_value;
                    let unique_key = $unique_expr;
                    self.$unique_name.insert(unique_key, storage_key_clone.clone());
                )*

                $(
                    let $non_unique_param = stored_value;
                    let non_unique_key = $non_unique_expr;
                    self.$non_unique_name
                        .entry(non_unique_key)
                        .or_default()
                        .insert(storage_key_clone.clone());
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
                }
            )*

            #[doc = concat!("Remove entry from store by unique key.")]
            pub fn remove(&mut self, storage_key: &$storage_key_type) -> Option<$value_type> {
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
          }

        impl Default for $map_name {

            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }
    };
}
