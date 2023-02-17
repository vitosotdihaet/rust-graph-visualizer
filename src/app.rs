pub use bevy::{
    prelude::*,
    window::close_on_esc,
    render::{
        render_resource::SamplerDescriptor,
        texture::ImageSampler
    },
};

use bevy::sprite::MaterialMesh2dBundle;
use std::{
    iter::zip,
    path::Path,
};

use crate::graph::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum GraphState {
    Graph,
    Algorithm,
}

#[derive(Resource)]
pub struct Resources {
    font: Handle<Font>,
}

#[derive(Component)]
pub struct StartingText;


#[derive(Component)]
pub struct BgVertex;


const FONT_NAME: &str = "FOTNewRodin Pro B.otf";

const FONT_SIZE: f32 = 60.0;
const INIT_TEXT_FONT_SIZE: f32 = 40.0;

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const INIT_TEXT_COLOR: Color = Color::rgb(0.4, 0.4, 0.4);

const COLOR_FG_NODE: Color = Color::rgb(0.5, 0.5, 0.5);
const COLOR_BG_NODE: Color = Color::rgb(0.2, 0.2, 0.2);
const COLOR_HOVERED_NODE: Color = Color::rgb(0.65, 0.65, 0.65);
const COLOR_PRESSED_NODE: Color = Color::rgb(0.3, 0.3, 0.3);

pub fn startup(
    a: Res<AssetServer>,
    mut c: Commands,
) {
    c.spawn(Camera2dBundle::default());

    c.insert_resource(Resources { font: a.load(Path::new("fonts").join(FONT_NAME)) });
}

pub fn init(
    r: Res<Resources>,
    mut c: Commands,
) {
    c.spawn(Text2dBundle {
        text: Text {
            sections: vec![TextSection {
                value: "To create a new vertex press RMB".to_owned(),
                style: TextStyle {
                    font: r.font.clone(),
                    font_size: INIT_TEXT_FONT_SIZE,
                    color: INIT_TEXT_COLOR,
                },
            }],
            alignment: TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        },
        ..Default::default()
    })
    .insert(StartingText);
}

pub fn app(
    r: Res<Resources>,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut c: Commands,
    mut g: ResMut<Graph>,
    mut state: ResMut<State<GraphState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cursor_moved: EventReader<CursorMoved>,
    mut cursor_position: Local<Vec2>,
    mut text_query: Query<Entity, With<StartingText>>,
    mut vertex_bg_query: Query<&mut Transform, (With<Vertex>, With<BgVertex>)>,
    mut vertex_fg_query: Query<&mut Transform, (With<Vertex>, Without<BgVertex>)>,
    mut vertex_interaction_query: Query<&Interaction, (Changed<Interaction>, With<Vertex>)>,
) {
    if let Some(moved_cursor) = cursor_moved.iter().last() {
        *cursor_position = moved_cursor.position;    
    }

    let left_click = mouse_button_input.just_released(MouseButton::Left);
    let right_click = mouse_button_input.just_released(MouseButton::Right);

    let window = windows.get_primary().unwrap();
    let (w, h) = ((*window).width(), (*window).height());

    // create new vertex
    if right_click {
        for e in &mut text_query { c.entity(e).despawn(); }

        let new_id = (*g).len();
        let (cx, cy) = ((*cursor_position).x - w/2., (*cursor_position).y - h/2.);
        let vertex = Vertex { id: new_id, coords: Vec2::new(cx, cy), ..Default::default() };

        (*g).add_vertex(vertex.clone());

        c.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            material: materials.add(ColorMaterial::from(COLOR_BG_NODE)),
            transform: Transform::from_translation(Vec3::new(cx, cy, 0.)),
            ..default()
        })
        .insert(vertex.clone())
        .insert(BgVertex);

        c.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(40.).into()).into(),
            material: materials.add(ColorMaterial::from(COLOR_FG_NODE)),
            transform: Transform::from_translation(Vec3::new(cx, cy, 1.)),
            ..default()
        })
        .insert(vertex);
    }

    for (mut v1, fgt1) in zip((*g).verticies.iter_mut(), &vertex_fg_query) {
        v1.coords = Vec2::new(fgt1.translation.x, fgt1.translation.y);
    }

    for (mut v1, (mut fgt1, mut bgt1)) in zip((*g).verticies.clone().iter_mut(), zip(&mut vertex_fg_query, &mut vertex_bg_query)) {
        v1.coords = Vec2::new(fgt1.translation.x, fgt1.translation.y);
        for v2 in (*g).verticies.clone() {
            if *v1 == v2 { continue; }
            let f = v1.relate(&v2);
            v1.add_acc(f);
        }
        v1.update();

        let (x, y) = (v1.coords.x, v1.coords.y); // bro i can't even unwrap Vec2 to tuple, literally 1984
        *bgt1 = Transform { translation: Vec3 { x, y, z: 0. }, ..Default::default() };
        *fgt1 = Transform { translation: Vec3 { x, y, z: 1. }, ..Default::default() };
    }

}
