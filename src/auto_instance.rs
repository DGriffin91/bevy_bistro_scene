use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use bevy::ecs::component::Component;
use bevy::math::*;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

pub struct AutoInstancePlugin;
impl Plugin for AutoInstancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (apply_auto_instance_recursive, consolidate_mesh_instances),
        );
    }
}

#[derive(Default)]
pub struct AutoInstanceMaterialPlugin<M: Material + MaterialHash>(pub PhantomData<M>);
impl<M: Material + MaterialHash> Plugin for AutoInstanceMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, consolidate_material_instances::<M>);
    }
}

pub fn all_children<F: FnMut(Entity)>(
    children: &Children,
    children_query: &Query<&Children>,
    closure: &mut F,
) {
    for child in children {
        if let Ok(children) = children_query.get(*child) {
            all_children(children, children_query, closure);
        }
        closure(*child);
    }
}

#[derive(Component)]
pub struct AutoInstanceMaterial;

#[derive(Component)]
pub struct AutoInstanceMaterialRecursive;

#[derive(Component)]
pub struct AutoInstanceMesh;

#[derive(Component)]
pub struct AutoInstanceMeshRecursive;

pub fn apply_auto_instance_recursive(
    mut commands: Commands,
    material_entities: Query<Entity, With<AutoInstanceMaterialRecursive>>,
    mesh_entities: Query<Entity, With<AutoInstanceMeshRecursive>>,
    children_query: Query<&Children>,
) {
    for entity in &material_entities {
        if let Ok(children) = children_query.get(entity) {
            all_children(children, &children_query, &mut |entity| {
                commands.entity(entity).insert(AutoInstanceMaterial);
            });
            commands
                .entity(entity)
                .remove::<AutoInstanceMaterialRecursive>();
        }
    }
    for entity in &mesh_entities {
        if let Ok(children) = children_query.get(entity) {
            all_children(children, &children_query, &mut |entity| {
                commands.entity(entity).insert(AutoInstanceMesh);
            });
            commands
                .entity(entity)
                .remove::<AutoInstanceMeshRecursive>();
        }
    }
}

pub fn consolidate_material_instances<M: Material + MaterialHash>(
    mut commands: Commands,
    materials: ResMut<Assets<M>>,
    entities: Query<(Entity, &Handle<M>), With<AutoInstanceMaterial>>,
    mut instances: Local<HashMap<u64, Handle<M>>>,
    mut handles: Local<HashSet<Handle<M>>>,
    mut count: Local<u32>,
) {
    let mut print = false;
    for (entity, mat_h) in &entities {
        if let Some(mat) = materials.get(mat_h) {
            if !handles.contains(mat_h) {
                print = true;
                let h = mat.generate_hash();
                if let Some(instance_h) = instances.get(&h) {
                    commands.entity(entity).insert(instance_h.clone());
                    *count += 1;
                } else {
                    instances.insert(h, mat_h.clone());
                    handles.insert(mat_h.clone());
                }
            }
            commands.entity(entity).remove::<AutoInstanceMaterial>();
        }
    }
    if print {
        println!("Duplicate material instances found: {}", *count);
        println!("Total unique materials: {}", instances.len());
    }
}

// Implement the MaterialHash trait for any material
pub trait MaterialHash {
    fn generate_hash(&self) -> u64;
}

impl MaterialHash for StandardMaterial {
    fn generate_hash(&self) -> u64 {
        let state = &mut DefaultHasher::new();
        hash_color(&self.base_color, state);
        self.base_color_texture.hash(state);
        hash_color(&self.emissive, state);
        self.emissive_texture.hash(state);
        self.perceptual_roughness.to_bits().hash(state);
        self.metallic.to_bits().hash(state);
        self.metallic_roughness_texture.hash(state);
        self.reflectance.to_bits().hash(state);
        self.diffuse_transmission.to_bits().hash(state);
        self.specular_transmission.to_bits().hash(state);
        self.thickness.to_bits().hash(state);
        self.ior.to_bits().hash(state);
        self.attenuation_distance.to_bits().hash(state);
        hash_color(&self.attenuation_color, state);
        self.normal_map_texture.hash(state);
        self.flip_normal_map_y.hash(state);
        self.occlusion_texture.hash(state);
        self.double_sided.hash(state);
        self.cull_mode.hash(state);
        self.unlit.hash(state);
        self.fog_enabled.hash(state);
        match self.alpha_mode {
            AlphaMode::Opaque => 798573452.hash(state),
            AlphaMode::Mask(m) => m.to_bits().hash(state),
            AlphaMode::Blend => 1345634567.hash(state),
            AlphaMode::Premultiplied => 297897363.hash(state),
            AlphaMode::Add => 36345667.hash(state),
            AlphaMode::Multiply => 48967896.hash(state),
            #[cfg(feature = "bevy_main")]
            AlphaMode::AlphaToCoverage => 20935847.hash(state),
        }
        self.depth_bias.to_bits().hash(state);
        self.depth_map.hash(state);
        self.parallax_depth_scale.to_bits().hash(state);
        self.parallax_mapping_method
            .reflect_hash()
            .unwrap()
            .hash(state);
        self.max_parallax_layer_count.to_bits().hash(state);
        self.opaque_render_method
            .reflect_hash()
            .unwrap()
            .hash(state);
        self.deferred_lighting_pass_id.hash(state);
        self.lightmap_exposure.to_bits().hash(state);
        state.finish()
    }
}

pub fn hash_color<H: Hasher>(color: &Color, state: &mut H) {
    #[cfg(feature = "bevy_main")]
    {
        let color = color.linear();
        color.red.to_bits().hash(state);
        color.green.to_bits().hash(state);
        color.blue.to_bits().hash(state);
        color.alpha().to_bits().hash(state);
    }
    #[cfg(not(feature = "bevy_main"))]
    {
        color.r().to_bits().hash(state);
        color.g().to_bits().hash(state);
        color.b().to_bits().hash(state);
        color.a().to_bits().hash(state);
    }
}

pub fn consolidate_mesh_instances(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    mut entities: Query<(Entity, &Handle<Mesh>), With<AutoInstanceMesh>>,
    mut instances: Local<HashMap<u64, Handle<Mesh>>>,
    mut handles: Local<HashSet<Handle<Mesh>>>,
    mut count: Local<u32>,
) {
    let mut print = false;
    for (entity, mesh_h) in &mut entities {
        if let Some(mesh) = meshes.get(mesh_h) {
            if !handles.contains(mesh_h) {
                print = true;
                let state = &mut DefaultHasher::new();

                mesh.attributes().count().hash(state);
                for (id, attribute) in mesh.attributes() {
                    id.hash(state);
                    attribute.get_bytes().hash(state);
                }
                let h = state.finish();

                if let Some(instance_h) = instances.get(&h) {
                    commands.entity(entity).insert(instance_h.clone());
                    *count += 1;
                } else {
                    instances.insert(h, mesh_h.clone());
                    handles.insert(mesh_h.clone());
                }
            }
            commands.entity(entity).remove::<AutoInstanceMesh>();
        }
    }
    if print {
        println!("Duplicate mesh instances found: {}", *count);
        println!("Total unique meshes: {}", instances.len());
    }
}
