"""CMU motion capture (ASF skeleton + AMC motion) to joint positions per frame.

Forward kinematics as in the usual ASF/AMC readers: a bone's world rotation is
parent · C · M · C⁻¹ (C from the bone's `axis`, M from the frame's degrees of freedom, both
static XYZ Euler angles), and its end is the parent's end plus length × rotation · direction.
Units: ASF lengths are inches / 0.45 (`:units length 0.45`); output is metres.
Axes: CMU is Y-up; output is Blender's Z-up, (x, y, z) -> (x, -z, y).

usage: python3 -I cmu_fk.py SKELETON.asf MOTION.amc OUT.json [STEP]
STEP: keep every STEP-th frame (120 fps / 4 = 30 fps by default).
"""
import json
import math
import sys

import numpy as np

SCALE = 0.0254 / 0.45


def euler_xyz(ax, ay, az):
    """Static XYZ Euler angles (radians): R = Rz · Ry · Rx."""
    cx, sx = math.cos(ax), math.sin(ax)
    cy, sy = math.cos(ay), math.sin(ay)
    cz, sz = math.cos(az), math.sin(az)
    rx = np.array([[1, 0, 0], [0, cx, -sx], [0, sx, cx]])
    ry = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
    rz = np.array([[cz, -sz, 0], [sz, cz, 0], [0, 0, 1]])
    return rz @ ry @ rx


def read_asf(path):
    bones, hierarchy = {}, {}
    lines = [l.strip() for l in open(path)]
    i = 0
    section = None
    cur = None
    while i < len(lines):
        l = lines[i]
        if l.startswith(":"):
            section = l.split()[0]
            i += 1
            continue
        if section == ":bonedata":
            if l == "begin":
                cur = {"dof": []}
            elif l == "end":
                bones[cur["name"]] = cur
                cur = None
            elif cur is not None:
                parts = l.split()
                key = parts[0]
                if key == "name":
                    cur["name"] = parts[1]
                elif key == "direction":
                    cur["direction"] = np.array([float(x) for x in parts[1:4]])
                elif key == "length":
                    cur["length"] = float(parts[1]) * SCALE
                elif key == "axis":
                    cur["axis"] = np.radians([float(x) for x in parts[1:4]])
                elif key == "dof":
                    cur["dof"] = parts[1:]
        elif section == ":hierarchy":
            if l not in ("begin", "end") and l:
                parts = l.split()
                for child in parts[1:]:
                    hierarchy[child] = parts[0]
        i += 1
    for b in bones.values():
        b["C"] = euler_xyz(*b["axis"])
        b["Cinv"] = b["C"].T
    return bones, hierarchy


def read_amc(path):
    frames = []
    cur = None
    for l in open(path):
        l = l.strip()
        if not l or l.startswith("#") or l.startswith(":"):
            continue
        parts = l.split()
        if len(parts) == 1 and parts[0].isdigit():
            cur = {}
            frames.append(cur)
            continue
        if cur is not None:
            cur[parts[0]] = [float(x) for x in parts[1:]]
    return frames


def fk(bones, hierarchy, frame):
    """World end position and rotation of every bone (CMU axes, metres)."""
    root = frame["root"]
    pos = {"root": np.array(root[:3]) * SCALE}
    rot = {"root": euler_xyz(*np.radians(root[3:6]))}
    order = []
    pending = list(bones)
    while pending:
        for name in list(pending):
            parent = hierarchy[name]
            if parent in pos:
                b = bones[name]
                vals = frame.get(name, [])
                ang = {"rx": 0.0, "ry": 0.0, "rz": 0.0}
                for d, v in zip(b["dof"], vals):
                    ang[d] = math.radians(v)
                M = euler_xyz(ang["rx"], ang["ry"], ang["rz"])
                R = rot[parent] @ b["C"] @ M @ b["Cinv"]
                rot[name] = R
                pos[name] = pos[parent] + b["length"] * (R @ b["direction"])
                pending.remove(name)
                order.append(name)
    return pos, rot


def to_blender(v):
    return [float(v[0]), float(-v[2]), float(v[1])]


def main():
    asf, amc, out = sys.argv[1:4]
    step = int(sys.argv[4]) if len(sys.argv) > 4 else 4
    bones, hierarchy = read_asf(asf)
    frames = read_amc(amc)
    joints = ["root", "lhipjoint", "rhipjoint", "lfemur", "rfemur", "ltibia", "rtibia", "lfoot", "rfoot", "ltoes", "rtoes",
              "lowerback", "upperback", "thorax", "lowerneck", "upperneck", "head", "lclavicle", "rclavicle",
              "lhumerus", "rhumerus", "lradius", "rradius", "lwrist", "rwrist", "lhand", "rhand", "lthumb", "rthumb", "lfingers", "rfingers"]
    out_frames = []
    for k, f in enumerate(frames):
        if k % step:
            continue
        pos, rot = fk(bones, hierarchy, f)
        rec = {j: to_blender(pos[j]) for j in joints}
        # The face points along +Z of the head's frame at rest (the feet point that way).
        rec["head_forward"] = to_blender(rot["head"] @ np.array([0.0, 0.0, 1.0]))
        rec["root_forward"] = to_blender(rot["root"] @ np.array([0.0, 0.0, 1.0]))
        rec["rhand_side"] = to_blender(rot["rhand"] @ np.array([0.0, 0.0, 1.0]))
        rec["lhand_side"] = to_blender(rot["lhand"] @ np.array([0.0, 0.0, 1.0]))
        out_frames.append(rec)
    fps = 120 / step
    # Diagnostics: hand speeds (which hand throws, and when), feet.
    def speed(j):
        p = np.array([fr[j] for fr in out_frames])
        v = np.linalg.norm(np.diff(p, axis=0), axis=1) * fps
        return v
    vr, vl = speed("rhand"), speed("lhand")
    hand = "right" if vr.max() > vl.max() else "left"
    v = vr if hand == "right" else vl
    rel = int(np.argmax(v))
    ft = np.array([fr["ltoes"] for fr in out_frames])
    rt = np.array([fr["rtoes"] for fr in out_frames])
    info = {"fps": fps, "frames": len(out_frames), "throwing_hand": hand, "peak_hand_speed": float(v.max()), "peak_frame": rel,
            "left_toe_start": ft[0].tolist(), "left_toe_end": ft[-1].tolist(), "right_toe_start": rt[0].tolist(), "right_toe_end": rt[-1].tolist(),
            "root_start": out_frames[0]["root"], "root_end": out_frames[-1]["root"],
            "femur_tibia_left": bones["lfemur"]["length"] + bones["ltibia"]["length"],
            "humerus_radius_right": bones["rhumerus"]["length"] + bones["rradius"]["length"]}
    json.dump({"info": info, "frames": out_frames}, open(out, "w"))
    print(json.dumps(info, indent=1))
    # Speed profile around the peak
    print("hand speed (m/s) by frame:", [round(x, 1) for x in v[max(0, rel - 12): rel + 6]])
    zmin = min(min(fr["ltoes"][2], fr["rtoes"][2]) for fr in out_frames)
    print("lowest toe height", round(zmin, 3))


if __name__ == "__main__":
    main()
