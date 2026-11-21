"""Side-by-side comparison: a stylized mannequin and a MakeHuman body on the SAME rig, pose, camera
and light. Both are dressed alike (white uniform, navy cap, brown glove, black shoes, white ball).

usage: blender -b --python compare.py -- OUTDIR POSE [SAMPLES] [VIEWS]
  POSE: ffc (front-foot contact) or mer (maximum shoulder external rotation)
  VIEWS: comma list of side, cf, close (default side,cf)

World frame: the pitcher throws toward +X (home plate 18.44 m away); his chest faces -Y (the
third-base side) at the start of the stride; +Z is up. A right-handed pitcher, 1.90 m.
Pose numbers (approximate, from the pitching literature): stride, ankle to ankle, 0.80 of the
height (Dowling et al. 2022: 77-81 %); at foot contact the pelvis is about 35 deg open and the
shoulders still closed (hip-shoulder separation about 40 deg), the throwing upper arm about 90 deg
abducted and the forearm about 50 deg externally rotated; at maximum external rotation the
forearm lies back about 170 deg (Thompson et al. 2018: 166-178 deg)."""
import math
import os
import sys

import bpy
from mathutils import Matrix, Quaternion, Vector

from bl_ext.user_default.mpfb.services.humanservice import HumanService
from bl_ext.user_default.mpfb.services.locationservice import LocationService
from bl_ext.user_default.mpfb.services.targetservice import TargetService

ARGS = sys.argv[sys.argv.index("--") + 1:]
OUT = ARGS[0]
POSE = ARGS[1] if len(ARGS) > 1 else "ffc"
SAMPLES = int(ARGS[2]) if len(ARGS) > 2 else 64
VIEWS = (ARGS[3] if len(ARGS) > 3 else "side,cf").split(",")
os.makedirs(OUT, exist_ok=True)
HOME_X = 18.44


def upd():
    bpy.context.view_layer.update()


def mat(name, color, rough=0.6, sss=0.0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    p = m.node_tree.nodes["Principled BSDF"]
    p.inputs["Base Color"].default_value = (*color, 1.0)
    p.inputs["Roughness"].default_value = rough
    if sss:
        p.inputs["Subsurface Weight"].default_value = sss
        p.inputs["Subsurface Radius"].default_value = (0.9, 0.35, 0.2)
        p.inputs["Subsurface Scale"].default_value = 0.01
    return m


M = {}


def make_materials():
    M.update({
    "jersey": mat("jersey", (0.80, 0.80, 0.78), 0.85),
    "skin": mat("skin", (0.66, 0.45, 0.33), 0.5, sss=0.25),
    "cap": mat("cap", (0.03, 0.06, 0.18), 0.7),
    "glove": mat("glove", (0.30, 0.15, 0.06), 0.45),
    "shoe": mat("shoe", (0.02, 0.02, 0.02), 0.35),
    "ball": mat("ball", (0.92, 0.91, 0.88), 0.55),
    "grass": mat("grass", (0.045, 0.15, 0.03), 0.9),
    "dirt": mat("dirt", (0.28, 0.15, 0.075), 0.95),
    "white": mat("chalk", (0.95, 0.95, 0.95), 0.6),
    })
    for m in M.values():
        m.use_fake_user = True


# ---------------------------------------------------------------------------------------
# The human (MakeHuman via MPFB) and its rig

def build_human():
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o, do_unlink=True)
    m = TargetService.get_default_macro_info_dict()
    m.update(gender=1.0, age=0.5, muscle=0.85, weight=0.7, proportions=0.8, height=0.65)
    m["race"] = {"asian": 0.5, "caucasian": 0.3, "african": 0.2}
    body = HumanService.create_human(macro_detail_dict=m)
    height = human_height(body)
    rig = HumanService.add_builtin_rig(body, "game_engine")
    data = LocationService.get_user_data()
    HumanService.set_character_skin(os.path.join(data, "skins/young_asian_male/young_asian_male.mhmat"), body, skin_type="ENHANCED_SSS")
    for kind, rel, mt in [("Eyes", "eyes/high-poly/high-poly.mhclo", "PROCEDURAL_EYES"),
                          ("Eyebrows", "eyebrows/eyebrow001/eyebrow001.mhclo", "MAKESKIN"),
                          ("Eyelashes", "eyelashes/eyelashes01/eyelashes01.mhclo", "MAKESKIN"),
                          ("Clothes", "clothes/male_casualsuit06/male_casualsuit06.mhclo", "MAKESKIN"),
                          ("Clothes", "clothes/shoes03/shoes03.mhclo", "MAKESKIN")]:
        HumanService.add_mhclo_asset(os.path.join(data, rel), body, asset_type=kind, material_type=mt)
    # The casual suit becomes a white uniform: the colour texture is replaced, the normal map
    # (folds) and the ambient-occlusion multiplier stay.
    for o in rig.children:
        if "casualsuit" in o.name:
            for s in o.material_slots:
                nt = s.material.node_tree
                tex = nt.nodes.get("diffuseTexture")
                if tex is None:
                    continue
                rgb = nt.nodes.new("ShaderNodeRGB")
                rgb.outputs[0].default_value = (0.80, 0.80, 0.78, 1.0)
                for link in list(tex.outputs["Color"].links):
                    nt.links.new(rgb.outputs[0], link.to_socket)
    upd()
    return body, rig, height


