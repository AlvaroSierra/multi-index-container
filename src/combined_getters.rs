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

            #[doc = "Get a reference to a value, if it exist, by indexing by multiple keys."]
            $vis fn [<get_by_ $first_name _ $second_name $(  _ $more_name)*>]<'a>(
                &'a self,
                $first_name: &$first_key,
                $second_name: &$second_key,
                $($more_name: &$more_key,)*
            ) -> impl Iterator<Item = &'a $value_type> {
                let result: Option<std::collections::HashSet<_>> = (|| {
                    let first_set = self.$first_name.get($first_name)?;
                    let second_set = self.$second_name.get($second_name)?;
                    let mut intersection: std::collections::HashSet<_> =
                        first_set.intersection(second_set).copied().collect();
                    $(
                        let next_set = self.$more_name.get($more_name)?;
                        intersection = intersection.intersection(next_set).copied().collect();
                    )*
                    Some(intersection)
                })();

                result
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .filter_map(|key| self.storage.get(&key))
            }


            #[doc = "Get a mutable modifier for a value, if it exist, by indexing by multiple keys."]
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

#[macro_export]
macro_rules! __multimap_combined_ordered_getters {
    // Entry point — split fields into the range field and equality filters
    (@generate_combos
        $vis:vis $map_name:ident<$value_type:ty>,
        // All non-unique fields (ordered and unordered) for equality filtering
        eq_fields: [$($eq_name:ident: $eq_key:ty,)*],
        // All non-unique ordered fields to range over
        ordered_fields: [$next_ordered:ident: $next_ordered_key:ty, $($rest_ordered:ident: $rest_ordered_key:ty,)*]
    ) => {
        // Generate combinations using $next_ordered as the range field,
        // with every subset of eq_fields as equality filters
        $crate::__multimap_combined_ordered_getters!(@generate_eq_combos
            $vis $map_name<$value_type>,
            range_field: $next_ordered: $next_ordered_key,
            selected_eq: [],
            remaining_eq: [$($eq_name: $eq_key,)*]
        );
        // Recurse for remaining ordered fields
        $crate::__multimap_combined_ordered_getters!(@generate_combos
            $vis $map_name<$value_type>,
            eq_fields: [$($eq_name: $eq_key,)*],
            ordered_fields: [$($rest_ordered: $rest_ordered_key,)*]
        );
    };
    // Base: no more ordered fields to range over
    (@generate_combos
        $vis:vis $map_name:ident<$value_type:ty>,
        eq_fields: [$($eq_name:ident: $eq_key:ty,)*],
        ordered_fields: []
    ) => {};

    // Expand every subset of eq_fields to pair with the range field
    (@generate_eq_combos
        $vis:vis $map_name:ident<$value_type:ty>,
        range_field: $range_name:ident: $range_key:ty,
        selected_eq: [$($sel_name:ident: $sel_key:ty),*],
        remaining_eq: [$next_eq:ident: $next_eq_key:ty, $($rest_eq:ident: $rest_eq_key:ty,)*]
    ) => {
        // Branch 1: skip $next_eq
        $crate::__multimap_combined_ordered_getters!(@generate_eq_combos
            $vis $map_name<$value_type>,
            range_field: $range_name: $range_key,
            selected_eq: [$($sel_name: $sel_key),*],
            remaining_eq: [$($rest_eq: $rest_eq_key,)*]
        );
        // Branch 2: include $next_eq
        $crate::__multimap_combined_ordered_getters!(@generate_eq_combos
            $vis $map_name<$value_type>,
            range_field: $range_name: $range_key,
            selected_eq: [$($sel_name: $sel_key,)* $next_eq: $next_eq_key],
            remaining_eq: [$($rest_eq: $rest_eq_key,)*]
        );
    };

    // Base: no remaining eq fields — emit getter if at least 1 eq field selected
    (@generate_eq_combos
        $vis:vis $map_name:ident<$value_type:ty>,
        range_field: $range_name:ident: $range_key:ty,
        selected_eq: [$first_eq:ident: $first_eq_key:ty $(,$more_eq:ident: $more_eq_key:ty)*],
        remaining_eq: []
    ) => {
        paste! {
            #[doc = "Get references to values within a key range on an ordered index, \
                     filtered by equality on additional indexes."]
            $vis fn [<get_by_ $range_name _range_ $first_eq $(  _ $more_eq)*>]<'a, R>(
                &'a self,
                [<$range_name range>]: R,
                $first_eq: &$first_eq_key,
                $($more_eq: &$more_eq_key,)*
            ) -> Box<dyn Iterator<Item = &'a $value_type> + 'a>
            where
                R: std::ops::RangeBounds<$range_key>,
            {
                let eq_intersection: Option<std::collections::HashSet<_>> = (|| {
                    let mut intersection: std::collections::HashSet<_> =
                        self.$first_eq.get($first_eq)?.iter().copied().collect();
                    $(
                        let next_set: std::collections::HashSet<_> =
                            self.$more_eq.get($more_eq)?.iter().copied().collect();
                        intersection = intersection.intersection(&next_set).copied().collect();
                    )*
                    Some(intersection)
                })();

                let eq_intersection = match eq_intersection {
                    Some(s) => s,
                    None => return Box::new(std::iter::empty()),
                };

                let iter = self
                    .$range_name
                    .range([<$range_name range>])
                    .flat_map(|(_, storage_keys)| storage_keys.iter().copied())
                    .filter(move |sk| eq_intersection.contains(sk))
                    .filter_map(|sk| self.storage.get(&sk));

                Box::new(iter)
            }

            #[doc = "Get mutable entries within a key range on an ordered index, \
                     filtered by equality on additional indexes."]
            $vis fn [<get_mut_by_ $range_name _range_ $first_eq $(  _ $more_eq)*>]<R>(
                &mut self,
                [<$range_name range>]: R,
                $first_eq: &$first_eq_key,
                $($more_eq: &$more_eq_key,)*
            ) -> [<$map_name MutEntries>]<'_>
            where
                R: std::ops::RangeBounds<$range_key>,
            {
                let eq_intersection: Option<std::collections::HashSet<_>> = (|| {
                    let mut intersection: std::collections::HashSet<_> =
                        self.$first_eq.get($first_eq)?.iter().copied().collect();
                    $(
                        let next_set: std::collections::HashSet<_> =
                            self.$more_eq.get($more_eq)?.iter().copied().collect();
                        intersection = intersection.intersection(&next_set).copied().collect();
                    )*
                    Some(intersection)
                })();

                let eq_intersection = match eq_intersection {
                    Some(s) => s,
                    None => return [<$map_name MutEntries>] {
                        entries: vec![].into_iter(),
                        hashmap: self,
                    },
                };

                let keys: Vec<_> = self
                    .$range_name
                    .range([<$range_name range>])
                    .flat_map(|(_, storage_keys)| storage_keys.iter().copied())
                    .filter(|sk| eq_intersection.contains(sk))
                    .collect();

                [<$map_name MutEntries>] {
                    entries: keys.into_iter(),
                    hashmap: self,
                }
            }
            #[doc = "Get the first value by ordered index within a set of equality filters."]
            $vis fn [<first_by_ $range_name _ $first_eq $(  _ $more_eq)*>](
                &self,
                $first_eq: &$first_eq_key,
                $($more_eq: &$more_eq_key,)*
            ) -> Option<&$value_type> {
                let eq_intersection: std::collections::HashSet<_> = (|| {
                    let mut intersection: std::collections::HashSet<_> =
                        self.$first_eq.get($first_eq)?.iter().copied().collect();
                    $(
                        let next_set: std::collections::HashSet<_> =
                            self.$more_eq.get($more_eq)?.iter().copied().collect();
                        intersection = intersection.intersection(&next_set).copied().collect();
                    )*
                    Some(intersection)
                })()?;

                self.$range_name
                    .iter()
                    .flat_map(|(_, storage_keys)| storage_keys.iter().copied())
                    .find(|sk| eq_intersection.contains(sk))
                    .and_then(|sk| self.storage.get(&sk))
            }

            #[doc = "Get the last value by ordered index within a set of equality filters."]
            $vis fn [<last_by_ $range_name _ $first_eq $(  _ $more_eq)*>](
                &self,
                $first_eq: &$first_eq_key,
                $($more_eq: &$more_eq_key,)*
            ) -> Option<&$value_type> {
                let eq_intersection: std::collections::HashSet<_> = (|| {
                    let mut intersection: std::collections::HashSet<_> =
                        self.$first_eq.get($first_eq)?.iter().copied().collect();
                    $(
                        let next_set: std::collections::HashSet<_> =
                            self.$more_eq.get($more_eq)?.iter().copied().collect();
                        intersection = intersection.intersection(&next_set).copied().collect();
                    )*
                    Some(intersection)
                })()?;

                self.$range_name
                    .iter()
                    .rev()
                    .flat_map(|(_, storage_keys)| storage_keys.iter().copied())
                    .find(|sk| eq_intersection.contains(sk))
                    .and_then(|sk| self.storage.get(&sk))
            }

        }
    };

    // Base: no eq fields selected — emit nothing (range-only is handled by single-index getters)
    (@generate_eq_combos
        $vis:vis $map_name:ident<$value_type:ty>,
        range_field: $range_name:ident: $range_key:ty,
        selected_eq: [],
        remaining_eq: []
    ) => {};
}
