"""The refined MakeHuman pitcher, animated with a CMU motion-capture pitch.

Joint positions come from cmu_fk.py. Each frame the rig is posed from them: the pelvis by
the frame of its hips and lower back, the spine by its segments with the shoulder line
setting the twist, arms and legs as two-bone chains (the target scaled by the limb-length
ratio, the bend toward the measured elbow or knee), hands and feet along their segments,
the head toward the measured face direction. The fingers keep the four-seam grip (right)
and the glove (left). The pelvis height is then corrected so the lower foot stays on the
ground. The ball rides in the hand until the frame of peak hand speed, then flies to the
plate.

usage: blender -b --python motion.py -- OUTDIR ffc SAMPLES side
env:   MOTION=pitch.json CLIP=36:150 [FRAMES=60,90,111] DECALS=dir
"""
import json
import math
import os
import sys

import bpy
from mathutils import Matrix, Quaternion, Vector

_here = os.path.dirname(os.path.abspath(sys.argv[sys.argv.index("--python") + 1]))
_src = open(os.path.join(_here, "refine.py")).read()
_src = _src[:_src.rindex("\nmain_refined()")]
exec(compile(_src, os.path.join(_here, "refine.py"), "exec"), globals())

MOTION = os.environ["MOTION"]
CLIP = tuple(int(x) for x in os.environ.get("CLIP", "36:150").split(":"))
ONLY = [int(x) for x in os.environ["FRAMES"].split(",")] if os.environ.get("FRAMES") else None
RUBBER = Vector((0.20, 0.0, 0.0))
HOME_TARGET = Vector((HOME_X - 0.3, 0.0, 0.75))
PITCH_SPEED = 38.0


def v3(a):
    return Vector(a)


def frame_axes(left, up):
    """Orthonormal frame (columns: left, up, forward) from a left-pointing and an up-pointing vector."""
    a = left.normalized()
    f = a.cross(up).normalized()
    u = f.cross(a).normalized()
    return Matrix((a, u, f)).transposed()


class Mocap:
    def __init__(self, path):
        d = json.load(open(path))
        self.info = d["info"]
        self.raw = d["frames"]
        f0 = self.raw[0]
        # Throwing direction: from the pivot (right) toe at the start to the landing of the
        # front (left) foot (its lowest point after the leg lift).
        lift = max(range(len(self.raw)), key=lambda k: self.raw[k]["ltoes"][2])
        land = next(k for k in range(lift, len(self.raw)) if self.raw[k]["ltoes"][2] < 0.04)
        self.land = land
        pivot = v3(f0["rtoes"])
        stride = v3(self.raw[land]["ltoes"]) - pivot
        stride.z = 0
        self.yaw = math.atan2(stride.y, stride.x)
        self.R = Matrix.Rotation(-self.yaw, 3, "Z")
        self.pivot = pivot
        print("MOCAP frames", len(self.raw), "leg lift", lift, "landing", land, "stride", round(stride.length, 3), "yaw deg", round(math.degrees(self.yaw), 1))

    def scale_to(self, leg_mh):
        self.s = leg_mh / self.info["femur_tibia_left"]
        print("MOCAP scale", round(self.s, 3))

    def p(self, k, joint):
        """Joint position in the scene: turned so the throw goes toward +X, the pivot toe on the
        rubber, scaled to the MakeHuman body."""
        q = self.R @ ((v3(self.raw[k][joint]) - self.pivot) * self.s)
        q.z = self.raw[k][joint][2] * self.s
        return RUBBER + Vector((q.x, q.y, 0.0)) + Vector((0, 0, q.z))

    def d(self, k, joint):
        return self.R @ v3(self.raw[k][joint])


def bone_frame_now(rig, name):
    pb = rig.pose.bones[name]
    r = rig.matrix_world.to_3x3().normalized() @ pb.matrix.to_3x3()
    return r


def set_world_rotation(rig, name, rot_world, head_world=None):
    """Give a pose bone a world orientation (and, for the root, a head position)."""
    upd()
    pb = rig.pose.bones[name]
    r3 = rig.matrix_world.to_3x3().normalized()
    rot_arm = r3.inverted() @ rot_world
    head_arm = rig.matrix_world.inverted() @ head_world if head_world is not None else pb.head.copy()
    pb.matrix = Matrix.Translation(head_arm) @ rot_arm.to_4x4()
    upd()


