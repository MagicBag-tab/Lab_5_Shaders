"""Exporta cada diseño .blend a OBJ + MTL."""
import bpy
import os

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
BLENDS = os.path.join(ROOT, "assets")
OUT = os.path.join(ROOT, "assets", "exports")
os.makedirs(OUT, exist_ok=True)

COLORS = {
    "solaria": (1.0, 0.35, 0.18, 1.0),
    "lunaria": (0.55, 0.35, 0.85, 1.0),
    "aurelia": (0.65, 0.45, 0.9, 1.0),
    "ocularis": (0.35, 0.9, 0.75, 1.0),
    "labyrinthia": (0.9, 0.45, 0.75, 1.0),
    "vortex": (0.65, 0.35, 0.9, 1.0),
}

for name in ("solaria", "lunaria", "aurelia", "ocularis", "labyrinthia", "vortex"):
    bpy.ops.wm.open_mainfile(filepath=os.path.join(BLENDS, name + ".blend"))
    color = COLORS[name]
    for obj in bpy.context.scene.objects:
        if obj.type != "MESH":
            continue
        mat = bpy.data.materials.new(obj.name + "_material")
        mat.diffuse_color = color
        obj.data.materials.clear()
        obj.data.materials.append(mat)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.wm.obj_export(
        filepath=os.path.join(OUT, name + ".obj"),
        export_materials=True,
        export_object_groups=True,
        export_material_groups=True,
        export_triangulated_mesh=True,
    )
    print("Exportado", name)
