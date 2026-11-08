"""Restore the scene-space placement supplied by the original OBJ archive."""
import os
import bpy

ASSETS = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'assets'))
for name, expected in [('ocularis', 12696), ('labyrinthia', 7416), ('lunaria', 5488)]:
    bpy.ops.wm.open_mainfile(filepath=os.path.join(ASSETS, name + '.blend'))
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == 'MESH']
    for obj in meshes:
        obj.location = (0, 0, 0)
    count = sum(len(poly.vertices) - 2 for obj in meshes for poly in obj.data.polygons)
    assert count == expected, (name, count, expected)
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(ASSETS, name + '.blend'))
    bpy.ops.wm.obj_export(
        filepath=os.path.join(ASSETS, name + '.obj'),
        export_materials=True,
        export_object_groups=True,
        export_material_groups=True,
        export_triangulated_mesh=True,
    )
    print(name, count, 'triangles; original placement restored')
