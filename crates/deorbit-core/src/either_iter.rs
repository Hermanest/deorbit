pub enum EitherIter<L, R> {
    Left(L),
    Right(R),
}

impl<L, R, T> Iterator for EitherIter<L, R>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::Left(inner) => inner.next(),
            EitherIter::Right(inner) => inner.next(),
        }
    }
}

impl<L, R, T> DoubleEndedIterator for EitherIter<L, R>
where
    L: DoubleEndedIterator<Item = T>,
    R: DoubleEndedIterator<Item = T>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::Left(inner) => inner.next_back(),
            EitherIter::Right(inner) => inner.next_back(),
        }
    }
}