def human_height(body):
    upd()
    ev = body.evaluated_get(bpy.context.evaluated_depsgraph_get())
    zs = [(ev.matrix_world @ v.co).z for v in ev.data.vertices]
    return max(zs) - min(zs)


# ---------------------------------------------------------------------------------------
# Posing in world space

def pb_world(rig, name):
    pb = rig.pose.bones[name]
    return rig.matrix_world @ pb.head, rig.matrix_world @ pb.tail


def rotate_about_head(rig, name, q_world):
    """Rotate a pose bone about its own head by a world-space rotation."""
    upd()
    pb = rig.pose.bones[name]
    r3 = rig.matrix_world.to_3x3().normalized()
    q_arm = (r3.inverted() @ q_world.to_matrix() @ r3).to_quaternion()
    h = pb.head.copy()
    pb.matrix = Matrix.Translation(h) @ q_arm.to_matrix().to_4x4() @ Matrix.Translation(-h) @ pb.matrix
    upd()


def point(rig, name, direction):
    h, t = pb_world(rig, name)
    q = (t - h).normalized().rotation_difference(Vector(direction).normalized())
    rotate_about_head(rig, name, q)


def twist(rig, name, deg):
    h, t = pb_world(rig, name)
    rotate_about_head(rig, name, Quaternion((t - h).normalized(), math.radians(deg)))


def x_axis(rig, name):
    pb = rig.pose.bones[name]
    return (rig.matrix_world.to_3x3() @ pb.matrix.to_3x3().col[0]).normalized()


def rest_bend_sign(rig, upper, lower):
    b1, b2 = rig.data.bones[upper], rig.data.bones[lower]
    n = (b1.tail_local - b1.head_local).cross(b2.tail_local - b2.head_local)
    return 1.0 if n.dot(b1.matrix_local.col[0].xyz) >= 0 else -1.0


def two_bone(rig, upper, lower, target, pole, sign):
    """Place the end of `lower` at `target` (world), bending towards `pole` (a world direction),
    and twist `upper` so that its hinge (local X) is normal to the bend."""
    s, e0 = pb_world(rig, upper)
    l1 = (e0 - s).length
    e1, w1 = pb_world(rig, lower)
    l2 = (w1 - e1).length
    d = Vector(target) - s
    dist = min(d.length, 0.999 * (l1 + l2))
    u = d.normalized()
    v = Vector(pole) - u * Vector(pole).dot(u)
    v.normalize()
    c = max(-1.0, min(1.0, (l1 * l1 + dist * dist - l2 * l2) / (2 * l1 * dist)))
    t = math.acos(c)
    elbow = s + l1 * (math.cos(t) * u + math.sin(t) * v)
    wrist = s + u * dist
    point(rig, upper, elbow - s)
    n = ((elbow - s).cross(wrist - elbow)).normalized() * sign
    x = x_axis(rig, upper)
    h, tl = pb_world(rig, upper)
    axis = (tl - h).normalized()
    xp = (x - axis * x.dot(axis)).normalized()
    np_ = (n - axis * n.dot(axis)).normalized()
    ang = math.atan2(axis.dot(xp.cross(np_)), xp.dot(np_))
    rotate_about_head(rig, upper, Quaternion(axis, ang))
    point(rig, lower, wrist - elbow)


