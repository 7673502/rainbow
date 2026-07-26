use std::cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub enum Combo {
    Single { card: u8 },
    Set { card: u8, count: u8 },
    Run { start: u8, end: u8 },
}

impl PartialEq for Combo {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Combo::Single { card: x }, Combo::Single { card: y }) => x == y,
            (
                Combo::Set {
                    card: x,
                    count: x_count,
                },
                Combo::Set {
                    card: y,
                    count: y_count,
                },
            ) => x == y && x_count == y_count,
            (
                Combo::Run {
                    start: x_start,
                    end: x_end,
                },
                Combo::Run {
                    start: y_start,
                    end: y_end,
                },
            ) => x_start == y_start && x_end == y_end,
            _ => false,
        }
    }
}

impl PartialOrd for Combo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Combo::Single { .. }, Combo::Set { .. }) => Some(Ordering::Less),
            (Combo::Single { .. }, Combo::Run { .. }) => Some(Ordering::Less),
            (Combo::Set { .. }, Combo::Single { .. }) => Some(Ordering::Greater),
            (Combo::Run { .. }, Combo::Single { .. }) => Some(Ordering::Greater),
            (Combo::Single { card: x }, Combo::Single { card: y }) => x.partial_cmp(y),
            (
                Combo::Set {
                    card: x,
                    count: x_count,
                },
                Combo::Set {
                    card: y,
                    count: y_count,
                },
            ) => Some(x_count.cmp(y_count).then(x.cmp(y))),
            (
                Combo::Run {
                    start: x_start,
                    end: x_end,
                },
                Combo::Run {
                    start: y_start,
                    end: y_end,
                },
            ) => {
                let x_len = x_end - x_start + 1;
                let y_len = y_end - y_start + 1;
                Some(x_len.cmp(&y_len).then(x_end.cmp(y_end)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::arbitrary::Arbitrary;
    use proptest::prelude::*;

    impl Arbitrary for Combo {
        type Parameters = ();
        type Strategy = BoxedStrategy<Combo>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                (1..=6u8).prop_map(|card| Combo::Single { card }),
                (1..=6u8, 1..=10u8).prop_map(|(card, count)| Combo::Set { card, count }),
                (1..=5u8).prop_flat_map(
                    |start| (start + 1..=6u8).prop_map(move |end| Combo::Run { start, end })
                ),
            ]
            .boxed()
        }
    }

    proptest! {
        #[test]
        fn test_partial_ord_symmetry(a in any::<Combo>(), b in any::<Combo>()) {
            match a.partial_cmp(&b) {
                Some(Ordering::Greater) => prop_assert!(b < a),
                Some(Ordering::Less) => prop_assert!(b > a),
                Some(Ordering::Equal) => prop_assert!(b == a),
                None => prop_assert!(b.partial_cmp(&a).is_none()),
            }
        }

        #[test]
        fn test_partial_ord_transitivity(
            a in any::<Combo>(),
            b in any::<Combo>(),
            c in any::<Combo>()
        ) {
            if let (Some(ab), Some(bc)) = (a.partial_cmp(&b), b.partial_cmp(&c)) {
                match (ab, bc) {
                    (Ordering::Less, Ordering::Less)
                    | (Ordering::Less, Ordering::Equal)
                    | (Ordering::Equal, Ordering::Less) => {
                        prop_assert_eq!(a.partial_cmp(&c), Some(Ordering::Less));
                    }
                    (Ordering::Greater, Ordering::Greater)
                    | (Ordering::Greater, Ordering::Equal)
                    | (Ordering::Equal, Ordering::Greater) => {
                        prop_assert_eq!(a.partial_cmp(&c), Some(Ordering::Greater));
                    }
                    (Ordering::Equal, Ordering::Equal) => {
                        prop_assert_eq!(a.partial_cmp(&c), Some(Ordering::Equal));
                    }
                    _ => {}
                }
            }
        }

        #[test]
        fn test_single_comparisons(card1 in 1..=6u8, card2 in 1..=6u8) {
            let combo1 = Combo::Single { card: card1 };
            let combo2 = Combo::Single { card: card2 };

            if card1 < card2 {
                prop_assert!(combo1 < combo2);
            } else if card2 < card1 {
                prop_assert!(combo2 < combo1);
            } else {
                prop_assert_eq!(combo1, combo2);
            }
        }

        #[test]
        fn test_set_comparisons(
            card1 in 1..=6u8, count1 in 1..=10u8,
            card2 in 1..=6u8, count2 in 1..=10u8,
        ) {
            let combo1 = Combo::Set { card: card1, count: count1 };
            let combo2 = Combo::Set { card: card2, count: count2 };

            if count1 < count2 {
                prop_assert!(combo1 < combo2);
            } else if count2 < count1 {
                prop_assert!(combo2 < combo1);
            } else if card1 < card2 {
                prop_assert!(combo1 < combo2);
            } else if card2 < card1 {
                prop_assert!(combo2 < combo1);
            } else {
                prop_assert_eq!(combo1, combo2);
            }
        }

        #[test]
        fn test_run_comparisons(
            start1 in 1..=5u8, end1 in 2..=6u8,
            start2 in 1..=5u8, end2 in 2..=6u8,
        ) {
            prop_assume!(start1 < end1);
            prop_assume!(start2 < end2);

            let combo1 = Combo::Run { start: start1, end: end1 };
            let combo2 = Combo::Run { start: start2, end: end2 };

            let combo1_len = end1 - start1 + 1;
            let combo2_len = end2 - start2 + 1;

            if combo1_len < combo2_len {
                prop_assert!(combo1 < combo2);
            } else if combo2_len < combo1_len {
                prop_assert!(combo2 < combo1);
            } else if end1 < end2 {
                prop_assert!(combo1 < combo2);
            } else if end2 < end1 {
                prop_assert!(combo2 < combo1);
            } else {
                prop_assert_eq!(combo1, combo2);
            }
        }

        #[test]
        fn test_single_vs_set_comparisons(
            set_card in 1..=6u8, set_count in 1..=10u8,
            single_card in 1..=6u8,
        ) {
            let single = Combo::Single { card: single_card };
            let set = Combo::Set { card: set_card, count: set_count };

            prop_assert!(single < set);
            prop_assert!(set > single);
        }

        #[test]
        fn test_single_vs_run_comparisons(
            run_start in 1..=5u8, run_end in 2..=6u8,
            single_card in 1..=6u8,
        ) {
            prop_assume!(run_start < run_end);
            let single = Combo::Single { card: single_card };
            let run = Combo::Run { start: run_start, end: run_end };

            prop_assert!(single < run);
            prop_assert!(run > single);
        }

        #[test]
        fn test_set_vs_run_comparisons(
            set_card in 1..=6u8, set_count in 1..=10u8,
            run_start in 1..=5u8, run_end in 2..=6u8,
        ) {
            prop_assume!(run_start < run_end);
            let set = Combo::Set { card: set_card, count: set_count };
            let run = Combo::Run { start: run_start, end: run_end };

            prop_assert_eq!(set.partial_cmp(&run), None);
        }
    }
}
