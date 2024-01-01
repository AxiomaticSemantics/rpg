use crate::{plugin::UiFont, style::UiTheme};

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::EventReader,
        query::{Changed, With},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::{BuildChildren, ChildBuilder, Parent},
    input::{
        keyboard::KeyCode,
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        ButtonInput,
    },
    math::{Rect, Vec2, Vec3, Vec3Swizzles},
    render::color::Color,
    text::{Text, TextStyle},
    transform::components::GlobalTransform,
    ui::{
        node_bundles::{ImageBundle, NodeBundle, TextBundle},
        AlignContent, AlignItems, AlignSelf, BackgroundColor, Display, FlexDirection, Interaction,
        JustifyContent, Node, Overflow, OverflowAxis, Style, UiRect, Val,
    },
    utils::default,
    window::{PrimaryWindow, ReceivedCharacter, Window},
};

pub trait RangedValue<T>
where
    T: Copy + PartialEq + PartialOrd + Cast<f32>,
{
    type Item;

    fn min(&self) -> T;
    fn max(&self) -> T;
    fn set(&mut self, value: T);
}

#[derive(Default, Debug, Resource)]
pub struct FocusedElement(pub Option<Entity>);

#[derive(Debug, Default, Clone, Component)]
pub struct ImageButton;

#[derive(Bundle, Default)]
pub struct ImageButtonBundle {
    pub marker: ImageButton,
    pub image: ImageBundle,
    pub interaction: Interaction,
}

#[derive(Default, Debug)]
pub struct Cursor {
    pub min: Option<usize>,
    pub max: Option<usize>,
    // For now all cursors allow 0 as a valid position
    pub position: usize,

    pub rect: Option<UiRect>,
    pub color: Option<Color>,
}

impl Cursor {
    pub fn with_min_max(position: usize, min: Option<usize>, max: Option<usize>) -> Self {
        let position = position.clamp(min.unwrap_or_default(), max.unwrap_or(usize::MAX));

        Self {
            position,
            min,
            max,
            ..default()
        }
    }

    pub fn set_rect(&mut self, rect: Option<UiRect>) {
        self.rect = rect;
    }

    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    pub fn get_min(&self) -> usize {
        self.min.unwrap_or_default()
    }

    pub fn get_max(&self) -> usize {
        self.max.unwrap_or(usize::MAX)
    }

    pub fn reset_position(&mut self) {
        self.position = self.get_min();
    }

    pub fn in_range(&self) -> bool {
        self.position >= self.get_min() && self.position <= self.get_max()
    }

    pub fn increment(&mut self) -> bool {
        let original = self.position;
        let new = (original + 1).clamp(self.get_min(), self.get_max());

        if new != original {
            self.position = new;
        }

        original == new
    }

    pub fn decrement(&mut self) -> bool {
        let original = self.position;
        let new = original
            .saturating_sub(1)
            .clamp(self.get_min(), self.get_max());

        if new != original {
            self.position = new;
        }

        new == original
    }

    pub fn set_min(&mut self, min: Option<usize>) {
        self.min = min;
        let min = min.unwrap_or_default();

        if self.position < min {
            self.position = min;
        }
    }

    pub fn set_max(&mut self, max: Option<usize>) {
        self.max = max;
        let max = max.unwrap_or(usize::MAX);

        if self.position > max {
            self.position = max;
        }
    }
}

#[derive(Debug)]
pub enum ListPosition {
    Index(usize),
    Position(f32),
}

impl Default for ListPosition {
    fn default() -> Self {
        Self::Index(0)
    }
}

#[derive(Component, Default)]
pub struct List {
    pub position: ListPosition,
    pub scrollable: bool,
}

#[derive(Component, Default)]
pub struct ResizeableView;

#[derive(Component, Default)]
pub struct SliderBar;

#[derive(Debug, Component, Default)]
pub struct Slider<T: std::fmt::Debug + Cast<f32>> {
    pub size: Rect,
    pub bar_size: Rect,
    pub min: T,
    pub max: T,
    pub position: T,
}

