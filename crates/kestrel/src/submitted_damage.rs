use smithay::utils::{Physical, Rectangle, Size};
use std::collections::VecDeque;

const MAX_DAMAGE_HISTORY: usize = 4;

#[derive(Debug, Default)]
pub struct SubmittedDamageHistory {
    damage: VecDeque<Vec<Rectangle<i32, Physical>>>,
}

impl SubmittedDamageHistory {
    pub fn clear(&mut self) {
        self.damage.clear();
    }

    pub fn accumulate(
        &self,
        output_size: Size<i32, Physical>,
        current: &[Rectangle<i32, Physical>],
        buffer_age: usize,
    ) -> Vec<Rectangle<i32, Physical>> {
        let output = Rectangle::<i32, Physical>::from_size(output_size);
        if buffer_age == 0 || buffer_age > self.damage.len() + 1 {
            return vec![output];
        }

        let mut damage = current.to_vec();
        for past in self.damage.iter().take(buffer_age.saturating_sub(1)) {
            damage.extend(past.iter().copied());
        }
        shape_damage(output, damage)
    }

    pub fn record(
        &mut self,
        output_size: Size<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
    ) {
        self.damage.push_front(shape_damage(
            Rectangle::<i32, Physical>::from_size(output_size),
            damage.to_vec(),
        ));
        self.damage.truncate(MAX_DAMAGE_HISTORY);
    }
}

fn shape_damage(
    output: Rectangle<i32, Physical>,
    damage: Vec<Rectangle<i32, Physical>>,
) -> Vec<Rectangle<i32, Physical>> {
    let mut shaped: Vec<Rectangle<i32, Physical>> = Vec::new();
    for rect in damage {
        let Some(mut rect) = rect.intersection(output) else {
            continue;
        };
        if rect.is_empty() {
            continue;
        }
        let mut index = 0;
        while index < shaped.len() {
            if shaped[index].overlaps_or_touches(rect) {
                rect = rect.merge(shaped.swap_remove(index));
            } else {
                index += 1;
            }
        }
        shaped.push(rect);
    }
    shaped
}
