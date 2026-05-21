use crate::gui::{Message, RoseApp};
use iced::border::Radius;
use iced::event::Status;
use iced::mouse::Cursor;
use iced::widget::{
    Action, button, canvas, column, container, scrollable, stack, text, text_input, toggler,
};
use iced::{
    Alignment, Background, Border, Color, Element, Event, Length, Padding, Point, Rectangle,
    Renderer, Size, Theme, alignment,
};

const WS_WIDTH: f32 = 850.0;
const WS_HEIGHT: f32 = 780.0;

const X_CENTER: f32 = 425.0;
const X_LEFT: f32 = 200.0;
const X_RIGHT: f32 = 650.0;

const Y_INPUTS: f32 = 20.0;
const Y_FETCH: f32 = 180.0;
const Y_PARSE: f32 = 290.0;
const Y_SPLIT: f32 = 370.0;
const Y_NODES: f32 = 480.0;
const Y_MERGE: f32 = 590.0;
const Y_OUT: f32 = 670.0;

const BOX_H: f32 = 54.0;
const INPUT_W: f32 = 200.0;

struct PipelineSchematic {
    use_api: bool,
    use_align: bool,
}

impl canvas::Program<Message> for PipelineSchematic {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<Action<Message>> {
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let palette = theme.extended_palette();

        let active_wire = canvas::Stroke::default()
            .with_color(palette.primary.base.color)
            .with_width(2.5)
            .with_line_join(canvas::LineJoin::Round);

        let inactive_wire = canvas::Stroke::default()
            .with_color(palette.background.strong.color)
            .with_width(2.5)
            .with_line_join(canvas::LineJoin::Round);

        let draw_arrow_tip =
            |frame: &mut canvas::Frame, x: f32, y: f32, stroke: &canvas::Stroke| {
                let tip = canvas::Path::new(|p| {
                    p.move_to(Point::new(x - 6.0, y - 8.0));
                    p.line_to(Point::new(x, y));
                    p.line_to(Point::new(x + 6.0, y - 8.0));
                });
                frame.stroke(&tip, stroke.clone());
            };

        let draw_junction = |frame: &mut canvas::Frame, x: f32, y: f32, color: Color| {
            let dot = canvas::Path::circle(Point::new(x, y), 4.5);
            frame.fill(&dot, color);
        };

        let input_path = canvas::Path::new(|p| {
            p.move_to(Point::new(X_LEFT, Y_INPUTS + 80.0));
            p.line_to(Point::new(X_LEFT, 125.0));
            p.line_to(Point::new(X_CENTER, 125.0));
            p.line_to(Point::new(X_CENTER, Y_FETCH - 35.0));

            p.move_to(Point::new(X_RIGHT, Y_INPUTS + 80.0));
            p.line_to(Point::new(X_RIGHT, 125.0));
            p.line_to(Point::new(X_CENTER, 125.0));
        });
        frame.stroke(&input_path, active_wire.clone());
        draw_arrow_tip(&mut frame, X_CENTER, Y_FETCH - 35.0, &active_wire);

        let fetch_to_parse = canvas::Path::new(|p| {
            p.move_to(Point::new(X_CENTER, Y_FETCH + 27.0));
            p.line_to(Point::new(X_CENTER, Y_PARSE - 35.0));
        });
        frame.stroke(&fetch_to_parse, active_wire.clone());
        draw_arrow_tip(&mut frame, X_CENTER, Y_PARSE - 35.0, &active_wire);

        let trunk_to_split = canvas::Path::new(|p| {
            p.move_to(Point::new(X_CENTER, Y_PARSE + 27.0));
            p.line_to(Point::new(X_CENTER, Y_SPLIT));
        });
        frame.stroke(&trunk_to_split, active_wire.clone());

        let api_stroke = if self.use_api {
            &active_wire
        } else {
            &inactive_wire
        };
        let left_branch = canvas::Path::new(|p| {
            p.move_to(Point::new(X_CENTER, Y_SPLIT));
            p.line_to(Point::new(X_LEFT, Y_SPLIT));
            p.line_to(Point::new(X_LEFT, Y_NODES - 35.0));

            p.move_to(Point::new(X_LEFT, Y_NODES + 27.0));
            p.line_to(Point::new(X_LEFT, Y_MERGE));
            p.line_to(Point::new(X_CENTER, Y_MERGE));
        });
        frame.stroke(&left_branch, api_stroke.clone());
        draw_arrow_tip(&mut frame, X_LEFT, Y_NODES - 35.0, api_stroke);

