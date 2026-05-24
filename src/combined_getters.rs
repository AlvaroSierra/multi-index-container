// Internal macro to compute intersections for combined getters
#[macro_export]
macro_rules! __multimap_combined_getters {
    // Base case: emit all accumulated combinations (2+ fields)
    // Entry point: called with all non-unique fields collected
    (@generate_combos
        $vis:vis $map_name:ident<$value_type:ty>,
        // all non-unique fields as (name, key_type, expr_param, expr) tuples
        all_fields: [$(($fname:ident: $fkey:ty))*],
        // currently selected combo (grows recursively)
        selected: [$($sel_name:ident: $sel_key:ty),*],
        // remaining fields to consider
        remaining: [$next_name:ident: $next_key:ty, $($rest_name:ident: $rest_key:ty,)*]
    ) => {
        // Branch 1: skip $next_name
        $crate::__multimap_combined_getters!(@generate_combos
            $vis $map_name<$value_type>,
            all_fields: [$(($fname: $fkey))*],
            selected: [$($sel_name: $sel_key),*],
            remaining: [$($rest_name: $rest_key,)*]
        );
        // Branch 2: include $next_name
        $crate::__multimap_combined_getters!(@generate_combos
            $vis $map_name<$value_type>,
            all_fields: [$(($fname: $fkey))*],
            selected: [$($sel_name: $sel_key,)* $next_name: $next_key],
            remaining: [$($rest_name: $rest_key,)*]
        );
    };
    // Recursive base: no more remaining, emit getter if selected has 2+ fields
    (@generate_combos
        $vis:vis $map_name:ident<$value_type:ty>,
        all_fields: [$($_f:tt)*],
        selected: [$first_name:ident: $first_key:ty, $second_name:ident: $second_key:ty $(,$more_name:ident: $more_key:ty)*],
        remaining: []
    ) => {
        paste! {
            $vis fn [<get_by_ $first_name _ $second_name $(  _ $more_name)*>](
                &self,
                $first_name: &$first_key,
                $second_name: &$second_key,
                $($more_name: &$more_key,)*
            ) -> Vec<&$value_type> {
                // Start with first index set
                let first_set = match self.$first_name.get($first_name) {
                    Some(s) => s,
                    None => return vec![],
                };
                let second_set = match self.$second_name.get($second_name) {
                    Some(s) => s,
                    None => return vec![],
                };
                // Intersect first two
                let mut intersection: std::collections::HashSet<_> =
                    first_set.intersection(second_set).copied().collect();
                // Intersect with remaining
                $(
                    let next_set = match self.$more_name.get($more_name) {
                        Some(s) => s,
                        None => return vec![],
                    };
                    intersection = intersection.intersection(next_set).copied().collect();
                )*
                // Resolve storage keys to values
                intersection.iter()
                    .filter_map(|key| self.storage.get(key))
                    .collect()
            }
            $vis fn [<get_mut_by_ $first_name _ $second_name $(  _ $more_name)*>](
                    &mut self,
                    $first_name: &$first_key,
                    $second_name: &$second_key,
                    $($more_name: &$more_key,)*
                ) -> [<$map_name MutEntries>]<'_> {
                    let first_set = match self.$first_name.get($first_name) {
                        Some(s) => s,
                        None => return [<$map_name MutEntries>] {
                            entries: vec![].into_iter(),
                            hashmap: self,
                        },
                    };
                    let second_set = match self.$second_name.get($second_name) {
                        Some(s) => s,
                        None => return [<$map_name MutEntries>] {
                            entries: vec![].into_iter(),
                            hashmap: self,
                        },
                    };
                    // We must clone into an owned set because we need &mut self below
                    let mut intersection: std::collections::HashSet<_> =
                        first_set.intersection(second_set).copied().collect();
                    $(
                        let next_set = match self.$more_name.get($more_name) {
                            Some(s) => s,
                            None => return [<$map_name MutEntries>] {
                                entries: vec![].into_iter(),
                                hashmap: self,
                            },
                        };
                        intersection = intersection.intersection(next_set).copied().collect();
                    )*
                    let keys: Vec<_> = intersection.into_iter().collect();
                    [<$map_name MutEntries>] {
                        entries: keys.into_iter(),
                        hashmap: self,
                    }
            }
        }
    };
    // Skip: selected has fewer than 2 fields and no remaining — emit nothing
    (@generate_combos
        $vis:vis $map_name:ident<$value_type:ty>,
        all_fields: [$($_f:tt)*],
        selected: [$($_s:tt)*],
        remaining: []
    ) => {};
}
