use super::utils;
use crate::app::App;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Frame;

pub(crate) fn draw_open_ch<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let size = rect.size();
    utils::check_size(&size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(6),
                Constraint::Min(10),
                Constraint::Length(10),
            ]
            .as_ref(),
        )
        .split(size);
    {
        let head_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), //
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        let tabs = utils::draw_tabs(app.get_state());
        rect.render_widget(tabs, head_chunks[0]);

        let balance = utils::draw_balance(app.get_state());
        rect.render_widget(balance, head_chunks[1]);
    }

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(4)].as_ref())
        .split(chunks[1]);

    let open_ch_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(body_chunks[0]);

    let (help_message, input, messages) =
        utils::draw_open_ch(app, rect, &chunks);

    rect.render_widget(help_message, open_ch_chunks[0]);
    rect.render_widget(input, open_ch_chunks[1]);
    rect.render_widget(messages, open_ch_chunks[2]);

    let help = utils::draw_help(app.get_actions());
    rect.render_widget(help, body_chunks[1]);

    let logs = utils::draw_logs();
    rect.render_widget(logs, chunks[2]);
}