def set_pelvis(rig, loc, yaw, tilt_fwd=0.0, tilt_side=0.0):
    """Pelvis head to `loc` (world); yaw about +Z (positive opens the pelvis towards home, +X),
    forward tilt towards +X about -Y... applied in world space on top of the rest orientation."""
    upd()
    pb = rig.pose.bones["pelvis"]
    rot = (Matrix.Rotation(math.radians(yaw), 4, "Z") @ Matrix.Rotation(math.radians(tilt_fwd), 4, "Y")
           @ Matrix.Rotation(math.radians(tilt_side), 4, "X"))
    rest = pb.bone.matrix_local
    arm_loc = rig.matrix_world.inverted() @ Vector(loc)
    r3 = rig.matrix_world.to_3x3().normalized()
    rot_arm = r3.inverted().to_4x4() @ rot @ r3.to_4x4()
    m = Matrix.Translation(arm_loc) @ rot_arm @ Matrix.Translation(-rest.translation) @ rest
    pb.matrix = m
    upd()


def pose(rig, kind):
    for pb in rig.pose.bones:
        pb.rotation_mode = "QUATERNION"
        pb.rotation_quaternion = (1, 0, 0, 0)
        pb.location = (0, 0, 0)
    upd()
    # The whole stride: back ankle on the rubber, front ankle 0.80 of the height ahead.
    back_ankle = Vector((0.12, 0.02, 0.10))
    front_ankle = Vector((0.12 + 1.52, -0.05, 0.09))
    if kind == "ffc":
        set_pelvis(rig, (0.82, 0.0, 0.88), yaw=35.0, tilt_fwd=8.0)
        for b in ("spine_01", "spine_02", "spine_03"):
            twist(rig, b, -40.0 / 3)
        # Throwing arm: upper arm out to the side and a little behind the trunk, elbow at 90 deg,
        # forearm about 50 deg up from pointing forward (external rotation at foot contact).
        point(rig, "clavicle_r", Vector((-0.97, 0.15, 0.17)))
        s_r, _ = pb_world(rig, "upperarm_r")
        up_dir = Vector((-0.94, 0.30, 0.10)).normalized()
        fore_dir = Vector((0.0, -0.64, 0.77)).normalized()
        l1 = rig.data.bones["upperarm_r"].length * rig.matrix_world.to_scale()[0]
        l2 = rig.data.bones["lowerarm_r"].length * rig.matrix_world.to_scale()[0]
        target_r = s_r + up_dir * l1 + fore_dir * l2
        two_bone(rig, "upperarm_r", "lowerarm_r", target_r, up_dir, SIGN["arm_r"])
        point(rig, "hand_r", fore_dir + Vector((0.0, 0.25, 0.0)))
        # Glove arm reaching towards home, elbow bent.
        s_l, _ = pb_world(rig, "upperarm_l")
        two_bone(rig, "upperarm_l", "lowerarm_l", s_l + Vector((0.50, -0.12, -0.02)), Vector((0.0, 0.3, 1.0)), SIGN["arm_l"])
        point(rig, "hand_l", Vector((0.8, -0.5, 0.1)))
    else:  # mer: a moment later, chest square to home (+X), forearm laid back about 170 deg
        set_pelvis(rig, (1.02, 0.0, 0.86), yaw=85.0, tilt_fwd=20.0)
        for b in ("spine_01", "spine_02", "spine_03"):
            twist(rig, b, 5.0 / 3)
        # Facing +X, the throwing (right) side is -Y.
        point(rig, "clavicle_r", Vector((-0.2, -0.95, 0.2)))
        s_r, _ = pb_world(rig, "upperarm_r")
        # Upper arm abducted 90 deg, a little behind the chest; external rotation 170 deg: the
        # forearm lies back towards second base (-X) and a little up.
        up_dir = Vector((-0.25, -0.96, 0.12)).normalized()
        fore_dir = Vector((-math.cos(math.radians(10)), 0.0, math.sin(math.radians(10)))).normalized()
        l1 = rig.data.bones["upperarm_r"].length * rig.matrix_world.to_scale()[0]
        l2 = rig.data.bones["lowerarm_r"].length * rig.matrix_world.to_scale()[0]
        target_r = s_r + up_dir * l1 + fore_dir * l2
        two_bone(rig, "upperarm_r", "lowerarm_r", target_r, up_dir, SIGN["arm_r"])
        point(rig, "hand_r", fore_dir + Vector((0.0, 0.0, 0.2)))
        # Glove arm pulled in to the chest.
        s_l, _ = pb_world(rig, "upperarm_l")
        two_bone(rig, "upperarm_l", "lowerarm_l", s_l + Vector((0.28, -0.12, -0.28)), Vector((0.0, 1.0, -0.6)), SIGN["arm_l"])
        point(rig, "hand_l", Vector((0.2, -0.8, -0.2)))
    # Head looks at home plate: the head bone's +Z points out of the face.
    head_dir_now = (rig.matrix_world.to_3x3() @ rig.pose.bones["head"].matrix.to_3x3().col[2]).normalized()
    flat = Vector((head_dir_now.x, head_dir_now.y, 0.0)).normalized()
    want = Vector((1.0, 0.0, 0.0))
    yaw = math.degrees(math.atan2(flat.cross(want).z, flat.dot(want)))
    twist(rig, "neck_01", 0.45 * yaw)
    twist(rig, "head", 0.55 * yaw)
    # Legs: front knee flexed towards home, back leg driving off the rubber.
    two_bone(rig, "thigh_l", "calf_l", front_ankle + Vector((0, 0, 0.0)), Vector((1.0, 0.0, 0.3)), SIGN["leg_l"])
    point(rig, "foot_l", Vector((0.95, -0.10, -0.30)))
    two_bone(rig, "thigh_r", "calf_r", back_ankle, Vector((0.6, -0.6, -0.1)), SIGN["leg_r"])
    point(rig, "foot_r", Vector((0.15, -0.55, -0.82)))
    # Fingers of the ball hand close a little.
    for f in ("index", "middle", "ring", "pinky"):
        for i, a in ((1, 35.0), (2, 45.0), (3, 30.0)):
            rotate_x(rig, f"{f}_0{i}_r", a * FINGER_SIGN)
    upd()


