use std::ops::Range;

use ratatui::layout::Rect;

use crate::{CouncilItemSelect, CreateNewEntry, ui::PASSWORD_GEN_SHOW_LENGTH};

impl CreateNewEntry {
    pub fn calculate_cursor_pos(&self, center_area: Rect) -> (u16, u16) {
        let center_offset = (((center_area.width - 3) as f32 * 0.333) + 1.0) as u16;
        (
            (center_area.x
                + 3
                + match self.selected {
                    CouncilItemSelect::Name => {
                        self.item.name.len() as u16 + center_offset - self.cursor_text_pos.name
                    }
                    CouncilItemSelect::Password if self.generate_phase => {
                        center_offset + center_area.width / PASSWORD_GEN_SHOW_LENGTH
                            - self
                                .cursor_text_pos
                                .password_gen
                                .min(center_area.width / PASSWORD_GEN_SHOW_LENGTH)
                    }
                    CouncilItemSelect::Password => {
                        self.item.password.len() as u16 + center_offset
                            - self.cursor_text_pos.password
                    }
                    CouncilItemSelect::First(i) => {
                        self.item.info[i].0.len() as u16 - self.cursor_text_pos.info[i].0
                    }
                    CouncilItemSelect::Second(i) => {
                        self.item.info[i].1.len() as u16 + center_offset
                            - self.cursor_text_pos.info[i].1
                    }
                })
            .min(center_area.x + center_area.width - 1),
            center_area.y
                + match self.selected {
                    CouncilItemSelect::Name => 1,
                    CouncilItemSelect::Password => 2,
                    CouncilItemSelect::First(i) | CouncilItemSelect::Second(i) => i + 3,
                } as u16,
        )
    }
    pub fn calculate_password_gen_range(&self, area: Rect) -> Range<usize> {
        let show_area = (area.width / PASSWORD_GEN_SHOW_LENGTH) as usize;
        let i = self.item.password_gen_rules.len().saturating_sub(show_area);
        let range = i.min(self.item.password_gen_rules.len().saturating_sub(show_area))
            ..(i + show_area).min(self.item.password_gen_rules.len());

        let correct_cursor_pos = self
            .item
            .password_gen_rules
            .len()
            .saturating_sub(self.cursor_text_pos.password_gen as usize);
        if correct_cursor_pos > range.end {
            let dif = correct_cursor_pos - range.end;
            (range.start + dif)..(range.end + dif)
        } else if correct_cursor_pos < range.start {
            let dif = range.start - correct_cursor_pos;
            (range.start - dif)..(range.end - dif)
        } else {
            range
        }
    }
}