impl<T> RangedValue<T> for Slider<T>
where
    T: PartialEq + PartialOrd + std::fmt::Debug + Copy + Cast<f32>,
{
    type Item = T;

    fn min(&self) -> T {
        self.min
    }

    fn max(&self) -> T {
        self.max
    }

    fn set(&mut self, value: T) {
        if value >= self.min() && value <= self.max() {
            self.position = value;
        }
    }
}

// TODO FIXME this needs to be extended to handle arbitrary input behaviours
#[derive(Debug, Default, PartialEq)]
pub enum NewlineBehaviour {
    #[default]
    Allow,
    Replace,
    Consume,
    ConsumeSilent,
}

#[derive(Component, Debug, Default)]
// TODO a fine-grained policy probably regex based is in order for IO filtering
pub struct EditText {
    pub newline_behaviour: NewlineBehaviour,
    pub replace_control: bool,
    pub prefix: Option<&'static str>,
    pub cursor: Cursor,
    pub consume: bool,
}

impl EditText {
    pub fn new(newline_behaviour: NewlineBehaviour) -> Self {
        Self {
            newline_behaviour,
            ..default()
        }
    }

    pub fn set_prefix(&mut self, prefix: Option<&'static str>) {
        let len = prefix.unwrap_or_default().len();

        self.prefix = prefix;

        self.cursor.set_min(Some(len + 1));
        self.cursor.set_max(Some(len + 1));
    }
}

pub fn setup_focus(mut commands: Commands) {
    commands.insert_resource(FocusedElement(None));
}

pub fn edit_focus_update(
    ui_theme: Res<UiTheme>,
    mut focused_element: ResMut<FocusedElement>,
    mut edit_text_q: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<EditText>),
    >,
) {
    for (entity, interaction, mut bg_color) in &mut edit_text_q {
        match &interaction {
            Interaction::Pressed => {
                if let Some(focused) = &mut focused_element.0 {
                    println!("updating focus");
                    if *focused != entity {
                        *focused = entity;
                    }
                } else {
                    println!("set focus");
                    focused_element.0 = Some(entity);
                }

                return;
            }
            Interaction::Hovered => *bg_color = ui_theme.button_theme.hovered_background_color,
            Interaction::None => *bg_color = ui_theme.button_theme.normal_background_color,
        }
    }
}

pub fn resize_view(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut resizeable_view_q: Query<
        (&mut Style, &Interaction, &GlobalTransform, &Node),
        (Changed<Interaction>, With<ResizeableView>),
    >,
) {
    let window = window_q.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    for (mut style, interaction, global_transform, node) in &mut resizeable_view_q {
        if interaction == &Interaction::Hovered || interaction == &Interaction::Pressed {
            let window_size = Vec2::new(window.width(), window.height());

            let rect = get_node_rect(node, &global_transform.translation());

            match &mut style.height {
                Val::Percent(ref mut height) => {
                    if cursor_position.y > rect.min.y && cursor_position.y < rect.min.y + 8. {
                        *height += 2.;
                    } else if cursor_position.y > rect.max.y - 8. && cursor_position.y < rect.max.y
                    {
                        *height -= 2.;
                    }
                }
                Val::Px(ref mut _height) => todo!(),
                _ => {}
            }

            match &mut style.width {
                Val::Percent(ref mut width) => {
                    println!("resize {}", node.size(),);

                    if cursor_position.x > rect.min.x && cursor_position.x < rect.min.x + 8. {
                        *width += 2.;
                    } else if cursor_position.x > rect.max.x - 8. && cursor_position.x < rect.max.x
                    {
                        *width -= 2.;
                    }
                }
                Val::Px(ref mut _height) => todo!(),
                _ => {}
            }

            /* println!(
                "style width {:?} height {:?} translation {:?} node size {:?} window_cursor {:?}",
                style.width,
                style.height,
                transform.translation,
                node.size(),
                cursor_position
            ); */
        }
    }
}

pub trait Cast<A> {
    fn cast(self) -> A;
}

