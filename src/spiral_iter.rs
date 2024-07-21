pub(crate) struct SpiralIter {
    items_remaining: usize,
    next_x: isize,
    next_y: isize,
    curr_dir: (i8, i8),
    already_gone_that_much: bool,
    total_steps_in_curr_dir: usize,
    steps_left_in_curr_dir: usize,
}

impl SpiralIter {
    pub fn new(count: usize) -> Self {
        Self {
            items_remaining: count,
            next_x: 0,
            next_y: 0,
            curr_dir: (1, 0),
            already_gone_that_much: false,
            total_steps_in_curr_dir: 1,
            steps_left_in_curr_dir: 1,
        }
    }
}

impl Iterator for SpiralIter {
    type Item = (isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.items_remaining == 0 {
            None
        } else {
            self.items_remaining -= 1;
            let ret_val = Some((self.next_x, self.next_y));

            let (dx, dy) = self.curr_dir;
            self.next_x += isize::from(dx);
            self.next_y += isize::from(dy);
            self.steps_left_in_curr_dir -= 1;
            if self.steps_left_in_curr_dir == 0 {
                if self.already_gone_that_much {
                    self.total_steps_in_curr_dir += 1;
                    self.already_gone_that_much = false;
                } else {
                    self.already_gone_that_much = true;
                }

                self.steps_left_in_curr_dir = self.total_steps_in_curr_dir;
                self.curr_dir = (dy * -1, dx * 1);
            }

            ret_val
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        return (self.items_remaining, Some(self.items_remaining));
    }
}

impl ExactSizeIterator for SpiralIter {
    fn len(&self) -> usize {
        self.items_remaining
    }
}
