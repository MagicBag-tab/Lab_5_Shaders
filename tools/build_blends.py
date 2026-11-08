"""Importa los modelos generados y crea un .blend por diseño.

Uso desde la raíz del proyecto:
    blender --background --python tools/build_blends.py
"""
import bpy
import os
from mathutils import Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
ASSETS = os.path.join(ROOT, "assets", "crystal_source")
OUT = os.path.join(ASSETS, "blends")
OUT = os.path.join(ROOT, "assets", "blends")
os.makedirs(OUT, exist_ok=True)

DESIGNS = {
    "solaria": [("solaria_esfera.obj", (0, 0, 0)), ("solaria_corona.obj", (0, 0, 0))],
    "lunaria": [("lunaria_planeta.obj", (0, 0, 0)), ("lunaria_luna.obj", (1.6, 0.25, 0))],
    "aurelia": [("aurelia_planeta.obj", (0, 0, 0)), ("aurelia_anillo_exterior.obj", (0, 0, 0)), ("aurelia_anillo_interior.obj", (0, 0, 0))],
    "ocularis": [("ocularis_cuerpo.obj", (0, 0, 0)), ("ocularis_iris.obj", (0, 0, .5)), ("ocularis_arco_superior.obj", (0, .65, .25)), ("ocularis_arco_inferior.obj", (0, -.65, .25))],
    "labyrinthia": [("labyrinthia_creciente.obj", (0, 0, 0)), ("labyrinthia_fragmento_1.obj", (1.0, .4, .2)), ("labyrinthia_fragmento_2.obj", (-.9, .2, .1)), ("labyrinthia_fragmento_3.obj", (.7, -.6, -.1))],
    "vortex": [("vortex_cascaron_superior.obj", (0, 0, 0)), ("vortex_cascaron_inferior.obj", (0, 0, 0)), ("vortex_nucleo.obj", (0, 0, 0)), ("vortex_anillo.obj", (0, 0, 0))],
}

def clear_scene():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)

def import_obj(path):
    before = set(bpy.context.scene.objects)
    if hasattr(bpy.ops.wm, "obj_import"):
        bpy.ops.wm.obj_import(filepath=path)
    else:
        bpy.ops.import_scene.obj(filepath=path)
    created = set(bpy.context.scene.objects) - before
    return list(created)

def build(name, pieces):
    clear_scene()
    collection = bpy.data.collections.new(name.upper())
    bpy.context.scene.collection.children.link(collection)
    for filename, location in pieces:
        objects = import_obj(os.path.join(ASSETS, filename))
        for obj in objects:
            obj.location = Vector(location)
            obj.name = os.path.splitext(filename)[0]
            for old in list(obj.users_collection):
                old.objects.unlink(obj)
            collection.objects.link(obj)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(OUT, name + ".blend"))

for design, pieces in DESIGNS.items():
    build(design, pieces)

print("Archivos .blend creados en", OUT)