        let align_stroke = if self.use_align {
            &active_wire
        } else {
            &inactive_wire
        };
        let right_branch = canvas::Path::new(|p| {
            p.move_to(Point::new(X_CENTER, Y_SPLIT));
            p.line_to(Point::new(X_RIGHT, Y_SPLIT));
            p.line_to(Point::new(X_RIGHT, Y_NODES - 35.0));

            p.move_to(Point::new(X_RIGHT, Y_NODES + 27.0));
            p.line_to(Point::new(X_RIGHT, Y_MERGE));
            p.line_to(Point::new(X_CENTER, Y_MERGE));
        });
        frame.stroke(&right_branch, align_stroke.clone());
        draw_arrow_tip(&mut frame, X_RIGHT, Y_NODES - 35.0, align_stroke);

        draw_junction(&mut frame, X_CENTER, Y_SPLIT, palette.primary.base.color);

        let merge_to_run = canvas::Path::new(|p| {
            p.move_to(Point::new(X_CENTER, Y_MERGE));
            p.line_to(Point::new(X_CENTER, Y_OUT - 8.0));
        });

        let out_stroke = if self.use_api || self.use_align {
            &active_wire
        } else {
            &inactive_wire
        };
        frame.stroke(&merge_to_run, out_stroke.clone());
        draw_arrow_tip(&mut frame, X_CENTER, Y_OUT - 8.0, out_stroke);
        draw_junction(
            &mut frame,
            X_CENTER,
            Y_MERGE,
            if self.use_api || self.use_align {
                palette.primary.base.color
            } else {
                palette.background.strong.color
            },
        );

        let mut hovered_tooltip: Option<(&str, Point)> = None;

        let nodes = vec![
            (
                X_CENTER,
                Y_FETCH,
                220.0,
                BOX_H,
                "Fetch records from NCBI",
                "Connects to NCBI Entrez and downloads fasta/xml data",
                true,
            ),
            (
                X_CENTER,
                Y_PARSE,
                220.0,
                BOX_H,
                "Find genes in records",
                "Extracts genetic sequences from the downloaded records",
                true,
            ),
            (
                X_LEFT,
                Y_NODES,
                260.0,
                BOX_H,
                "Parse annotations in records",
                "Parses XML features to extract rich annotations",
                self.use_api,
            ),
            (
                X_RIGHT,
                Y_NODES,
                280.0,
                BOX_H,
                "Align using SW",
                "Try aligning all genes in source organism to all genes in target organism",
                self.use_align,
            ),
        ];

        for (x, y, w, h, label, tooltip, is_active) in nodes {
            let rect = Rectangle::new(Point::new(x - w / 2.0, y - h / 2.0), Size::new(w, h));

            let path =
                canvas::Path::rounded_rectangle(rect.position(), rect.size(), Radius::from(4.0));
            frame.fill(&path, palette.background.base.color);

            let border = if is_active {
                canvas::Stroke::default()
                    .with_color(palette.primary.base.color)
                    .with_width(2.0)
            } else {
                canvas::Stroke::default()
                    .with_color(palette.background.strong.color)
                    .with_width(2.0)
            };
            frame.stroke(&path, border);

            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: Point::new(x, y),
                color: palette.background.base.text,
                size: 15.0.into(),
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }
}

fn absolute_pos<'a>(
    widget: impl Into<Element<'a, Message>>,
    x: f32,
    y: f32,
) -> Element<'a, Message> {
    container(widget)
        .width(Length::Fixed(WS_WIDTH))
        .height(Length::Fixed(WS_HEIGHT))
        .align_x(alignment::Horizontal::Left)
        .align_y(alignment::Vertical::Top)
        .padding(Padding {
            top: y,
            left: x,
            right: 0.0,
            bottom: 0.0,
        })
        .into()
}

