use super::{AppModel, Message, icons};
use crate::fl;
use crate::services::clipboard;
use cosmic::iced::widget::Stack;
use cosmic::iced::widget::image::Handle as ImageHandle;
use cosmic::iced::{Alignment, Length, window::Id};
use cosmic::prelude::*;
use cosmic::widget;

pub fn view(app: &AppModel) -> Element<'_, Message> {
    app.core
        .applet
        .icon_button("edit-copy-symbolic")
        .on_press(Message::TogglePopup)
        .into()
}

pub fn view_window(app: &AppModel, _id: Id) -> Element<'_, Message> {
    let theme = cosmic::theme::active();
    let is_dark = theme.theme_type.is_dark();
    let icon_color = if is_dark { "#dcdcdc" } else { "#2e3436" };

    let mut history_column = widget::column::Column::new().spacing(4);

    if app.history.is_empty() {
        history_column = history_column.push(widget::text::body(fl!("empty")));
    } else {
        let pinned_count = app.history.iter().filter(|it| it.pinned).count();

        for (idx, item) in app.history.iter().enumerate() {
            // Divider between pinned and unpinned sections
            if idx == pinned_count && pinned_count > 0 && pinned_count < app.history.len() {
                history_column = history_column.push(widget::divider::horizontal::default());
            }

            let is_hovered = app.hovered_index == Some(idx);

            let label: Element<'_, Message> = match &item.entry {
                clipboard::ClipboardEntry::Text(text) => {
                    widget::text::body(summarize_one_line(text)).into()
                }
                clipboard::ClipboardEntry::Image {
                    mime,
                    bytes,
                    thumbnail_png,
                    ..
                } => {
                    let thumb = thumbnail_png.as_ref().map(|png| {
                        widget::image(ImageHandle::from_bytes(png.clone()))
                            .width(Length::Fill)
                            .content_fit(cosmic::iced::ContentFit::Contain)
                    });

                    let mut col = widget::column::Column::new()
                        .width(Length::Fill)
                        .align_x(Alignment::Center);
                    if let Some(thumb) = thumb {
                        col = col.push(thumb);
                    }
                    if is_hovered {
                        col = col.push(
                            widget::text::caption(format!(
                                "{} ({} KB)",
                                mime,
                                (bytes.len().saturating_add(1023)) / 1024
                            ))
                            .width(Length::Fill),
                        );
                    }
                    col.into()
                }
            };

            let copy_button = widget::button::custom(label)
                .class(cosmic::theme::Button::MenuItem)
                .on_press(Message::CopyFromHistory(idx))
                .width(Length::Fill)
                .padding([8, 12]);

            let entry: Element<'_, Message> = if is_hovered {
                let pin_button = widget::button::icon(if item.pinned {
                    icons::pin_icon_pinned()
                } else {
                    icons::pin_icon(icon_color)
                })
                .tooltip(if item.pinned {
                    fl!("unpin")
                } else {
                    fl!("pin")
                })
                .on_press(Message::TogglePin(idx))
                .extra_small()
                .width(Length::Shrink);

                let remove_button = widget::button::icon(icons::remove_icon(icon_color))
                    .tooltip(fl!("remove"))
                    .on_press(Message::RemoveHistory(idx))
                    .extra_small()
                    .width(Length::Shrink);

                let actions_overlay = widget::row::Row::new()
                    .push(widget::horizontal_space())
                    .push(pin_button)
                    .push(remove_button)
                    .spacing(2)
                    .padding([0, 4])
                    .align_y(Alignment::Center);

                Stack::with_children(vec![copy_button.into(), actions_overlay.into()])
                    .width(Length::Fill)
                    .into()
            } else {
                copy_button.into()
            };

            history_column = history_column.push(
                widget::container(
                    widget::mouse_area(entry)
                        .on_enter(Message::HoverEntry(Some(idx)))
                        .on_exit(Message::HoverEntry(None)),
                )
                .class(cosmic::theme::Container::Card)
                .width(Length::Fill),
            );
        }
    }

    // Grow with content up to 400 px, then scroll.
    let history_scrollable = widget::container(
        widget::scrollable(history_column).width(Length::Fill),
    )
    .max_height(400.0)
    .width(Length::Fill);

    // On a destructive button the background is red; the icon must contrast with it,
    // which is the inverse of the neutral-background icon color.
    let destructive_icon_color = if is_dark { "#2e3436" } else { "#dcdcdc" };

    let delete_all_button = widget::button::destructive(fl!("delete-all"))
        .leading_icon(icons::remove_icon(destructive_icon_color))
        .on_press(Message::ClearHistory);

    let controls_sheet = widget::container(delete_all_button)
        .padding([8, 8])
        .align_x(Alignment::End)
        .width(Length::Fill);

    let content = widget::column::Column::new()
        .spacing(8)
        .padding([8, 8])
        .push(history_scrollable)
        .push(controls_sheet);

    app.core.applet.popup_container(content).into()
}

fn summarize_one_line(text: &str) -> String {
    let mut line = text
        .lines()
        .map(|line| line.trim_start())
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .trim_end()
        .to_string();
    const MAX_CHARS: usize = 60;
    if line.chars().count() > MAX_CHARS {
        line = line.chars().take(MAX_CHARS - 1).collect::<String>();
        line.push('…');
    }
    line
}

#[cfg(test)]
mod tests {
    use super::summarize_one_line;

    #[test]
    fn summarizes_first_nonempty_line() {
        let input = "\n   \n  hello world  \nsecond line";
        assert_eq!(summarize_one_line(input), "hello world");
    }

    #[test]
    fn truncates_long_lines_with_ellipsis() {
        let input = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnop";
        assert_eq!(
            summarize_one_line(input),
            "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefg…"
        );
    }

    #[test]
    fn returns_empty_for_blank_text() {
        assert_eq!(summarize_one_line("\n  \n\t"), "");
    }
}
