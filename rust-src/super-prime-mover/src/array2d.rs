#[derive(Debug, Clone)]
pub struct Array2D<T> {
    inner: Box<[T]>,
    width: usize
}

impl<T> Array2D<T> {
    pub fn new(width: usize, height: usize) -> Array2D<T>
    where
        T: Default
    {
        let mut vec = Vec::with_capacity(height * width);
        vec.resize_with(height * width, Default::default);
        Array2D {
            inner: vec.into_boxed_slice(),
            width: width,
        }
    }
    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x >= self.width { return None }
        self.inner.get(y.checked_mul(self.width)?.checked_add(x)?)
    }
    #[inline(always)]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x >= self.width { return None }
        self.inner.get_mut(y.checked_mul(self.width)?.checked_add(x)?)
    }

    pub fn get_mut2(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) -> (Option<&mut T>, Option<&mut T>) {
        let slice = &mut self.inner[..];

        let pos1 = y1.checked_mul(self.width).and_then(|v| v.checked_add(x1));
        let pos2 = y2.checked_mul(self.width).and_then(|v| v.checked_add(x2));

        let (pos1, pos2) = match (pos1, pos2) {
            (None, None) => return (None, None),
            (Some(pos1), None) => return (slice.get_mut(pos1), None),
            (None, Some(pos2)) => return (None, slice.get_mut(pos2)),
            (Some(pos1), Some(pos2)) => (pos1, pos2)
        };

        assert_ne!(pos1, pos2);

        if pos1 >= slice.len() || x1 >= self.width {
            if x2 >= slice.len() {
                return (None, None)
            } else {
                return (None, slice.get_mut(pos2))
            }
        } else if pos2 >= slice.len() || x2 >= self.width {
            if x1 >= slice.len() {
                return (None, None)
            } else {
                return (slice.get_mut(pos1), None)
            }
        }

        let (slice_before, slice_pos1) = slice.split_at_mut(pos1);
        if pos1 < pos2 {
            let (slice_pos1, slice_pos2) = slice_pos1.split_at_mut(pos2 - pos1);
            (Some(&mut slice_pos1[0]), Some(&mut slice_pos2[0]))
        } else {
            let (_slice_before, slice_pos2) = slice_before.split_at_mut(pos2);
            (Some(&mut slice_pos1[0]), Some(&mut slice_pos2[0]))
        }
    }
}

impl<'a, T> IntoIterator for &'a Array2D<T> {
    type Item = (usize, usize, &'a T);
    type IntoIter = IterArray2D<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterArray2D{ iter: (&self.inner).into_iter().enumerate(), width: self.width }
    }
}

pub struct IterArray2D<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, T>>,
    width: usize
}

impl<'a, T> Iterator for IterArray2D<'a, T> {
    type Item = (usize, usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (idx, val) = self.iter.next()?;

        Some((idx % self.width, idx / self.width, val))
    }
}

impl<'a, T> IntoIterator for &'a mut Array2D<T> {
    type Item = (usize, usize, &'a mut T);
    type IntoIter = IterMutArray2D<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMutArray2D{ iter: (&mut self.inner).iter_mut().enumerate(), width: self.width }
    }
}

pub struct IterMutArray2D<'a, T> {
    iter: std::iter::Enumerate<std::slice::IterMut<'a, T>>,
    width: usize
}

impl<'a, T> Iterator for IterMutArray2D<'a, T> {
    type Item = (usize, usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let (idx, val) = self.iter.next()?;

        Some((idx % self.width, idx / self.width, val))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn try_get_mut2() {
        let mut vec = Array2D::new(8, 8);
        *vec.get_mut(0, 0).unwrap() = 5;
        assert_eq!(vec.get_mut2(0, 0, 1, 0), (Some(&mut 5), Some(&mut 0)));
        assert_eq!(vec.get_mut2(0, 0, 8, 0), (Some(&mut 5), None));
        assert_eq!(vec.get_mut2(8, 0, 0, 0), (None, Some(&mut 5)));
    }
    #[test]
    #[should_panic]
    fn try_get_mut2_panic() {
        let mut vec = Array2D::<u8>::new(8, 8);
        vec.get_mut2(0, 0, 0, 0);
    }
}