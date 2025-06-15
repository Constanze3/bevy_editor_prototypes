//! An interactive, collapsible tree view for hierarchical ECS data in Bevy.

use bevy::{app::Plugin, color::palettes::tailwind, prelude::*};
use bevy_editor_core::{SceneRootMarker, SelectedEntity};
use bevy_i_cant_believe_its_not_bsn::{on, template, Template, TemplateEntityCommandsExt};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};

/// Plugin for the editor scene tree pane.
pub struct SceneTreeEditorPlugin;

impl Plugin for SceneTreeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Scene Tree", setup_pane);
        app.add_systems(PostUpdate, build_scene_tree);
        app.add_systems(PostUpdate, update_expansion_tile);
    }
}

/// Root UI node of the scene tree.
#[derive(Component)]
struct SceneTreeEditorRoot;

fn setup_pane(pane: In<PaneStructure>, mut commands: Commands) {
    commands
        .entity(pane.content)
        .insert((
            SceneTreeEditorRoot,
            Node::default(),
            BackgroundColor(tailwind::NEUTRAL_600.into()),
        ))
        .observe(deselect_entity);
}

#[derive(Component)]
struct SceneTreeNode(Entity);

fn build_scene_tree(
    keyboard: Res<ButtonInput<KeyCode>>,
    scene_tree_editor_query: Query<Entity, With<SceneTreeEditorRoot>>,
    scene_query: Query<Entity, With<SceneRootMarker>>,
    spawn_nodes_query: Query<(Option<&Name>, Option<&Children>)>,
    selected_entity: Res<SelectedEntity>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::KeyT) {
        if keyboard.just_pressed(KeyCode::KeyA) {
            let scene = scene_query.single().unwrap();
            let child = spawn_nodes_query
                .get(scene)
                .unwrap()
                .1
                .unwrap()
                .iter()
                .next()
                .unwrap();

            commands.entity(child).with_child((
                Transform::default(),
                Visibility::default(),
                Name::new("Dummy"),
            ));
        }

        return;
    }

    if scene_tree_editor_query.is_empty() {
        return;
    }

    for scene_tree_editor in scene_tree_editor_query.iter() {
        let screen_trees: Template = scene_query
            .iter()
            .map(|root| scene_tree_nodes(root, &spawn_nodes_query, &selected_entity, &mut commands))
            .flatten()
            .collect();

        commands
            .entity(scene_tree_editor)
            .build_nonexistent_children(screen_trees);
    }
}

fn scene_tree_nodes(
    entity: Entity,
    query: &Query<(Option<&Name>, Option<&Children>)>,
    selected_entity: &SelectedEntity,
    commands: &mut Commands,
) -> Template {
    let (name, children) = query.get(entity).unwrap();

    let children_template = {
        if let Some(children) = children {
            children
                .into_iter()
                .map(|child| scene_tree_nodes(*child, query, selected_entity, commands))
                .flatten()
                .collect()
        } else {
            template! {}
        }
    };

    let name: String = name.map(Into::into).unwrap_or("<No Name>".into());

    if 0 < children_template.len() {
        return template! {
            @{expansion_tile(name, children_template, entity, selected_entity.0)};
        };
    } else {
        return template! {
            (Node {
                padding: UiRect::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                ..Default::default()
            }) => [
                on(select_entity);
                (Text(name), TextFont::from_font_size(11.0), Pickable::IGNORE);
            ];
        };
    }
}

#[derive(Component)]
#[require(ExpansionTileFolded)]
struct ExpansionTile;

#[derive(Component, Default)]
struct ExpansionTileFolded(bool);

#[derive(Component)]
struct ExpansionTileChildren;

fn expansion_tile(
    title: String,
    children: Template,
    entity: Entity,
    selected_entity: Option<Entity>,
) -> Template {
    template! {
        (
            ExpansionTile,
            Node {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            SceneTreeNode(entity),
            BackgroundColor(if selected_entity == Some(entity) {
                tailwind::NEUTRAL_700.into()
            } else {
                Color::NONE
            }),

        ) => [
            (Node {
                padding: UiRect::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                ..Default::default()
            }) => [
                on(toggle_expansion_tile);
                // on(select_entity);
                (Text(title), TextFont::from_font_size(11.0), Pickable::IGNORE);
            ];
            (
                // this depends
                ExpansionTileChildren,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    column_gap: Val::Px(2.0),
                    left: Val::Px(5.0),
                    ..default()
                }
            ) => [
                @{ children };
            ];
        ];

    }
}

fn toggle_expansion_tile(
    trigger: Trigger<Pointer<Click>>,
    mut parent_query: Query<&ChildOf>,
    mut folded_query: Query<&mut ExpansionTileFolded>,
) {
    let parent = parent_query.get_mut(trigger.target).unwrap().0;
    let mut folded = folded_query.get_mut(parent).unwrap();

    folded.0 = !folded.0;
}

fn update_expansion_tile(
    query: Query<(&ExpansionTileFolded, &Children), Changed<ExpansionTileFolded>>,
    etc_query: Query<(), With<ExpansionTileChildren>>,
    mut node_query: Query<&mut Node>,
) {
    for (folded, folded_children) in query.iter() {
        // Find the first child with the ExpansionTileChildren component.
        let children_root = folded_children
            .iter()
            .filter(|child| etc_query.get(*child).is_ok())
            .next();

        let Some(children_root) = children_root else {
            panic!("Expansion tile should have ExpansionTileChildren as a child");
        };

        let display = match folded.0 {
            true => Display::None,
            false => Display::Flex,
        };

        let mut node = node_query.get_mut(children_root).unwrap();
        node.display = display;
    }
}

fn deselect_entity(
    mut trigger: Trigger<Pointer<Click>>,
    mut selected_entity: ResMut<SelectedEntity>,
) {
    selected_entity.0 = None;
    trigger.propagate(false);
}

fn select_entity(
    mut trigger: Trigger<Pointer<Click>>,
    node_query: Query<&SceneTreeNode>,
    mut selected_entity: ResMut<SelectedEntity>,
) {
    let Ok(node) = node_query.get(trigger.target) else {
        return;
    };
    let entity = node.0;

    if selected_entity.0 == Some(entity) {
        selected_entity.0 = None;
    } else {
        selected_entity.0 = Some(entity);
    }

    trigger.propagate(false);
}