def rotate_x(rig, name, deg):
    rotate_about_head(rig, name, Quaternion(x_axis(rig, name), math.radians(deg)))


SIGN = {}
FINGER_SIGN = 1.0


# ---------------------------------------------------------------------------------------
# Props and the mannequin, parented to bones (built in the rest pose)

def parent_to_bone(obj, rig, bone):
    pb = rig.pose.bones[bone]
    world = obj.matrix_world.copy()
    obj.parent = rig
    obj.parent_type = "BONE"
    obj.parent_bone = bone
    obj.matrix_parent_inverse = (rig.matrix_world @ pb.matrix @ Matrix.Translation((0.0, pb.bone.length, 0.0))).inverted()
    obj.matrix_basis = world


def ellipsoid(name, center, rot3, radii, material, smooth=True, segments=32):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=segments // 2, radius=1.0)
    o = bpy.context.active_object
    o.name = name
    o.matrix_world = Matrix.Translation(center) @ rot3.to_4x4() @ Matrix.Diagonal((*radii, 1.0))
    o.data.materials.append(material)
    if smooth:
        bpy.ops.object.shade_smooth()
    return o


def bone_frame(rig, name):
    """Rest frame of a bone in world space: (head, tail, 3x3 rotation with Y along the bone)."""
    b = rig.data.bones[name]
    mw = rig.matrix_world
    head = mw @ b.head_local
    tail = mw @ b.tail_local
    rot = (mw.to_3x3().normalized() @ b.matrix_local.to_3x3()).normalized()
    return head, tail, rot


def segment(rig, bone, rx, rz, material, name=None, extend=1.12, shift=0.5, half=None):
    head, tail, rot = bone_frame(rig, bone)
    length = (tail - head).length
    center = head + (tail - head) * shift
    o = ellipsoid(name or ("mq_" + bone), center, rot, (rx, half if half else length * 0.5 * extend, rz), material)
    parent_to_bone(o, rig, bone)
    return o


def build_mannequin(rig):
    """A stylized figure: one smooth ellipsoid per body segment, sized from the MakeHuman body
    of the same height, coloured like the uniform."""
    parts = []
    s = rig.matrix_world.to_scale()[0]
    J, SK, SH = M["jersey"], M["skin"], M["shoe"]
    # Torso: three large overlapping shapes so that it reads as one piece, twisting between
    # the hips and the shoulders.
    # Heights at 1.90 m (rest): hip centre 1.04, spine_03 1.27-1.42, shoulders 1.53, neck base
    # 1.64, top of the head 1.91. Chest 1.25-1.65 with the shoulder joints on its surface.
    parts.append(segment(rig, "pelvis", 0.18, 0.13, J, shift=-0.4, half=0.125))
    parts.append(segment(rig, "spine_01", 0.165, 0.12, J, shift=0.7, half=0.135))
    parts.append(segment(rig, "spine_03", 0.20, 0.135, J, shift=1.22, half=0.205))
    parts.append(segment(rig, "neck_01", 0.056, 0.056, SK, shift=0.5, half=0.09))
    parts.append(segment(rig, "head", 0.088, 0.10, SK, shift=0.43, half=0.115))
    for side in ("l", "r"):
        parts.append(segment(rig, f"upperarm_{side}", 0.058, 0.058, J))
        parts.append(segment(rig, f"lowerarm_{side}", 0.045, 0.045, SK))
        parts.append(segment(rig, f"thigh_{side}", 0.088, 0.088, J))
        parts.append(segment(rig, f"calf_{side}", 0.063, 0.063, J))
        parts.append(segment(rig, f"foot_{side}", 0.052, 0.04, SH, extend=1.6, shift=0.55))
        if side == "r":
            parts.append(segment(rig, "hand_r", 0.045, 0.024, SK, extend=4.0, shift=1.6))
    return parts


