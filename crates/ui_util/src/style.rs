use crate::{plugin::UiFont, widgets::ImageButton};

use bevy::{
    asset::Handle,
    ecs::{
        component::Component,
        query::{Changed, Or, With},
        system::{Query, Res, ResMut, Resource},
    },
    render::color::Color,
    text::{Font, TextStyle},
    ui::{
        widget::Button, AlignItems, AlignSelf, BackgroundColor, BorderColor, FlexDirection,
        Interaction, JustifyContent, PositionType, Style, TargetCamera, UiRect, Val,
    },
    utils::default,
};

#[derive(Component)]
pub struct UiRoot(pub Option<TargetCamera>);

#[derive(Debug, Default)]
pub struct ButtonTheme {
    pub normal_background_color: BackgroundColor,
    pub normal_border_color: Color,
    pub pressed_background_color: BackgroundColor,
    pub pressed_border_color: Color,
    pub hovered_background_color: BackgroundColor,
    pub hovered_border_color: Color,
    pub style: Style,
}

#[derive(Debug, Default, Resource)]
pub struct UiTheme {
    pub button_theme: ButtonTheme,
    pub text_color_dark: Color,
    pub text_color_light: Color,
    pub vertical_spacer: Style,
    pub horizontal_spacer: Style,
    pub container_absolute_max: Style,
    pub frame_row_style: Style,
    pub frame_col_style: Style,
    pub row_style: Style,
    pub col_style: Style,
    pub background_color: BackgroundColor,
    pub menu_background_color: BackgroundColor,
    pub frame_background_color: BackgroundColor,
    pub frame_border_color: BorderColor,
    pub border: Val,
    pub padding: Val,
    pub margin: Val,
    pub border_color: BorderColor,
    pub font: Handle<Font>,
    pub font_size_xtra_large: f32,
    pub font_size_large: f32,
    pub font_size_regular: f32,
    pub font_size_small: f32,
    pub text_style_small: TextStyle,
    pub text_style_regular: TextStyle,
}

/// Generic style updates on interaction for all buttons
pub fn button_style(
    theme: Res<UiTheme>,
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, Or<(With<Button>, With<ImageButton>)>),
    >,
) {
    for (interaction, mut color) in &mut interaction_q {
        //println!("{interaction:?} {color:?}");
        match *interaction {
            Interaction::Pressed => *color = theme.button_theme.pressed_background_color,
            Interaction::Hovered => *color = theme.button_theme.hovered_background_color,
            Interaction::None => *color = theme.button_theme.normal_background_color,
        }
    }
}

pub fn insert_theme(mut ui_theme: ResMut<UiTheme>, ui_font: Res<UiFont>) {
    let container_style = Style {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        ..default()
    };

    let vertical_spacer = Style {
        height: Val::Px(8.),
        ..default()
    };

    let horizontal_spacer = Style {
        width: Val::Px(8.),
        ..default()
    };

    let row_style = Style {
        flex_direction: FlexDirection::Row,
        ..default()
    };

    let col_style = Style {
        flex_direction: FlexDirection::Column,
        ..default()
    };

    let border = Val::Px(3.);
    let padding = Val::Px(4.);
    let margin = Val::Px(4.);

    let border_color = BorderColor(Color::rgb_u8(50, 50, 55));

    let frame_col_style = Style {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect::all(margin),
        border: UiRect::all(border),
        ..default()
    };

    let frame_row_style = Style {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        align_self: AlignSelf::FlexStart,
        justify_content: JustifyContent::Center,
        margin: UiRect::all(margin),
        border: UiRect::all(border),
        ..default()
    };

    let text_color_dark = Color::rgb(0.85, 0.85, 0.80);
    let text_color_light = Color::rgb(0.2, 0.2, 0.25);

    let font = ui_font.prime.clone_weak();
    let font_size_xtra_large = 32.;
    let font_size_large = 20.;
    let font_size_regular = 18.;
    let font_size_small = 16.;

    let text_color = text_color_dark;

    let text_style_small = TextStyle {
        font: font.clone_weak(),
        font_size: font_size_small,
        color: text_color,
    };

    let text_style_regular = TextStyle {
        font: font.clone_weak(),
        font_size: font_size_regular,
        color: text_color,
    };

    *ui_theme = UiTheme {
        text_color_dark,
        text_color_light,
        container_absolute_max: container_style,
        frame_row_style,
        frame_col_style,
        row_style,
        col_style,
        vertical_spacer,
        horizontal_spacer,
        button_theme: ButtonTheme {
            hovered_border_color: Color::rgb(0.4, 0.14, 0.15).into(),
            hovered_background_color: Color::rgb(0.5, 0.45, 0.45).into(),
            pressed_background_color: Color::rgb(0.4, 0.15, 0.15).into(),
            pressed_border_color: Color::rgb(0.4, 0.15, 0.15).into(),
            normal_background_color: Color::rgb(0.3, 0.10, 0.10).into(),
            normal_border_color: Color::rgb(1. - 0.25, 1. - 0.25, 1. - 0.25).into(),
            style: Style {
                justify_content: JustifyContent::Center,
                ..default()
            },
        },
        background_color: Color::rgb_u8(15, 10, 10).into(),
        menu_background_color: Color::rgb_u8(25, 20, 20).into(),
        frame_background_color: Color::rgb_u8(25, 25, 30).into(),
        frame_border_color: Color::rgb_u8(40, 20, 20).into(),
        border_color,
        border,
        padding,
        margin,
        font,
        font_size_xtra_large,
        font_size_large,
        font_size_regular,
        font_size_small,
        text_style_small,
        text_style_regular,
    };
}
