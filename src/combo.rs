use std::cmp::Ordering;

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
            (Combo::Single { card: x }, Combo::Single { card: y }) => x.partial_cmp(&y),
            (
                Combo::Set {
                    card: x,
                    count: x_count,
                },
                Combo::Set {
                    card: y,
                    count: y_count,
                },
            ) => Some(x_count.cmp(&y_count).then(x.cmp(&y))),
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
                Some(x_len.cmp(&y_len).then(x_end.cmp(&y_end)))
            }
            _ => None,
        }
    }
}