impl Cast<u32> for f32 {
    fn cast(self) -> u32 {
        self as u32
    }
}

impl Cast<f32> for u32 {
    fn cast(self) -> f32 {
        self as f32
    }
}

fn get_node_rect(node: &Node, origin: &Vec3) -> Rect {
    let size = node.size();
    let half_size = size / 2.;

    let top_left = origin.xy() - half_size;
    let bottom_right = origin.xy() + half_size;
    Rect::from_corners(top_left, bottom_right)
}

pub fn slider_update(
    ui_theme: Res<UiTheme>,
    mut slider_q: Query<(&Node, &mut Slider<u32>, &GlobalTransform)>,
    mut slider_inner_q: Query<
        (
            &Interaction,
            &mut Style,
            &Node,
            &GlobalTransform,
            &mut BackgroundColor,
            &Parent,
        ),
        With<SliderBar>,
    >,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    for (interaction, mut style, node, transform, mut bg_color, parent) in &mut slider_inner_q {
        match interaction {
            Interaction::Pressed => {
                let (parent_node, mut slider, parent_transform) =
                    slider_q.get_mut(parent.get()).unwrap();
                let parent_size = parent_node.size();
                let size = node.size();

                let max_x = parent_size.x - size.x;
                let window = window_q.single();
                let mouse_position = window.cursor_position().unwrap_or_default();

                let node_rect = get_node_rect(node, &transform.translation());
                let parent_rect = get_node_rect(parent_node, &parent_transform.translation());

                if mouse_position.x >= parent_rect.min.x && mouse_position.x <= parent_rect.max.x {
                    let target_pos = parent_size.x - (parent_rect.max.x - mouse_position.x);

                    let percent = target_pos / parent_size.x;
                    let value: f32 = slider.max.cast() * percent;
                    let u_value = value.round().cast();
                    slider.set(u_value);

                    //println!("slider: {slider:?} % {percent} value {value}");
                    let left_pos = (target_pos - size.x / 2.).clamp(0., max_x);
                    style.left = Val::Px(left_pos);
                    style.right = Val::Px(left_pos + size.x);
                    /*println!(
                        "slider mouse_dx {mouse_position} {} {parent_size} p_left {parent_left} {parent_right} {target_pos}",
                        parent_transform.translation()
                    );*/
                }
            }
            Interaction::Hovered => *bg_color = ui_theme.button_theme.hovered_background_color,
            Interaction::None => *bg_color = ui_theme.button_theme.normal_background_color,
        }
    }
}