def twist_to(rig, name, axis_local, target_world):
    """Turn a bone about its own axis so that its local axis `axis_local` (0 = X, 2 = Z) points
    as close as it can to `target_world`."""
    h, t = pb_world(rig, name)
    b = (t - h).normalized()
    cur = bone_frame_now(rig, name).col[axis_local].normalized()
    cur_p = (cur - b * cur.dot(b))
    tgt_p = (target_world - b * target_world.dot(b))
    if cur_p.length < 1e-6 or tgt_p.length < 1e-6:
        return 0.0
    cur_p.normalize()
    tgt_p.normalize()
    ang = math.atan2(b.dot(cur_p.cross(tgt_p)), cur_p.dot(tgt_p))
    rotate_about_head(rig, name, Quaternion(b, ang))
    return ang


def main_motion():
    global FINGER_SIGN
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

    # Rest-pose references.
    def L(a, b):
        return (pb_world(rig, a)[0] - pb_world(rig, b)[0]).length
    leg_mh = (pb_world(rig, "calf_l")[0] - pb_world(rig, "thigh_l")[0]).length + (pb_world(rig, "foot_l")[0] - pb_world(rig, "calf_l")[0]).length
    arm_mh = (pb_world(rig, "lowerarm_r")[0] - pb_world(rig, "upperarm_r")[0]).length + (pb_world(rig, "hand_r")[0] - pb_world(rig, "lowerarm_r")[0]).length
    mc = Mocap(MOTION)
    mc.scale_to(leg_mh)
    arm_cmu = mc.info["humerus_radius_right"]
    arm_ratio = arm_mh / (arm_cmu * mc.s)
    print("LIMBS mh leg", round(leg_mh, 3), "mh arm", round(arm_mh, 3), "arm ratio after scaling", round(arm_ratio, 3))
    ball_rest_z = min(pb_world(rig, "foot_l")[1].z, pb_world(rig, "foot_r")[1].z)
    # CMU rest frame of the pelvis, in the same (Blender, unturned) axes: from the ASF rest pose
    # the hips are level and the subject faces -Y.
    F_rest_cmu = frame_axes(Vector((1, 0, 0)), Vector((0, 0, 1)))
    pel_rest = bone_frame_now(rig, "pelvis").copy()
    # Sign of the spine's local X relative to the body's left (+X at rest).
    sx_spine = 1.0 if bone_frame_now(rig, "spine_03").col[0].x > 0 else -1.0

    # Grip and glove in the rest pose; the fingers' local rotations then hold in every frame.
    C, rot = grip(rig)
    hand_rest = rig.matrix_world @ rig.pose.bones["hand_r"].matrix
    ball_local = hand_rest.inverted() @ C
    ball_rot_local = hand_rest.to_3x3().normalized().inverted() @ rot
    glove = build_glove(rig)
    laces = bpy.data.objects.get("glove_laces")
    cap_parts = build_cap(rig, body)
    upd()
    fingers = [pb.name for pb in rig.pose.bones if pb.name.split("_")[0] in ("thumb", "index", "middle", "ring", "pinky")]
    finger_rot = {n: rig.pose.bones[n].rotation_quaternion.copy() for n in fingers}
    for o, bone in [(glove, "hand_l")] + ([(laces, "hand_l")] if laces else []) + [(p, "head") for p in cap_parts] + [(bpy.data.objects["cap_button"], "head")]:
        parent_to_bone(o, rig, bone)
    ball_objs = build_ball(Vector((0, 0, 0)), Matrix.Identity(3))
    stitches = ball_objs[1]
    stitches.parent = ball_objs[0]
    ball = ball_objs[0]
    for o in [ball]:
        o.matrix_world = Matrix.Identity(4)

    meshes = [o for o in bpy.data.objects if o.type == "MESH"]
    k0, k1 = CLIP
    release = mc.info["peak_frame"]
    frames = list(range(k0, k1 + 1))
    scene = bpy.context.scene
    scene.frame_start = 1
    scene.frame_end = len(frames)
    scene.render.fps = 30
    hand_track = {}
    for o in meshes:
        o.hide_viewport = True
    for fi, k in enumerate(frames, start=1):
        for pb in rig.pose.bones:
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.location = (0, 0, 0)
            pb.scale = (1, 1, 1)
        upd()
        P = lambda j: mc.p(k, j)
        # Pelvis.
        root = P("root")
        hipL, hipR = P("lhipjoint"), P("rhipjoint")
        spine0 = P("lowerback")
        F_cur = frame_axes(hipL - hipR, spine0 - root)
        R_delta = F_cur @ F_rest_cmu.transposed()
        set_world_rotation(rig, "pelvis", R_delta @ pel_rest, root + (pb_world(rig, "pelvis")[0] - pb_world(rig, "pelvis")[0]))
        # Put the pelvis head where the mocap root is (the MakeHuman pelvis head is near the hip centre).
        # Spine.
        point(rig, "spine_01", P("lowerback") - P("root"))
        point(rig, "spine_02", P("upperback") - P("lowerback"))
        point(rig, "spine_03", P("thorax") - P("upperback"))
        lateral = P("lclavicle") - P("rclavicle")
        total = 0.0
        for b in ("spine_01", "spine_02", "spine_03"):
            h, t = pb_world(rig, b)
            ax = (t - h).normalized()
            cur = bone_frame_now(rig, "spine_03").col[0] * sx_spine
            cp = (cur - ax * cur.dot(ax)).normalized()
            tp = (lateral - ax * lateral.dot(ax)).normalized()
            ang = math.atan2(ax.dot(cp.cross(tp)), cp.dot(tp))
            share = {"spine_01": 1 / 3, "spine_02": 1 / 2, "spine_03": 1.0}[b]
            rotate_about_head(rig, b, Quaternion(ax, ang * share))
            total += ang * share
        # Neck and head.
        point(rig, "neck_01", P("upperneck") - P("thorax"))
        point(rig, "head", P("head") - P("upperneck"))
        twist_to(rig, "head", 2, mc.d(k, "head_forward"))
        # Shoulders and arms.
        for side, cs in (("r", "r"), ("l", "l")):
            point(rig, f"clavicle_{side}", P(f"{cs}clavicle") - P("thorax"))
            sh_c, el_c, wr_c = P(f"{cs}clavicle"), P(f"{cs}humerus"), P(f"{cs}radius")
            sh_m = pb_world(rig, f"upperarm_{side}")[0]
            target = sh_m + (wr_c - sh_c) * arm_ratio
            two_bone(rig, f"upperarm_{side}", f"lowerarm_{side}", target, el_c - sh_c, SIGN[f"arm_{side}"])
            point(rig, f"hand_{side}", P(f"{cs}hand") - P(f"{cs}radius"))
            thumb_c = P(f"{cs}thumb") - P(f"{cs}wrist")
            # The thumb of the MakeHuman hand: from the hand bone's head to the first thumb joint.
            hh = pb_world(rig, f"hand_{side}")[0]
            th = pb_world(rig, f"thumb_01_{side}")[0] - hh
            fr = bone_frame_now(rig, f"hand_{side}")
            local = fr.inverted() @ th
            ax = 0 if abs(local.x) > abs(local.z) else 2
            sgn = 1.0 if (local.x if ax == 0 else local.z) > 0 else -1.0
            twist_to(rig, f"hand_{side}", ax, thumb_c * sgn)
        # Legs.
        for side, cs in (("l", "l"), ("r", "r")):
            hip_c, knee_c, ank_c = P(f"{cs}hipjoint"), P(f"{cs}femur"), P(f"{cs}tibia")
            hip_m = pb_world(rig, f"thigh_{side}")[0]
            target = hip_m + (ank_c - hip_c)
            two_bone(rig, f"thigh_{side}", f"calf_{side}", target, knee_c - hip_c, SIGN[f"leg_{side}"])
            point(rig, f"foot_{side}", P(f"{cs}foot") - P(f"{cs}tibia"))
        for n, q in finger_rot.items():
            rig.pose.bones[n].rotation_quaternion = q
        upd()
        # Keep the lower foot on the ground.
        low = min(pb_world(rig, "foot_l")[1].z, pb_world(rig, "foot_r")[1].z, pb_world(rig, "foot_l")[0].z - 0.06, pb_world(rig, "foot_r")[0].z - 0.06)
        dz = ball_rest_z - low
        pel = rig.pose.bones["pelvis"]
        m = pel.matrix.copy()
        m.translation += rig.matrix_world.inverted().to_3x3() @ Vector((0, 0, dz))
        pel.matrix = m
        upd()
        for pb in rig.pose.bones:
            pb.scale = (1, 1, 1)
            pb.keyframe_insert("rotation_quaternion", frame=fi)
            pb.keyframe_insert("scale", frame=fi)
        pel.keyframe_insert("location", frame=fi)
        hand_track[fi] = (rig.matrix_world @ rig.pose.bones["hand_r"].matrix).copy()
    for o in meshes:
        o.hide_viewport = False
    # The ball: in the hand up to the frame of peak hand speed, then toward the plate.
    rel_fi = frames.index(release) + 1 if release in frames else len(frames)
    p_rel = hand_track[rel_fi] @ ball_local
    v = (HOME_TARGET - p_rel).normalized() * PITCH_SPEED
    v.z += 0.5 * 9.81 * ((HOME_TARGET - p_rel).length / PITCH_SPEED)
    for fi in range(1, len(frames) + 1):
        if fi <= rel_fi:
            mw = hand_track[fi]
            pos = mw @ ball_local
            r = mw.to_3x3().normalized() @ ball_rot_local
        else:
            t = (fi - rel_fi) / 30.0
            pos = p_rel + v * t + Vector((0, 0, -0.5 * 9.81 * t * t))
            r = hand_track[rel_fi].to_3x3().normalized() @ ball_rot_local
        ball.matrix_world = Matrix.Translation(pos) @ r.to_4x4()
        ball.keyframe_insert("location", frame=fi)
        ball.keyframe_insert("rotation_euler", frame=fi)
    print("BALL release frame", rel_fi, "of", len(frames), "at", tuple(round(c, 2) for c in p_rel))

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
    scene.render.motion_blur_shutter = 0.5
    cams = {
        "side": camera("side", (0.9, -7.6, 1.15), (0.9, 0.0, 1.0), 40, (1280, 720)),
        "cf": camera("cf", (-110.0, -6.5, 14.5), (8.0, 0.0, 1.5), 330, (1280, 720)),
        "front": camera("front", (6.5, -3.2, 1.4), (0.8, 0.0, 1.1), 50, (1280, 720)),
        # Portrait close-up for a vertical video (the whole delivery fits 2.7 m across).
        "close": camera("close", (0.9, -5.2, 1.1), (0.9, 0.0, 0.95), 69, (1080, 1350)),
    }
    if os.environ.get("DEBUG_BBOX"):
        dg = bpy.context.evaluated_depsgraph_get()
        for fi in [int(x) for x in os.environ["DEBUG_BBOX"].split(",")]:
            scene.frame_set(fi)
            dg = bpy.context.evaluated_depsgraph_get()
            hand = (rig.matrix_world @ rig.pose.bones["hand_r"].matrix).translation
            handl = (rig.matrix_world @ rig.pose.bones["hand_l"].matrix).translation
            for o in bpy.data.objects:
                if o.type != "MESH":
                    continue
                ev = o.evaluated_get(dg)
                me = ev.to_mesh()
                pts = [ev.matrix_world @ v.co for v in me.vertices]
                ev.to_mesh_clear()
                if not pts:
                    continue
                far_r = max((p - hand).length for p in pts)
                far_l = max((p - handl).length for p in pts)
                zs = [p.z for p in pts]
                print("BBOX", fi, o.name, "n", len(pts), "z", round(min(zs), 2), round(max(zs), 2), "far_from_hand_r", round(far_r, 2), "far_l", round(far_l, 2))
            for b in ("hand_r", "thumb_01_r", "thumb_03_r", "index_03_r", "hand_l", "thumb_03_l", "pinky_03_l"):
                h, t = pb_world(rig, b)
                print("BONE", fi, b, tuple(round(c, 2) for c in h), tuple(round(c, 2) for c in t), "len", round((t - h).length, 3))
        return
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


main_motion()
