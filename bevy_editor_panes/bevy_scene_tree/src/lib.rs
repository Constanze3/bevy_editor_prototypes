//! An interactive, collapsible tree view for hierarchical ECS data in Bevy.

use bevy::{app::Plugin, color::palettes::tailwind, prelude::*};
use bevy_editor_core::{SceneMarker, SelectedEntity};
use bevy_i_cant_believe_its_not_bsn::{on, template, Template, TemplateEntityCommandsExt};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};

/// Plugin for the editor scene tree pane.
pub struct SceneTreeEditorPlugin;

impl Plugin for SceneTreeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Scene Tree", setup_pane);
        app.add_systems(PostUpdate, update_scene_tree);
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
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                column_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..Default::default()
            },
            BackgroundColor(tailwind::NEUTRAL_600.into()),
        ))
        .observe(deselect_entity);
}

#[derive(Component)]
struct SceneTreeNode(Entity);

fn update_scene_tree(
    scene_tree_editor_query: Query<Entity, With<SceneTreeEditorRoot>>,
    scene_query: Query<Entity, With<SceneMarker>>,
    spawn_nodes_query: Query<(Option<&Name>, Option<&Children>)>,
    selected_entity: Res<SelectedEntity>,
    mut commands: Commands,
) {
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
            .build_children(screen_trees);
    }
}

fn scene_tree_nodes(
    entity: Entity,
    query: &Query<(Option<&Name>, Option<&Children>)>,
    selected_entity: &SelectedEntity,
    commands: &mut Commands,
) -> Template {
    let (name, children) = query.get(entity).unwrap();

    if let Some(children) = children {
        children
            .into_iter()
            .map(|child| scene_tree_nodes(*child, query, selected_entity, commands))
            .flatten()
            .collect()
    } else {
        let name: String = name.map(Into::into).unwrap_or("<No Name>".into());

        return template! {
                   {entity}: (
                       SceneTreeNode(entity),
                        Node {
                            padding: UiRect::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(if selected_entity.0 == Some(entity) {
                            tailwind::NEUTRAL_700.into()
                        } else {
                            Color::NONE
                        }),
                   ) => [
                        // TODO this observer doesn't seem to work
                        on(select_entity);
                        (
                            Text(name), TextFont::from_font_size(11.0)
                        );
                   ];
        };
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