pub fn edit_text(
    mut input_chars: EventReader<ReceivedCharacter>,
    input: Res<ButtonInput<KeyCode>>,
    focused_element: Res<FocusedElement>,
    mut edit_text_q: Query<(Entity, &mut EditText, &mut Text, &Style)>,
) {
    let Some(focused_element) = focused_element.0 else {
        return;
    };

    for (entity, mut edit_text, mut text, style) in &mut edit_text_q {
        if entity != focused_element {
            println!("skipping non-focused edit_text");
            continue;
        }

        if style.display == Display::None {
            println!("skipping non-displayable edit_text");
            continue;
        }

        /*println!("cursor: {:?} text len: {len}");*/
        assert!(edit_text.cursor.in_range());

        for input in input_chars.read() {
            let (can_add, ch) = match input.char.as_str() {
                // `Backspace`
                "\u{8}" => {
                    assert!(edit_text.cursor.in_range());

                    let len = text.sections[0].value.len();
                    let min = edit_text.cursor.get_min();
                    if edit_text.cursor.position > min {
                        if edit_text.cursor.position >= len {
                            text.sections[0].value.pop();
                        } else if len > 1 {
                            text.sections[0].value.remove(edit_text.cursor.position);
                        }
                        edit_text
                            .cursor
                            .set_max(Some(text.sections[0].value.len() + 1));
                    }

                    assert!(edit_text.cursor.in_range());
                    println!("backspace {}", text.sections[0].value);

                    (false, '\u{0}')
                }
                // `Delete`
                "\u{7f}" => {
                    let len = text.sections[0].value.len();
                    let pos = edit_text.cursor.position.saturating_sub(1);
                    if len > 0 && len <= edit_text.cursor.position {
                        text.sections[0].value.remove(pos);
                        edit_text.cursor.set_max(Some(len));
                    }
                    (false, '\u{0}')
                }
                ch if ch.chars().next().unwrap().is_control()
                    && ch.chars().next().unwrap().is_whitespace() =>
                {
                    if ch == "\r" || ch == "\n" {
                        // TODO FIXME Improve this at some point
                        // println!("{:?} `{}`", edit_text, text.sections[0].value);

                        match edit_text.newline_behaviour {
                            NewlineBehaviour::Consume | NewlineBehaviour::ConsumeSilent => {
                                edit_text.consume = true;

                                (false, '\u{0}')
                            }
                            NewlineBehaviour::Allow => (true, ch.chars().next().unwrap()),
                            NewlineBehaviour::Replace => (true, '\n'),
                        }
                    } else {
                        // FIXME input replacement policy
                        println!("adding whitespace: `{ch}`");

                        (true, ' ')
                    }
                }
                ch if ch.chars().next().unwrap().is_control() => {
                    println!("rejecting control: `{ch:#?}`");

                    (false, ch.chars().next().unwrap())
                }
                ch => {
                    //println!("adding: `{ch}`");

                    (true, ch.chars().next().unwrap())
                }
            };

            if !can_add {
                continue;
            }

            let len = text.sections[0].value.len();
            edit_text.cursor.set_max(Some(len));

            if edit_text.cursor.position >= len {
                text.sections[0].value.push(ch);
                edit_text.cursor.position = len;
            } else {
                edit_text.cursor.increment();
                text.sections[0].value.insert(edit_text.cursor.position, ch);
            }

            assert!(edit_text.cursor.position <= text.sections[0].value.len());
        }

        if input.just_pressed(KeyCode::ArrowRight) {
            let position = edit_text.cursor.position;
            edit_text.cursor.increment();
            println!("right {position} -> {}", edit_text.cursor.position);
        } else if input.just_pressed(KeyCode::ArrowLeft) {
            let position = edit_text.cursor.position;
            edit_text.cursor.decrement();
            println!("left {position} -> {}", edit_text.cursor.position);
        }
    }
}

pub fn build_horizontal_bar<L: Component, B: Component>(
    parent: &mut ChildBuilder,
    ui_theme: &UiTheme,
    fill_color: Color,
    label_marker: L,
    bar_marker: B,
    width: Val,
    height: Val,
) {
    let bar_style = Style {
        min_width: width,
        min_height: height,
        border: UiRect::all(ui_theme.border),
        margin: UiRect::all(ui_theme.margin),
        padding: UiRect::all(ui_theme.padding),
        flex_direction: FlexDirection::Row,
        ..default()
    };

    build_bar_label(parent, ui_theme, label_marker);

    parent
        .spawn(NodeBundle {
            style: bar_style,
            background_color: ui_theme.frame_background_color,
            border_color: ui_theme.frame_border_color,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                NodeBundle {
                    style: Style {
                        // FIXME margin and borders don't play well with sized nodes
                        //margin: UiRect::all(ui_theme.margin),
                        //padding: UiRect::all(ui_theme.style_padding),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    background_color: fill_color.into(),
                    border_color: ui_theme.border_color,
                    ..default()
                },
                bar_marker,
            ));
        });
}

pub fn _build_vertical_bar<B: Component>(
    parent: &mut ChildBuilder,
    ui_theme: &UiTheme,
    fill_color: Color,
    marker: B,
    width: Val,
    height: Val,
) {
    let bar_style = Style {
        width,
        height,
        border: UiRect::all(ui_theme.border),
        margin: UiRect::all(ui_theme.margin),
        padding: UiRect::all(ui_theme.padding),
        flex_direction: FlexDirection::Column,
        ..default()
    };

    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(96.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: Color::rgb(0.45, 0.4, 0.5).into(),
            ..default()
        })
        .with_children(|p| {
            // Bar
            p.spawn(NodeBundle {
                style: bar_style,
                background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                ..default()
            })
            .with_children(|p| {
                p.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        background_color: fill_color.into(),
                        ..default()
                    },
                    marker,
                ));
            });
        });
}

