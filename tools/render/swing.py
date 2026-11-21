"""The refined MakeHuman body swinging a bat: CMU trial 124_07 (a right-handed batter).

Same retargeting as motion.py (pelvis frame, spine with shoulder-line twist, two-bone arms and
legs, head toward the face, ground lock). Differences: the batter stands beside home plate with
the hip line at the stance pointing back at the mound, both hands make fists, and a bat is
placed every frame on the line fitted through the two fists (knob under the bottom hand).

usage: blender -b --python swing.py -- OUTDIR ffc SAMPLES bat_side
env:   MOTION=swing.json CLIP=40:125 [FRAMES=...] DECALS=dir [HANDS=1 prints fist checks]
       [FPS=30 to match the motion file's rate] [SHUTTER=0.5 frames]
"""
import json
import math
import os
import sys

import bpy
import bmesh
from mathutils import Matrix, Quaternion, Vector

_here = os.path.dirname(os.path.abspath(sys.argv[sys.argv.index("--python") + 1]))
_src = open(os.path.join(_here, "motion.py")).read()
_src = _src[:_src.rindex("\nmain_motion()")]
exec(compile(_src, os.path.join(_here, "motion.py"), "exec"), globals())

BOX = Vector((HOME_X + 0.2, -0.80, 0.0))  # midpoint of the feet at the stance
BAT_LEN = 0.84  # 33 in
# (distance from the knob end, radius) in metres; barrel 2.6 in, handle about 1 in.
BAT_PROFILE = [(0.0, 0.0), (0.0, 0.020), (0.006, 0.026), (0.016, 0.024), (0.022, 0.0125), (0.25, 0.0125), (0.36, 0.016),
               (0.50, 0.028), (0.62, 0.032), (0.82, 0.033), (0.835, 0.030), (0.84, 0.0)]


class SwingMocap(Mocap):
    def __init__(self, path):
        d = json.load(open(path))
        self.info = d["info"]
        self.raw = d["frames"]
        f0 = self.raw[0]
        hl = v3(f0["lhipjoint"]) - v3(f0["rhipjoint"])
        hl.z = 0
        self.yaw = math.atan2(hl.y, hl.x)
        # The front (left) hip points at the mound, which is towards -X from the plate.
        self.R = Matrix.Rotation(math.pi - self.yaw, 3, "Z")
        self.pivot = (v3(f0["ltoes"]) + v3(f0["rtoes"])) / 2
        self.pivot.z = 0
        print("SWING frames", len(self.raw), "hip line yaw deg", round(math.degrees(self.yaw), 1), "peak", self.info["peak_frame"])

    def p(self, k, joint):
        q = self.R @ ((v3(self.raw[k][joint]) - self.pivot) * self.s)
        q.z = self.raw[k][joint][2] * self.s
        return BOX + Vector((q.x, q.y, 0.0)) + Vector((0, 0, q.z))


def build_bat():
    bm = bmesh.new()
    n = 32
    rings = []
    for z, r in BAT_PROFILE:
        if r == 0.0:
            rings.append([bm.verts.new((0.0, 0.0, z))])
        else:
            rings.append([bm.verts.new((r * math.cos(2 * math.pi * i / n), r * math.sin(2 * math.pi * i / n), z)) for i in range(n)])
    for a, b in zip(rings, rings[1:]):
        if len(a) == 1 and len(b) == 1:
            continue
        if len(a) == 1:
            for i in range(n):
                bm.faces.new((a[0], b[(i + 1) % n], b[i]))
        elif len(b) == 1:
            for i in range(n):
                bm.faces.new((a[i], a[(i + 1) % n], b[0]))
        else:
            for i in range(n):
                bm.faces.new((a[i], a[(i + 1) % n], b[(i + 1) % n], b[i]))
    me = bpy.data.meshes.new("bat")
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons:
        p.use_smooth = True
    wood = principled("bat_wood", (0.56, 0.36, 0.17), 0.35)
    tape = principled("bat_tape", (0.05, 0.05, 0.05), 0.8)
    me.materials.append(wood)
    me.materials.append(tape)
    for p in me.polygons:
        zc = sum(me.vertices[v].co.z for v in p.vertices) / len(p.vertices)
        p.material_index = 1 if zc < 0.24 else 0
    o = bpy.data.objects.new("bat", me)
    bpy.context.scene.collection.objects.link(o)
    o.rotation_mode = "QUATERNION"
    return o


