# Exports the 3D monogram to static/monogram.glb for the front page.
#
#   blender -b 3d_mongram.blend --python scripts/export_monogram.py
#
# The model is two disconnected halves, like the SVG monogram. They are split
# into nodes named ns-a (upper) and ns-b (lower) so static/monogram.js can
# colour them the way style.css colours .ns-a and .ns-b.
import bpy

mesh = next(o for o in bpy.data.objects if o.type == "MESH")
for o in bpy.data.objects:
    o.select_set(o == mesh)
bpy.context.view_layer.objects.active = mesh
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

bpy.ops.object.mode_set(mode="EDIT")
bpy.ops.mesh.select_all(action="SELECT")
bpy.ops.mesh.separate(type="LOOSE")
bpy.ops.object.mode_set(mode="OBJECT")

halves = list(bpy.context.selected_objects)
assert len(halves) == 2, f"expected two halves, found {len(halves)}"

def height(o):
    return sum(v.co.z for v in o.data.vertices) / len(o.data.vertices)

halves.sort(key=height, reverse=True)
for name, o in zip(("ns-a", "ns-b"), halves):
    o.name = name

bpy.ops.export_scene.gltf(
    filepath="static/monogram.glb",
    use_selection=True,
    export_yup=True,
    export_materials="NONE",
    export_normals=False,  # the viewer shades flat from screen-space derivatives
    export_texcoords=False,
    export_vertex_color="NONE",
    export_all_vertex_colors=False,
)
