#![feature(core_intrinsics, rustc_attrs, iter_advance_by, try_trait_v2, try_blocks)]

// Reimplentation of std::iter::Skip to hack around `n`
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Skip<I> {
    iter: I,
    pub(crate) n: usize,
}

impl<I> Skip<I> {
    pub fn new(iter: I, n: usize) -> Skip<I> {
        Skip { iter, n }
    }
}

impl<I> Iterator for Skip<I>
where
    I: Iterator,
{
    type Item = <I as Iterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if std::intrinsics::unlikely(self.n > 0) {
            self.iter.nth(std::mem::take(&mut self.n) - 1);
        }
        self.iter.next()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<I::Item> {
        // Can't just add n + self.n due to overflow.
        if self.n > 0 {
            let to_skip = self.n;
            self.n = 0;
            // nth(n) skips n+1
            self.iter.nth(to_skip - 1)?;
        }
        self.iter.nth(n)
    }

    #[inline]
    fn count(mut self) -> usize {
        if self.n > 0 {
            // nth(n) skips n+1
            if self.iter.nth(self.n - 1).is_none() {
                return 0;
            }
        }
        self.iter.count()
    }

    #[inline]
    fn last(mut self) -> Option<I::Item> {
        if self.n > 0 {
            // nth(n) skips n+1
            self.iter.nth(self.n - 1)?;
        }
        self.iter.last()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();

        let lower = lower.saturating_sub(self.n);
        let upper = match upper {
            Some(x) => Some(x.saturating_sub(self.n)),
            None => None,
        };

        (lower, upper)
    }

    #[inline]
    fn try_fold<Acc, Fold, R>(&mut self, init: Acc, fold: Fold) -> R
    where
        Self: Sized,
        Fold: FnMut(Acc, Self::Item) -> R,
        R: std::ops::Try<Output = Acc>,
    {
        let n = self.n;
        self.n = 0;
        if n > 0 {
            // nth(n) skips n+1
            if self.iter.nth(n - 1).is_none() {
                return try { init };
            }
        }
        self.iter.try_fold(init, fold)
    }

    #[inline]
    fn fold<Acc, Fold>(mut self, init: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        if self.n > 0 {
            // nth(n) skips n+1
            if self.iter.nth(self.n - 1).is_none() {
                return init;
            }
        }
        self.iter.fold(init, fold)
    }

    #[inline]
    #[rustc_inherit_overflow_checks]
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        let mut rem = n;
        let step_one = self.n.saturating_add(rem);

        match self.iter.advance_by(step_one) {
            Ok(_) => {
                rem -= step_one - self.n;
                self.n = 0;
            }
            Err(advanced) => {
                let advanced_without_skip = advanced.saturating_sub(self.n);
                self.n = self.n.saturating_sub(advanced);
                return if n == 0 { Ok(()) } else { Err(advanced_without_skip) };
            }
        }

        // step_one calculation may have saturated
        if std::intrinsics::unlikely(rem > 0) {
            return match self.iter.advance_by(rem) {
                ret @ Ok(_) => ret,
                Err(advanced) => {
                    rem -= advanced;
                    Err(n - rem)
                }
            };
        }

        Ok(())
    }
}


use std::{collections::VecDeque, fmt::Debug};

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
struct Skak<I> where
	I: Iterator,
{
	iter: I,
}

struct SkakTaken<T>
{
    items: VecDeque<T>,
}

impl<T> Debug for SkakTaken<T> where
    T: Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.items)
    }
}

impl<T> Iterator for SkakTaken<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop_front()
    }
}

impl<I> Skak<I> where
	I: Iterator + Clone,
{
	pub fn new(iter: I, index: usize) -> (SkakTaken<I::Item>, Skak<Skip<I>>) {
        let taken = iter.clone().take(index).collect::<VecDeque<I::Item>>();
        (
            SkakTaken {
                items: taken, 
            },
            Skak {
                iter: Skip::new(iter, index)
            }
        )
	}

    pub fn skip(mut iter: Skak<Skip<I>>, index: usize) -> (SkakTaken<I::Item>, Skak<Skip<I>>) {
        let taken = iter.clone().take(index).collect::<VecDeque<I::Item>>();
        iter.iter.n += index;
        (
            SkakTaken {
                items: taken, 
            },
            Skak {
                iter: iter.iter
            }
        )
    }
}
	
impl<I> Iterator for Skak<I> where
    I: Iterator
{
    type Item = I::Item;

	fn next(&mut self) -> Option<I::Item> {
        self.iter.next()
	}

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let v: Vec<i32> = vec![1,2,3,4,5,6,7,8];
        let (mut taken, mut next) = Skak::new(v.iter(), 2);
        let mut count = 0;
        assert_eq!(next.size_hint().0, v.len() - 2);
        while next.size_hint().0 > 0 {
            println!("Set {}", count);
            assert_eq!(taken.next(), Some(&(count * 2 + 1)));
            assert_eq!(taken.next(), Some(&(count * 2 + 2)));
            assert_eq!(taken.next(), None);
            (taken, next) = Skak::skip(next, 2);
            count += 1;
        }
    }
}