def circumcenter(a, b, c):
    ab, ac = b - a, c - a
    n = ab.cross(ac)
    if n.length < 1e-9:
        return (a + b + c) / 3
    return a + (n.cross(ab) * ac.length_squared + ac.cross(n) * ab.length_squared) / (2 * n.length_squared)


def fist_centres(rig, side):
    """Where the handle runs through a fist: per finger, the centre of the circle through its
    knuckle, middle and end joints."""
    out = []
    for f in ("index", "middle", "ring", "pinky"):
        a = pb_world(rig, f"{f}_01_{side}")[0]
        b = pb_world(rig, f"{f}_02_{side}")[0]
        c = pb_world(rig, f"{f}_03_{side}")[0]
        out.append(circumcenter(a, b, c))
    return out


def fit_line(pts):
    c = sum(pts, Vector()) / len(pts)
    # principal direction by power iteration on the scatter matrix
    M = Matrix(((0.0,) * 3,) * 3)
    for p in pts:
        d = p - c
        for i in range(3):
            for j in range(3):
                M[i][j] += d[i] * d[j]
    v = Vector((1.0, 0.3, 0.2))
    for _ in range(50):
        v = (M @ v).normalized()
    return c, v


def make_fists(rig):
    for side in ("r", "l"):
        for f, a in (("index", 62.0), ("middle", 66.0), ("ring", 70.0), ("pinky", 74.0)):
            curl(rig, f, side, a)
        rotate_x(rig, f"thumb_02_{side}", 25.0 * FINGER_SIGN)
        rotate_x(rig, f"thumb_03_{side}", 35.0 * FINGER_SIGN)
    upd()