META_COUNT = {}


def meta_part(family, rig, bone, center, rot3, radii, material):
    """One metaball element in its own object, parented to a bone. Objects named FAMILY,
    FAMILY.001, ... blend into one smooth surface; the base object's settings and material apply.
    Element radius 1.49 with stiffness 2 and threshold 0.6 puts the surface at 1 (object space),
    so the object's scale gives the radii of a lone part."""
    n = META_COUNT.get(family, 0)
    META_COUNT[family] = n + 1
    name = family if n == 0 else f"{family}.{n:03d}"
    mb = bpy.data.metaballs.new(name)
    mb.resolution = 0.03
    mb.render_resolution = 0.012
    mb.threshold = 0.6
    # An ellipsoid element: the field is a ball of radius 1.49 in space scaled by the sizes, so
    # with stiffness 2 and threshold 0.6 the surface of a lone part lies at the sizes (Blender's
    # metaball field s*(1-d^2/r^2)^2). The object itself is only rotated and moved.
    el = mb.elements.new(type="ELLIPSOID")
    el.co = (0.0, 0.0, 0.0)
    el.radius = 1.49
    el.stiffness = 2.0
    el.size_x, el.size_y, el.size_z = radii
    if n == 0:
        mb.materials.append(material)
    o = bpy.data.objects.new(name, mb)
    bpy.context.scene.collection.objects.link(o)
    o.matrix_world = Matrix.Translation(center) @ rot3.to_4x4()
    parent_to_bone(o, rig, bone)
    return o


def meta_segment(family, rig, bone, rx, rz, material, extend=1.0, shift=0.5, half=None):
    head, tail, rot = bone_frame(rig, bone)
    length = (tail - head).length
    center = head + (tail - head) * shift
    return meta_part(family, rig, bone, center, rot, (rx, half if half else length * 0.5 * extend, rz), material)


def build_mannequin_meta(rig):
    """The stylized figure as two blended surfaces (uniform, skin) plus rigid shoes: the
    same sizes as the ellipsoid figure, but joints and waist flow together."""
    U, SK = "MBUni", "MBSkin"
    J, S_, SH = M["jersey"], M["skin"], M["shoe"]
    parts = []
    parts.append(meta_segment(U, rig, "pelvis", 0.17, 0.12, J, shift=-0.4, half=0.12))
    parts.append(meta_segment(U, rig, "spine_01", 0.15, 0.11, J, shift=0.7, half=0.13))
    parts.append(meta_segment(U, rig, "spine_03", 0.19, 0.125, J, shift=1.2, half=0.19))
    parts.append(meta_segment(SK, rig, "neck_01", 0.052, 0.052, S_, shift=0.45, half=0.085))
    parts.append(meta_segment(SK, rig, "head", 0.085, 0.098, S_, shift=0.43, half=0.112))
    for side in ("l", "r"):
        parts.append(meta_segment(U, rig, f"upperarm_{side}", 0.055, 0.055, J, extend=1.05))
        parts.append(meta_segment(SK, rig, f"lowerarm_{side}", 0.043, 0.043, S_, extend=1.0))
        parts.append(meta_segment(U, rig, f"thigh_{side}", 0.085, 0.085, J, extend=1.0))
        parts.append(meta_segment(U, rig, f"calf_{side}", 0.06, 0.06, J, extend=1.0))
        parts.append(segment(rig, f"foot_{side}", 0.052, 0.04, SH, extend=1.6, shift=0.55))
    parts.append(meta_segment(SK, rig, "hand_r", 0.043, 0.024, S_, extend=4.0, shift=1.6))
    return parts