fn _spawn_scrolling_list(mut commands: Commands, ui_font: Res<UiFont>) {
    commands
        .spawn(
            TextBundle::from_section(
                "Scrolling list",
                TextStyle {
                    font: ui_font.fira_sans.clone(),
                    font_size: 24.,
                    color: Color::WHITE,
                },
            )
            .with_style(Style {
                height: Val::Px(24.),
                ..default()
            }),
        )
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    align_self: AlignSelf::Center,
                    width: Val::Percent(96.0),
                    height: Val::Percent(48.0),
                    overflow: Overflow::DEFAULT,
                    ..default()
                },
                background_color: Color::rgb(0.10, 0.10, 0.10).into(),
                ..default()
            })
            .with_children(|p| {
                p.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::ColumnReverse,
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    },
                    List::default(),
                ))
                .with_children(|p| {
                    for i in 0..30 {
                        p.spawn(
                            TextBundle::from_section(
                                format!("Item {:}", i + 1),
                                TextStyle {
                                    font: ui_font.fira_sans.clone(),
                                    font_size: 20.,
                                    color: Color::WHITE,
                                },
                            )
                            .with_style(Style {
                                height: Val::Px(20.),
                                margin: UiRect {
                                    left: Val::Auto,
                                    right: Val::Auto,
                                    ..default()
                                },
                                ..default()
                            }),
                        );
                    }
                });
            });
        });
}

pub fn build_bar_label<T: Component>(
    parent: &mut ChildBuilder,
    ui_theme: &UiTheme,
    text_marker: T,
) {
    let node = NodeBundle {
        style: ui_theme.frame_col_style.clone(),
        background_color: ui_theme.frame_background_color,
        border_color: ui_theme.frame_border_color,
        ..default()
    };
    parent.spawn(node).with_children(|p| {
        p.spawn((
            TextBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: ui_theme.font.clone(),
                        font_size: ui_theme.font_size_regular,
                        color: ui_theme.text_color_light,
                    },
                ),
                style: Style {
                    align_items: AlignItems::Center,
                    margin: UiRect::all(ui_theme.margin),
                    ..default()
                },
                background_color: ui_theme.frame_background_color,
                ..default()
            },
            text_marker,
        ));
    });
}

pub fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut scroll_list_q: Query<(&mut List, &mut Style, &Parent, &Node)>,
    item_q: Query<&Node>,
) {
    let dy: f32 = mouse_wheel_events
        .read()
        .map(|e| match e.unit {
            MouseScrollUnit::Line => e.y * 24.,
            MouseScrollUnit::Pixel => e.y,
        })
        .sum();

    for (mut scrolling_list, mut style, parent, list_node) in &mut scroll_list_q {
        let items_height = list_node.size().y;
        let container_height = item_q.get(parent.get()).unwrap().size().y;
        let max_scroll = (items_height - container_height).max(0.);
        if max_scroll <= std::f32::EPSILON {
            if style.bottom != Val::Px(items_height) {
                style.bottom = Val::Px(items_height);
            }
            continue;
        }

        match &mut scrolling_list.position {
            ListPosition::Index(_) => {}
            ListPosition::Position(ref mut position) => {
                // scroll position is relative to the bottom
                let dy = (*position - dy).clamp(0., max_scroll);
                if dy != *position {
                    println!(
                        "items {items_height} panel {container_height} dy {dy} max {max_scroll} pos {position}"
                    );
                    *position = dy;
                    let position = Val::Px(dy);

                    if style.bottom != position {
                        style.bottom = position;
                    }
                }

                //println!("scrolled to {position:?} top {:?}", style.top);
            }
        }
    }
}