def main_swing():
    body, rig, h0 = build_human()
    make_materials()
    s = 1.90 / h0
    rig.scale = (s, s, s)
    upd()
    SIGN["arm_r"] = rest_bend_sign(rig, "upperarm_r", "lowerarm_r")
    SIGN["arm_l"] = rest_bend_sign(rig, "upperarm_l", "lowerarm_l")
    SIGN["leg_l"] = rest_bend_sign(rig, "thigh_l", "calf_l")
    SIGN["leg_r"] = rest_bend_sign(rig, "thigh_r", "calf_r")
    HumanService.add_mhclo_asset(os.path.join(DATA, "hair/short02/short02.mhclo"), body, asset_type="Hair", material_type="MAKESKIN")
    suit = [o for o in rig.children if "casualsuit" in o.name][0]
    geo = suit_attributes(suit)
    jersey_shader(suit, geo)
    undershirt(body, rig)
    for pb in rig.pose.bones:
        pb.rotation_mode = "QUATERNION"
    upd()
    leg_mh = (pb_world(rig, "calf_l")[0] - pb_world(rig, "thigh_l")[0]).length + (pb_world(rig, "foot_l")[0] - pb_world(rig, "calf_l")[0]).length
    arm_mh = (pb_world(rig, "lowerarm_r")[0] - pb_world(rig, "upperarm_r")[0]).length + (pb_world(rig, "hand_r")[0] - pb_world(rig, "lowerarm_r")[0]).length
    mc = SwingMocap(MOTION)
    mc.scale_to(leg_mh)
    arm_ratio = arm_mh / (mc.info["humerus_radius_right"] * mc.s)
    print("LIMBS mh leg", round(leg_mh, 3), "arm ratio after scaling", round(arm_ratio, 3))
    rest_ground = min(pb_world(rig, "foot_l")[1].z, pb_world(rig, "foot_r")[1].z)
    F_rest_cmu = frame_axes(Vector((1, 0, 0)), Vector((0, 0, 1)))
    pel_rest = bone_frame_now(rig, "pelvis").copy()
    sx_spine = 1.0 if bone_frame_now(rig, "spine_03").col[0].x > 0 else -1.0

    make_fists(rig)
    cap_parts = build_cap(rig, body)
    upd()
    fingers = [pb.name for pb in rig.pose.bones if pb.name.split("_")[0] in ("thumb", "index", "middle", "ring", "pinky")]
    finger_rot = {n: rig.pose.bones[n].rotation_quaternion.copy() for n in fingers}
    for o in [p for p in cap_parts] + [bpy.data.objects["cap_button"]]:
        parent_to_bone(o, rig, "head")
    bat = build_bat()

    meshes = [o for o in bpy.data.objects if o.type == "MESH" and o is not bat]
    k0, k1 = CLIP
    frames = list(range(k0, k1 + 1))
    scene = bpy.context.scene
    scene.frame_start = 1
    scene.frame_end = len(frames)
    scene.render.fps = int(os.environ.get("FPS", "30"))
    for o in meshes:
        o.hide_viewport = True
    prev_axis = None
    for fi, k in enumerate(frames, start=1):
        for pb in rig.pose.bones:
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.location = (0, 0, 0)
            pb.scale = (1, 1, 1)
        upd()
        P = lambda j: mc.p(k, j)
        root = P("root")
        F_cur = frame_axes(P("lhipjoint") - P("rhipjoint"), P("lowerback") - root)
        set_world_rotation(rig, "pelvis", F_cur @ F_rest_cmu.transposed() @ pel_rest, root)
        point(rig, "spine_01", P("lowerback") - P("root"))
        point(rig, "spine_02", P("upperback") - P("lowerback"))
        point(rig, "spine_03", P("thorax") - P("upperback"))
        lateral = P("lclavicle") - P("rclavicle")
        for b in ("spine_01", "spine_02", "spine_03"):
            h, t = pb_world(rig, b)
            ax = (t - h).normalized()
            cur = bone_frame_now(rig, "spine_03").col[0] * sx_spine
            cp = (cur - ax * cur.dot(ax)).normalized()
            tp = (lateral - ax * lateral.dot(ax)).normalized()
            ang = math.atan2(ax.dot(cp.cross(tp)), cp.dot(tp))
            rotate_about_head(rig, b, Quaternion(ax, ang * {"spine_01": 1 / 3, "spine_02": 1 / 2, "spine_03": 1.0}[b]))
        point(rig, "neck_01", P("upperneck") - P("thorax"))
        point(rig, "head", P("head") - P("upperneck"))
        twist_to(rig, "head", 2, mc.d(k, "head_forward"))
        for side in ("r", "l"):
            point(rig, f"clavicle_{side}", P(f"{side}clavicle") - P("thorax"))
            sh_c, el_c, wr_c = P(f"{side}clavicle"), P(f"{side}humerus"), P(f"{side}radius")
            target = pb_world(rig, f"upperarm_{side}")[0] + (wr_c - sh_c) * arm_ratio
            two_bone(rig, f"upperarm_{side}", f"lowerarm_{side}", target, el_c - sh_c, SIGN[f"arm_{side}"])
            point(rig, f"hand_{side}", P(f"{side}hand") - P(f"{side}radius"))
            thumb_c = P(f"{side}thumb") - P(f"{side}wrist")
            hh = pb_world(rig, f"hand_{side}")[0]
            th = pb_world(rig, f"thumb_01_{side}")[0] - hh
            local = bone_frame_now(rig, f"hand_{side}").inverted() @ th
            ax = 0 if abs(local.x) > abs(local.z) else 2
            sgn = 1.0 if (local.x if ax == 0 else local.z) > 0 else -1.0
            twist_to(rig, f"hand_{side}", ax, thumb_c * sgn)
        for side in ("l", "r"):
            hip_c, knee_c, ank_c = P(f"{side}hipjoint"), P(f"{side}femur"), P(f"{side}tibia")
            target = pb_world(rig, f"thigh_{side}")[0] + (ank_c - hip_c)
            two_bone(rig, f"thigh_{side}", f"calf_{side}", target, knee_c - hip_c, SIGN[f"leg_{side}"])
            point(rig, f"foot_{side}", P(f"{side}foot") - P(f"{side}tibia"))
        for n, q in finger_rot.items():
            rig.pose.bones[n].rotation_quaternion = q
        upd()
        low = min(pb_world(rig, "foot_l")[1].z, pb_world(rig, "foot_r")[1].z, pb_world(rig, "foot_l")[0].z - 0.06, pb_world(rig, "foot_r")[0].z - 0.06)
        pel = rig.pose.bones["pelvis"]
        m = pel.matrix.copy()
        m.translation += rig.matrix_world.inverted().to_3x3() @ Vector((0, 0, rest_ground - low))
        pel.matrix = m
        upd()
        for pb in rig.pose.bones:
            pb.scale = (1, 1, 1)
            pb.keyframe_insert("rotation_quaternion", frame=fi)
            pb.keyframe_insert("scale", frame=fi)
        pel.keyframe_insert("location", frame=fi)
        # The bat on the line through both fists, from the bottom (left) hand to the top (right).
        lf, rf = fist_centres(rig, "l"), fist_centres(rig, "r")
        c, axis = fit_line(lf + rf)
        cl = sum(lf, Vector()) / 4
        cr = sum(rf, Vector()) / 4
        if axis.dot(cr - cl) < 0:
            axis = -axis
        if prev_axis is not None and axis.dot(prev_axis) < 0.0:
            axis = -axis
        prev_axis = axis.copy()
        knob = c + axis * (cl - c).dot(axis) - axis * 0.06
        bat.location = knob
        bat.rotation_quaternion = axis.to_track_quat("Z", "Y")
        bat.keyframe_insert("location", frame=fi)
        bat.keyframe_insert("rotation_quaternion", frame=fi)
        if os.environ.get("HANDS"):
            spread = max(seg_dist(p, c - axis, c + axis) for p in lf + rf)
            print("FIST", fi, "hand gap", round((cr - cl).length, 3), "worst finger centre off the bat line", round(spread, 4))
    for o in meshes:
        o.hide_viewport = False

    build_field()
    build_light()
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = SAMPLES
    scene.cycles.use_denoising = True
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.view_settings.exposure = -0.4
    scene.render.use_motion_blur = True
    scene.render.motion_blur_shutter = float(os.environ.get("SHUTTER", "0.5"))
    x = HOME_X + 0.1
    cams = {
        # From the first-base side, across the plate, facing the batter's front.
        "bat_side": camera("bat_side", (x, 5.6, 1.15), (x, -0.8, 0.95), 45, (1280, 720)),
        "bat_close": camera("bat_close", (x, 4.0, 1.1), (x, -0.8, 0.95), 52, (1080, 1350)),
        "bat_hands": camera("bat_hands", (x + 0.2, 1.2, 1.5), (x, -0.6, 1.15), 50, (900, 900)),
        # From the mound side, as on a broadcast: a squared-around bunt shows the bat across.
        "bat_mound": camera("bat_mound", (x - 7.0, -0.85, 1.4), (x, -0.95, 0.95), 100, (1080, 1350)),
    }
    for view in VIEWS:
        cam = cams[view]
        scene.camera = cam
        scene.render.resolution_x, scene.render.resolution_y = cam["res"]
        todo = ONLY if ONLY else list(range(1, len(frames) + 1))
        for fi in todo:
            scene.frame_set(fi)
            scene.render.filepath = os.path.join(OUT, view, f"f{fi:04d}.png")
            bpy.ops.render.render(write_still=True)
            print("RENDERED", view, fi, flush=True)


main_swing()