def build_mannequin_skin(rig):
    """The stylized figure as ONE smooth skinned mesh, the way a stylized game character is
    built: a skeleton of joint points with a thickness at each (Blender's Skin modifier), smoothed
    (Subdivision Surface), bound to the same rig by distance weights (linear blend skinning, like
    the MakeHuman body). Shoes stay rigid."""
    import bmesh

    def hd(n):
        return rig.matrix_world @ rig.data.bones[n].head_local

    def tl(n):
        return rig.matrix_world @ rig.data.bones[n].tail_local

    pts, radii, edges = [], [], []

    def add(p, r):
        pts.append(Vector(p))
        radii.append(r)
        return len(pts) - 1

    def chain(ids):
        for a, b in zip(ids, ids[1:]):
            edges.append((a, b))

    # (depth, width) for the vertical chain: the Skin modifier's first radius lies front to back
    # here. Radii are about 1.3 times the body's, since smoothing shrinks the hull.
    TW = float(os.environ.get("TW", "1.0"))
    p0 = add(hd("pelvis") + Vector((0, 0, -0.01)), (0.15, 0.20 * TW))
    s1 = add(hd("spine_01"), (0.14, 0.185 * TW))
    s3 = add(hd("spine_03"), (0.15, 0.205 * TW))
    s4 = add(tl("spine_03"), (0.155, 0.225 * TW))
    cn = add(Vector((0.0, -0.02, 1.555)), (0.13, 0.17 * TW))
    nb = add(hd("neck_01"), (0.068, 0.068))
    nt = add(hd("head"), (0.07, 0.07))
    hc = add(hd("head") + (tl("head") - hd("head")) * 0.48, (0.125, 0.112))
    ht = add(tl("head") - (tl("head") - hd("head")) * 0.04, (0.05, 0.05))
    chain([p0, s1, s3, s4, cn, nb, nt, hc, ht])
    for sd in ("l", "r"):
        sh = add(hd(f"upperarm_{sd}"), (0.08, 0.08))
        el = add(hd(f"lowerarm_{sd}"), (0.06, 0.06))
        wr = add(hd(f"hand_{sd}"), (0.045, 0.038))
        d = (hd(f"hand_{sd}") - hd(f"lowerarm_{sd}")).normalized()
        he = add(hd(f"hand_{sd}") + d * 0.085, (0.05, 0.026))
        chain([cn, sh, el, wr, he])
        hp = add(hd(f"thigh_{sd}"), (0.125, 0.125))
        kn = add(hd(f"calf_{sd}"), (0.08, 0.08))
        an = add(hd(f"foot_{sd}") + Vector((0, 0, 0.02)), (0.06, 0.06))
        chain([p0, hp, kn, an])
    me = bpy.data.meshes.new("stylized")
    me.from_pydata([tuple(p) for p in pts], edges, [])
    o = bpy.data.objects.new("stylized", me)
    bpy.context.scene.collection.objects.link(o)
    skin = o.modifiers.new("skin", "SKIN")
    skin.use_smooth_shade = True
    sv = me.skin_vertices[0].data
    for i, r in enumerate(radii):
        sv[i].radius = r
        sv[i].use_root = (i == p0)
    sub = o.modifiers.new("sub", "SUBSURF")
    sub.levels = sub.render_levels = 2
    bpy.context.view_layer.objects.active = o
    for mname in ("skin", "sub"):
        with bpy.context.temp_override(object=o, active_object=o):
            bpy.ops.object.modifier_apply(modifier=mname)
    # Weights by distance to each deforming bone (the nearest three, falling off with the
    # fourth power of the distance), normalized.
    bones = [b.name for b in rig.data.bones if b.use_deform and not any(k in b.name for k in ("index", "middle", "ring", "pinky", "thumb"))]
    segs = {n: (hd(n), tl(n)) for n in bones}

    def dist(p, a, b):
        ab = b - a
        t = max(0.0, min(1.0, (p - a).dot(ab) / max(ab.length_squared, 1e-9)))
        return (p - (a + ab * t)).length

    groups = {n: o.vertex_groups.new(name=n) for n in bones}
    skin_bones = {"neck_01", "head", "lowerarm_l", "lowerarm_r", "hand_l", "hand_r"}
    dominant = []
    for v in me.vertices:
        p = o.matrix_world @ v.co
        ds = sorted((dist(p, *segs[n]), n) for n in bones)[:3]
        ws = [(1.0 / (d + 0.02) ** 4, n) for d, n in ds]
        tot = sum(w for w, _ in ws)
        for w, n in ws:
            groups[n].add([v.index], w / tot, "REPLACE")
        dominant.append(ds[0][1])
    o.data.materials.append(M["jersey"])
    o.data.materials.append(M["skin"])
    for f in me.polygons:
        votes = sum(1 for i in f.vertices if dominant[i] in skin_bones)
        f.material_index = 1 if votes * 2 > len(f.vertices) else 0
    o.parent = rig
    o.matrix_parent_inverse = rig.matrix_world.inverted()
    arm = o.modifiers.new("rig", "ARMATURE")
    arm.object = rig
    parts = [o]
    for sd in ("l", "r"):
        parts.append(segment(rig, f"foot_{sd}", 0.052, 0.04, M["shoe"], extend=1.6, shift=0.55))
    return parts


