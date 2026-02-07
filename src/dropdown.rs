use bevy::prelude::*;

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_dropdown_interactions);
    }
}

#[derive(Component)]
pub struct Dropdown;

#[derive(Component)]
pub struct DropdownButton;

#[derive(Component)]
pub struct DropdownList;

#[derive(Component)]
pub struct DropdownItem(String);

#[derive(Event)]
pub struct DropdownChanged {
    //pub entity: Entity,
    pub selected_item: String,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn spawn_dropdown<'a>(
    commands: &'a mut Commands,
    position: Vec2,
    size: Vec2,
    label: impl Into<String>,
    options: impl AsRef<[String]>,
) -> EntityCommands<'a> {
    let mut entity_commands = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(position.x),
            top: Val::Px(position.y),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        Dropdown,
    ));

    entity_commands.with_children(|parent| {
        // button
        parent
            .spawn((
                Button,
                Node {
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::BLACK),
                BackgroundColor(NORMAL_BUTTON),
                DropdownButton,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(label.into()),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

        // list
        parent
            .spawn((
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(size.x),
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
                BorderColor::all(Color::BLACK),
                BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                DropdownList,
            ))
            .with_children(|parent| {
                for option in options.as_ref() {
                    parent
                        .spawn((
                            Button,
                            Node {
                                height: Val::Px(size.y - 10.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            DropdownItem(option.to_string()),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(option),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                }
            });
    });

    entity_commands
}

fn handle_dropdown_interactions(
    mut commands: Commands,
    dropdown_children_query: Query<&Children, With<Dropdown>>,
    mut dropdown_list_query: Query<&mut Node, With<DropdownList>>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DropdownButton>),
    >,
    button_children_query: Query<&Children, With<DropdownButton>>,
    mut item_query: Query<
        (&Interaction, &mut BackgroundColor, &DropdownItem),
        (Changed<Interaction>, Without<DropdownButton>),
    >,
    mut text_query: Query<&mut Text>,
) {
    // main button click
    for (interaction, mut color) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                // Toggle list visibility (Toggle ALL dropdowns for simplicity in this example)
                for dropdown_children in &dropdown_children_query {
                    for child in dropdown_children {
                        if let Ok(mut list_node) = dropdown_list_query.get_mut(*child) {
                            list_node.display = match list_node.display {
                                Display::None => Display::Flex,
                                _ => Display::None,
                            };
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }

    // Handle item clicks
    for (interaction, mut color, item) in &mut item_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                commands.trigger(DropdownChanged {
                    selected_item: item.0.clone(),
                });

                // Close list and update button text (Update ALL dropdowns for simplicity)
                for dropdown_children in &dropdown_children_query {
                    for child in dropdown_children {
                        // Close list
                        if let Ok(mut list_node) = dropdown_list_query.get_mut(*child) {
                            list_node.display = Display::None;
                        }
                        // Update button text
                        if let Ok(text_children) = button_children_query.get(*child) {
                            for text_child in text_children {
                                if let Ok(mut text) = text_query.get_mut(*text_child) {
                                    **text = item.0.clone();
                                }
                            }
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = Color::NONE.into();
            }
        }
    }
}
