# Bevy Bistro Example

Download scene from https://developer.nvidia.com/orca/amazon-lumberyard-bistro

Reexport BistroExterior.fbx and BistroInterior_Wine.fbx as GLTF files (in .gltf + .bin + textures format). Move the files into the respective bistro_exterior and bistro_interior_wine folders.

To optionally convert the textures to KTX2 use: `cargo run -- --convert`. You need [kram](https://github.com/alecazam/kram) in your path to do this. It will convert all the textures to BC7 KTX2 zstd 0 using `available_parallelism()` and update the gltf files to use the KTX2 textures.