use std::slice;

#[derive(Debug)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> FromIterator<T> for OneOrMany<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();

        let first = iter.next().expect("OneOrMany requires at least one element");

        match iter.next() {
            None => OneOrMany::One(first),

            Some(second) => {
                let mut v = vec![first, second];
                v.extend(iter);
                OneOrMany::Many(v)
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrMany::One(x) => slice::from_ref(x).iter(),
            OneOrMany::Many(x) => x.into_iter()
        }
    }
}

impl<T> OneOrMany<T> {
    pub fn from_vec(mut vec: Vec<T>) -> Option<Self> {
        match vec.len() {
            x if x > 1 => Some(Self::Many(vec)),
            1 => Some(Self::One(vec.remove(0))),
            _ => None,
        }
    }

    pub fn from_val(val: T) -> Self {
        OneOrMany::One(val)
    }

    pub fn to_first_or_one(self) -> T {
        match self {
            OneOrMany::One(x) => x,
            OneOrMany::Many(mut x) => x.remove(0),
        }
    }

    pub fn to_last_or_one(self) -> T {
        match self {
            OneOrMany::One(x) => x,
            OneOrMany::Many(mut x) => x.remove(x.len() - 1),
        }
    }
}