def build_props(rig):
    """Cap, glove and ball, shared by both figures."""
    props = []
    head, tail, rot = bone_frame(rig, "head")
    up = (tail - head).normalized()
    fwd = Vector((0.0, -1.0, 0.0))
    crown_c = head + up * 0.115 + fwd * 0.012
    crown = ellipsoid("cap_crown", crown_c, rot, (0.103, 0.075, 0.112), M["cap"])
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="DESELECT")
    bpy.ops.object.mode_set(mode="OBJECT")
    for v in crown.data.vertices:
        if v.co.y < -0.05:
            v.select = True
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.delete(type="VERT")
    bpy.ops.object.mode_set(mode="OBJECT")
    props.append(crown)
    brim = ellipsoid("cap_brim", crown_c - up * 0.005 + fwd * 0.10, rot, (0.085, 0.006, 0.075), M["cap"])
    props.append(brim)
    for o in (crown, brim):
        parent_to_bone(o, rig, "head")
    h, t, r = bone_frame(rig, "hand_l")
    glove = ellipsoid("glove", h + (t - h).normalized() * 0.08 + Vector((0.025, 0.0, 0.0)), r, (0.07, 0.14, 0.045), M["glove"])
    parent_to_bone(glove, rig, "hand_l")
    props.append(glove)
    h, t, r = bone_frame(rig, "hand_r")
    palm = (r @ Vector((0.0, 0.0, 1.0))).normalized()
    ball = ellipsoid("ball", h + (t - h).normalized() * 0.09 + palm * 0.045, Matrix.Identity(3), (0.0366,) * 3, M["ball"])
    parent_to_bone(ball, rig, "hand_r")
    props.append(ball)
    return props


# ---------------------------------------------------------------------------------------
# Field, light, cameras

def build_field():
    bpy.ops.mesh.primitive_plane_add(size=400, location=(HOME_X / 2, 0, 0))
    bpy.context.active_object.data.materials.append(M["grass"])
    bpy.ops.mesh.primitive_cylinder_add(vertices=96, radius=2.74, depth=0.02, location=(0.3, 0, 0.0))
    bpy.context.active_object.data.materials.append(M["dirt"])
    bpy.ops.mesh.primitive_cube_add(size=1, location=(0.12, 0, 0.012))
    rubber = bpy.context.active_object
    rubber.scale = (0.15, 0.61, 0.012)
    rubber.data.materials.append(M["white"])
    bpy.ops.mesh.primitive_cylinder_add(vertices=96, radius=4.0, depth=0.02, location=(HOME_X + 0.3, 0, 0.0))
    bpy.context.active_object.data.materials.append(M["dirt"])
    bpy.ops.mesh.primitive_cylinder_add(vertices=5, radius=0.3, depth=0.03, location=(HOME_X + 0.2, 0, 0.012))
    plate = bpy.context.active_object
    plate.rotation_euler = (0, 0, math.pi)
    plate.data.materials.append(M["white"])
    # a dirt path along the line, for depth in the broadcast view
    bpy.ops.mesh.primitive_plane_add(size=1, location=(HOME_X / 2 + 0.6, 0, 0.004))
    path = bpy.context.active_object
    path.scale = (HOME_X - 4.0, 1.0, 1)
    path.data.materials.append(M["dirt"])


def build_light():
    scene = bpy.context.scene
    world = bpy.data.worlds.new("sky")
    world.use_nodes = True
    nt = world.node_tree
    sky = nt.nodes.new("ShaderNodeTexSky")
    sky.sky_type = "NISHITA"
    sky.sun_disc = False
    sky.sun_elevation = math.radians(42)
    sky.sun_rotation = math.radians(250)
    nt.links.new(sky.outputs[0], nt.nodes["Background"].inputs[0])
    nt.nodes["Background"].inputs[1].default_value = 0.8
    scene.world = world
    sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
    sun.data.energy = 3.5
    sun.data.angle = math.radians(1.0)
    d = Vector((math.cos(math.radians(250)) * math.cos(math.radians(42)), math.sin(math.radians(250)) * math.cos(math.radians(42)), math.sin(math.radians(42))))
    sun.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
    scene.collection.objects.link(sun)