pub fn view(app: &RoseApp) -> Element<Message> {
    let schematic_layer = canvas(PipelineSchematic {
        use_api: app.ui_use_api,
        use_align: app.ui_use_align,
    })
    .width(Length::Fixed(WS_WIDTH))
    .height(Length::Fixed(WS_HEIGHT));

    let src_input = container(
        column![
            text("Source Organism").size(14),
            text_input("e.g. Homo sapiens", &app.source_input)
                .on_input(Message::SourceChanged)
                .padding(8),
        ]
        .spacing(4),
    )
    .width(INPUT_W)
    .padding(12)
    .style(input_box_style);

    let tgt_input = container(
        column![
            text("Target Organism").size(14),
            text_input("e.g. Homo erectus", &app.target_input)
                .on_input(Message::TargetChanged)
                .padding(8),
        ]
        .spacing(4),
    )
    .width(INPUT_W)
    .padding(12)
    .style(input_box_style);

    let api_switch = container(
        toggler(!app.ui_use_api)
            .on_toggle(|b| Message::ToggleApiMethod(!b))
            .style(left_toggler_style),
    )
    .padding(Padding {
        top: 4.0,
        bottom: 4.0,
        left: 6.0,
        right: 6.0,
    })
    .style(toggler_track_style);

    let align_switch = container(
        toggler(app.ui_use_align)
            .on_toggle(Message::ToggleAlignMethod)
            .style(right_toggler_style),
    )
    .padding(Padding {
        top: 4.0,
        bottom: 4.0,
        left: 6.0,
        right: 6.0,
    })
    .style(toggler_track_style);

    let execute_btn = container(
        container(
            button(text("RUN").size(16))
                .on_press(Message::StartSearch)
                .padding([8, 20]),
        )
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center),
    )
    .width(120.0);

    let fixed_workspace = container(stack![
        schematic_layer,
        absolute_pos(src_input, X_LEFT - INPUT_W / 2.0, Y_INPUTS),
        absolute_pos(tgt_input, X_RIGHT - INPUT_W / 2.0, Y_INPUTS),
        absolute_pos(api_switch, 310.0, Y_SPLIT - 12.0),
        absolute_pos(align_switch, 495.0, Y_SPLIT - 12.0),
        absolute_pos(execute_btn, X_CENTER - 60.0, Y_OUT)
    ])
    .width(Length::Fixed(WS_WIDTH))
    .height(Length::Fixed(WS_HEIGHT));

    scrollable(fixed_workspace)
        .direction(scrollable::Direction::Both {
            vertical: scrollable::Scrollbar::new().width(12).scroller_width(8),
            horizontal: scrollable::Scrollbar::new().width(12).scroller_width(8),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn left_toggler_style(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let palette = theme.extended_palette();

    let is_toggled = match status {
        toggler::Status::Active { is_toggled } => is_toggled,
        toggler::Status::Hovered { is_toggled } => is_toggled,
        _ => false,
    };

    let is_on = !is_toggled;

    toggler::Style {
        background: Background::from(if is_on {
            palette.primary.base.color
        } else {
            palette.background.strong.color
        }),
        background_border_width: 1.0,
        background_border_color: palette.background.strong.color,
        foreground: Background::from(palette.background.base.color),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        text_color: None,
        border_radius: Option::from(Radius::from(20.0)),
        padding_ratio: 0.1,
    }
}

fn right_toggler_style(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let palette = theme.extended_palette();

    let is_toggled = match status {
        toggler::Status::Active { is_toggled } => is_toggled,
        toggler::Status::Hovered { is_toggled } => is_toggled,
        _ => false,
    };

    let is_on = is_toggled;

    toggler::Style {
        background: Background::from(if is_on {
            palette.primary.base.color
        } else {
            palette.background.strong.color
        }),
        background_border_width: 1.0,
        background_border_color: palette.background.strong.color,
        foreground: Background::from(palette.background.base.color),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        text_color: None,
        border_radius: Option::from(Radius::from(20.0)),
        padding_ratio: 0.1,
    }
}

fn toggler_track_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(palette.background.base.color)),
        border: Default::default(),
        ..Default::default()
    }
}

fn input_box_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(palette.background.base.color)),
        border: Border {
            color: palette.primary.base.color,
            width: 2.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

fn run_button_style(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    button::Style {
        background: Some(Background::Color(palette.primary.base.color)),
        text_color: palette.primary.base.text,
        border: Border {
            color: palette.primary.base.color,
            width: 2.0,
            radius: 4.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}