def camera(name, loc, target, lens, res):
    cam = bpy.data.objects.new(name, bpy.data.cameras.new(name))
    bpy.context.scene.collection.objects.link(cam)
    cam.location = loc
    cam.rotation_euler = (Vector(target) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
    cam.data.lens = lens
    cam.data.sensor_fit = "HORIZONTAL"
    cam.data.sensor_width = 36.0
    cam["res"] = res
    return cam


def render(cam, path):
    scene = bpy.context.scene
    scene.camera = cam
    scene.render.resolution_x, scene.render.resolution_y = cam["res"]
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    print("RENDERED", path)


def main():
    global FINGER_SIGN
    body, rig, h0 = build_human()
    make_materials()
    s = 1.90 / h0
    rig.scale = (s, s, s)
    upd()
    print("HEIGHT", round(h0, 3), "scale", round(s, 4))
    if os.environ.get("JOINTS"):
        for n in ("pelvis", "spine_01", "spine_02", "spine_03", "neck_01", "head", "clavicle_r", "upperarm_r", "lowerarm_r", "hand_r", "thigh_r", "calf_r", "foot_r", "ball_r"):
            b = rig.data.bones[n]
            print("REST", n, "head", tuple(round(x, 3) for x in rig.matrix_world @ b.head_local), "tail", tuple(round(x, 3) for x in rig.matrix_world @ b.tail_local))
    SIGN["arm_r"] = rest_bend_sign(rig, "upperarm_r", "lowerarm_r")
    SIGN["arm_l"] = rest_bend_sign(rig, "upperarm_l", "lowerarm_l")
    SIGN["leg_l"] = rest_bend_sign(rig, "thigh_l", "calf_l")
    SIGN["leg_r"] = rest_bend_sign(rig, "thigh_r", "calf_r")
    FINGER_SIGN = float(os.environ.get("FINGER_SIGN", "1"))
    print("SIGNS", SIGN, "finger", FINGER_SIGN)
    human_objs = [body] + [o for o in rig.children if o.type == "MESH"]
    mq = os.environ.get("MQ", "skin")
    mannequin = {"skin": build_mannequin_skin, "meta": build_mannequin_meta, "rigid": build_mannequin}[mq](rig)
    props = build_props(rig)
    pose(rig, POSE)
    build_field()
    build_light()
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = SAMPLES
    scene.cycles.use_denoising = True
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.view_settings.exposure = -0.4
    cams = {
        "side": camera("side", (0.9, -6.8, 1.1), (0.9, 0.0, 1.0), 66, (1100, 900)),
        "close": camera("close", (0.2, -2.6, 1.75), (0.35, 0.0, 1.45), 50, (900, 900)),
        "cf": camera("cf", (-110.0, -6.5, 14.5), (8.0, 0.0, 1.5), 400, (1280, 720)),
    }
    if os.environ.get("JOINTS"):
        for n in ("pelvis", "spine_01", "spine_02", "spine_03", "neck_01", "head", "clavicle_r", "upperarm_r", "lowerarm_r", "hand_r", "thigh_r", "calf_r", "foot_r", "ball_r"):
            b = rig.data.bones[n]
            print("JOINT", n, "head", tuple(round(x, 3) for x in rig.matrix_world @ b.head_local), "tail", tuple(round(x, 3) for x in rig.matrix_world @ b.tail_local))
    if os.environ.get("DEBUG"):
        upd()
        for o in mannequin:
            mw = o.matrix_world
            print("PART", o.name, "scale", tuple(round(x, 3) for x in mw.to_scale()), "basis", tuple(round(x, 3) for x in o.matrix_basis.to_scale()), "pinv", tuple(round(x, 3) for x in o.matrix_parent_inverse.to_scale()))
    # A close look at the throwing shoulder, from behind, above and the throwing-arm side.
    sh, _ = pb_world(rig, "upperarm_r")
    el, _ = pb_world(rig, "lowerarm_r")
    aim = (sh + el) * 0.5
    cams["shoulder"] = camera("shoulder", aim + Vector((-1.0, -1.1, 0.55)), aim, 55, (900, 900))
    variants = {"A_mannequin": (mannequin, human_objs), "B_makehuman": (human_objs, mannequin)}
    for vname, (show, hide) in variants.items():
        for o in show:
            o.hide_render = False
        for o in hide:
            o.hide_render = True
        for view in VIEWS:
            if view not in cams:
                continue
            render(cams[view], os.path.join(OUT, f"{POSE}_{view}_{vname}.png"))


main()
