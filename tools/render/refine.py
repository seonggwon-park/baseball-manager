"""The MakeHuman pitcher, refined: a baseball uniform (piping on the collar and sleeves, a placket
with buttons, a number on the back, a chest wordmark, the jersey tucked into belted trousers, a navy
long-sleeve undershirt), short hair, a cap fitted to the head, a glove built around the hand, and a
ball with stitched seams held in a four-seam grip. Same rig, poses, cameras and light as compare.py.

usage: blender -b --python refine.py -- OUTDIR POSE [SAMPLES] [VIEWS]
env:   DECALS=dir with number.png, wordmark.png, caplogo.png
"""
import math
import os
import sys

import bmesh
import bpy
import numpy as np
from mathutils import Matrix, Quaternion, Vector

_here = os.path.dirname(os.path.abspath(sys.argv[sys.argv.index("--python") + 1]))
_src = open(os.path.join(_here, "compare.py")).read()
_src = _src[:_src.rindex("\nmain()")]
exec(compile(_src, os.path.join(_here, "compare.py"), "exec"), globals())

DATA = LocationService.get_user_data()
DECALS = os.environ.get("DECALS", "")
NAVY = (0.018, 0.032, 0.11)
BALL_R = 0.0366


# ---------------------------------------------------------------------------------------
# Small helpers

def rest_coords(o):
    """World positions and normals of o's own vertices in the rest pose (shape keys included,
    every modifier off), in the order of o.data.vertices."""
    saved = [(m, m.show_viewport) for m in o.modifiers]
    for m, _ in saved:
        m.show_viewport = False
    upd()
    ev = o.evaluated_get(bpy.context.evaluated_depsgraph_get())
    me = ev.to_mesh()
    mw = o.matrix_world.copy()
    n3 = mw.to_3x3()
    pos = np.array([tuple(mw @ v.co) for v in me.vertices])
    nrm = np.array([tuple((n3 @ v.normal).normalized()) for v in me.vertices])
    ev.to_mesh_clear()
    for m, s in saved:
        m.show_viewport = s
    upd()
    assert len(pos) == len(o.data.vertices), (o.name, len(pos), len(o.data.vertices))
    return pos, nrm


def posed_coords(o, keep=("ARMATURE",)):
    """World positions of o's own vertices after only the listed modifiers (index-stable)."""
    saved = [(m, m.show_viewport) for m in o.modifiers]
    for m, _ in saved:
        m.show_viewport = m.type in keep
    upd()
    ev = o.evaluated_get(bpy.context.evaluated_depsgraph_get())
    me = ev.to_mesh()
    mw = o.matrix_world.copy()
    pos = np.array([tuple(mw @ v.co) for v in me.vertices])
    ev.to_mesh_clear()
    for m, s in saved:
        m.show_viewport = s
    upd()
    assert len(pos) == len(o.data.vertices)
    return pos


def set_attr(o, name, values):
    me = o.data
    values = np.asarray(values, dtype=np.float32)
    kind = "FLOAT" if values.ndim == 1 else "FLOAT_VECTOR"
    a = me.attributes.get(name) or me.attributes.new(name, kind, "POINT")
    if kind == "FLOAT":
        a.data.foreach_set("value", values)
    else:
        a.data.foreach_set("vector", values.reshape(-1))


class Graph:
    """A few shader-node shortcuts; arguments are sockets or numbers."""

    def __init__(self, nt):
        self.nt = nt

    def node(self, kind, **props):
        n = self.nt.nodes.new(kind)
        for k, v in props.items():
            setattr(n, k, v)
        return n

    def put(self, sock, val):
        if isinstance(val, (int, float)):
            sock.default_value = val
        elif isinstance(val, tuple):
            sock.default_value = val if len(val) == len(sock.default_value) else (*val, 1.0)
        else:
            self.nt.links.new(val, sock)

    def math(self, op, a, b=0.0, clamp=False):
        n = self.node("ShaderNodeMath", operation=op, use_clamp=clamp)
        self.put(n.inputs[0], a)
        self.put(n.inputs[1], b)
        return n.outputs[0]

    def attr(self, name, out="Fac"):
        return self.node("ShaderNodeAttribute", attribute_name=name).outputs[out]

    def xyz(self, v):
        n = self.node("ShaderNodeSeparateXYZ")
        self.nt.links.new(v, n.inputs[0])
        return n.outputs

    def vec(self, x, y, z=0.0):
        n = self.node("ShaderNodeCombineXYZ")
        for i, c in enumerate((x, y, z)):
            self.put(n.inputs[i], c)
        return n.outputs[0]

    def mix(self, fac, a, b):
        n = self.node("ShaderNodeMix", data_type="RGBA")
        sa = [s for s in n.inputs if s.name == "A" and s.type == "RGBA"][0]
        sb = [s for s in n.inputs if s.name == "B" and s.type == "RGBA"][0]
        self.put(n.inputs["Factor"], fac)
        self.put(sa, a)
        self.put(sb, b)
        return [s for s in n.outputs if s.type == "RGBA"][0]

    def band(self, x, lo, hi):
        return self.math("MULTIPLY", self.math("GREATER_THAN", x, lo), self.math("LESS_THAN", x, hi))

    def all(self, *facs):
        out = facs[0]
        for f in facs[1:]:
            out = self.math("MULTIPLY", out, f)
        return out

    def image(self, path, vector):
        n = self.node("ShaderNodeTexImage", extension="CLIP", interpolation="Cubic")
        n.image = bpy.data.images.load(path)
        self.nt.links.new(vector, n.inputs["Vector"])
        return n.outputs["Color"], n.outputs["Alpha"]

    def shader_out(self):
        out = [n for n in self.nt.nodes if n.type == "OUTPUT_MATERIAL"][0]
        return out.inputs["Surface"]


def principled(name, color, rough, sheen=0.0, bump=0.0, bump_scale=300.0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    m.use_fake_user = True
    p = m.node_tree.nodes["Principled BSDF"]
    p.inputs["Base Color"].default_value = (*color, 1.0)
    p.inputs["Roughness"].default_value = rough
    if sheen:
        p.inputs["Sheen Weight"].default_value = sheen
    if bump:
        g = Graph(m.node_tree)
        tex = g.node("ShaderNodeTexNoise")
        tex.inputs["Scale"].default_value = bump_scale
        tex.inputs["Detail"].default_value = 6.0
        b = g.node("ShaderNodeBump")
        b.inputs["Strength"].default_value = bump
        b.inputs["Distance"].default_value = 0.0005
        m.node_tree.links.new(tex.outputs["Fac"], b.inputs["Height"])
        m.node_tree.links.new(b.outputs["Normal"], p.inputs["Normal"])
    return m


def mesh_object(name, bm, material):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    o = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(o)
    o.data.materials.append(material)
    for p in o.data.polygons:
        p.use_smooth = True
    return o


def frame_of(rig, bone):
    """Posed world frame of a bone: origin (head) and unit axes X, Y (along the bone), Z."""
    pb = rig.pose.bones[bone]
    r = (rig.matrix_world.to_3x3() @ pb.matrix.to_3x3())
    x, y, z = (r.col[i].normalized() for i in range(3))
    return rig.matrix_world @ pb.head, x, y, z


# ---------------------------------------------------------------------------------------
# The uniform: attributes computed in the rest pose, a shader that reads them

def suit_attributes(suit):
    pos, nrm = rest_coords(suit)
    bm = bmesh.new()
    bm.from_mesh(suit.data)
    bm.verts.ensure_lookup_table()
    island = np.full(len(bm.verts), -1)
    k = 0
    for v in bm.verts:
        if island[v.index] >= 0:
            continue
        stack = [v]
        island[v.index] = k
        while stack:
            a = stack.pop()
            for e in a.link_edges:
                b = e.other_vert(a)
                if island[b.index] < 0:
                    island[b.index] = k
                    stack.append(b)
        k += 1
    adj = {}
    for e in bm.edges:
        if e.is_boundary:
            a, b = e.verts[0].index, e.verts[1].index
            adj.setdefault(a, []).append(b)
            adj.setdefault(b, []).append(a)
    bm.free()
    loops, seen = [], set()
    for s in adj:
        if s in seen:
            continue
        comp, stack = [], [s]
        seen.add(s)
        while stack:
            a = stack.pop()
            comp.append(a)
            for b in adj[a]:
                if b not in seen:
                    seen.add(b)
                    stack.append(b)
        loops.append(np.array(comp))
    shirt_i = island[np.argmax(pos[:, 2])]
    pants_i = island[np.argmin(pos[:, 2])]
    for i in range(k):
        sel = island == i
        print("ISLAND", i, "verts", int(sel.sum()), "z", round(pos[sel, 2].min(), 3), round(pos[sel, 2].max(), 3),
              "shirt" if i == shirt_i else "pants" if i == pants_i else "")
    xc = float(pos[:, 0].mean())
    info = []
    for lp in loops:
        p = pos[lp]
        info.append((lp, island[lp[0]], float(p[:, 2].mean()), float(np.abs(p[:, 0] - xc).mean()), len(lp)))
        print("LOOP island", island[lp[0]], "n", len(lp), "z", round(p[:, 2].mean(), 3), "|x|", round(np.abs(p[:, 0] - xc).mean(), 3))
    shirt_loops = sorted([i for i in info if i[1] == shirt_i], key=lambda t: -t[2])
    pants_loops = sorted([i for i in info if i[1] == pants_i], key=lambda t: -t[2])
    collar = shirt_loops[0]
    sleeves = sorted(shirt_loops[1:], key=lambda t: -t[3])[:2]
    waist = pants_loops[0]
    trim = np.concatenate([collar[0]] + [s[0] for s in sleeves])
    shirt = island == shirt_i
    pants = ~shirt
    # distance to the collar and sleeve edges (piping)
    d = np.full(len(pos), 1.0)
    tp = pos[trim]
    for start in range(0, len(pos), 2000):
        chunk = pos[start:start + 2000]
        dd = np.sqrt(((chunk[:, None, :] - tp[None, :, :]) ** 2).sum(-1)).min(1)
        d[start:start + 2000] = dd
    d[pants] = 1.0
    # distance below the trousers' top edge (belt), and the jersey's height above it (tuck)
    wp = pos[waist[0]]
    cx, cy = float(wp[:, 0].mean()), float(wp[:, 1].mean())
    wang = np.arctan2(wp[:, 1] - cy, wp[:, 0] - cx)
    belt = np.full(len(pos), 1.0)
    pv = pos[pants]
    belt[pants] = np.sqrt(((pv[:, None, :] - wp[None, :, :]) ** 2).sum(-1)).min(1)
    tuck = np.full(len(pos), 1.0)
    sv = pos[shirt]
    ang = np.arctan2(sv[:, 1] - cy, sv[:, 0] - cx)
    # the trousers' top edge as a smooth function of the angle around the body: linear between
    # neighbouring edge vertices, then a light circular smoothing
    order = np.argsort(wang)
    wa, wz = wang[order], wp[order, 2]
    grid = np.linspace(-math.pi, math.pi, 361)
    zg = np.interp(grid, np.concatenate([wa - 2 * math.pi, wa, wa + 2 * math.pi]), np.concatenate([wz, wz, wz]))
    kern = np.exp(-0.5 * (np.arange(-12, 13) / 5.0) ** 2)
    kern /= kern.sum()
    zg = np.convolve(np.concatenate([zg[-12:], zg, zg[:12]]), kern, mode="same")[12:-12]
    tuck[shirt] = sv[:, 2] - np.interp(ang, grid, zg)
    set_attr(suit, "rest", pos)
    set_attr(suit, "rest_n", nrm)
    set_attr(suit, "pipe", d)
    set_attr(suit, "belt", belt)
    set_attr(suit, "tuck", tuck)
    cp = pos[collar[0]]
    front = cp[np.argmin(cp[:, 1])]
    geo = {"xc": float(cp[:, 0].mean()), "collar_front_z": float(front[2]), "collar_back_z": float(cp[np.argmax(cp[:, 1]), 2]),
           "waist_front_z": float(wp[np.argmin(wp[:, 1]), 2]), "waist_z": float(wp[:, 2].mean())}
    print("SUIT GEO", {k: round(v, 3) for k, v in geo.items()})
    return geo


def jersey_shader(suit, geo):
    m = suit.material_slots[0].material
    g = Graph(m.node_tree)
    rgb = [n for n in m.node_tree.nodes if n.type == "RGB"][0]
    targets = [l.to_socket for l in rgb.outputs[0].links]
    x, y, z = g.xyz(g.attr("rest", "Vector"))
    nx, ny, nz = g.xyz(g.attr("rest_n", "Vector"))
    back = g.math("GREATER_THAN", ny, 0.3)
    front = g.math("LESS_THAN", ny, -0.3)
    xc = geo["xc"]
    white = (0.80, 0.80, 0.77)
    c = g.mix(g.math("LESS_THAN", g.attr("pipe"), 0.011), white, NAVY)
    # placket: two piped edges down the front from the collar to the belt, buttons between
    ax = g.math("ABSOLUTE", g.math("SUBTRACT", x, xc))
    z_top, z_bot = geo["collar_front_z"], geo["waist_front_z"]
    in_front = g.all(front, g.band(z, z_bot, z_top + 0.005))
    c = g.mix(g.all(in_front, g.band(ax, 0.0155, 0.0205)), c, NAVY)
    dz = g.math("SUBTRACT", g.math("FLOORED_MODULO", g.math("SUBTRACT", z_top - 0.035, z), 0.082), 0.041)
    r = g.math("SQRT", g.math("ADD", g.math("MULTIPLY", ax, ax), g.math("MULTIPLY", dz, dz)))
    buttons = g.all(in_front, g.band(z, z_bot + 0.04, z_top - 0.02), g.band(r, 0.0032, 0.0052))
    c = g.mix(buttons, c, (0.42, 0.42, 0.42))
    if DECALS:
        # number on the back: 0.25 m tall, its top 0.13 m below the back of the collar
        h = 0.25
        w = h * 1.498
        zb = geo["collar_back_z"] - 0.13 - h
        u = g.math("ADD", g.math("DIVIDE", g.math("SUBTRACT", xc, x), w), 0.5)
        v = g.math("DIVIDE", g.math("SUBTRACT", z, zb), h)
        col, alpha = g.image(os.path.join(DECALS, "number.png"), g.vec(u, v))
        c = g.mix(g.math("MULTIPLY", alpha, back), c, col)
        # wordmark across the chest, letters 0.075 m tall
        h2 = 0.075
        w2 = h2 * 6.815
        zb2 = z_top - 0.135
        u2 = g.math("ADD", g.math("DIVIDE", g.math("SUBTRACT", x, xc), w2), 0.5)
        v2 = g.math("DIVIDE", g.math("SUBTRACT", z, zb2), h2)
        col2, alpha2 = g.image(os.path.join(DECALS, "wordmark.png"), g.vec(u2, v2))
        c = g.mix(g.math("MULTIPLY", alpha2, front), c, col2)
    c = g.mix(g.math("LESS_THAN", g.attr("belt"), 0.036), c, NAVY)
    for t in targets:
        m.node_tree.links.new(c, t)
    # the trousers' top was in the shadow of the untucked shirt in the baked ambient occlusion
    ao = m.node_tree.nodes["aoMix"]
    keep = g.math("DIVIDE", g.math("SUBTRACT", g.attr("belt"), 0.04), 0.10, clamp=True)
    m.node_tree.links.new(g.math("MULTIPLY", keep, ao.inputs[0].default_value), ao.inputs[0])
    # the jersey below the trousers' top edge is tucked in: not drawn
    surf = g.shader_out()
    old = surf.links[0].from_socket
    tr = g.node("ShaderNodeBsdfTransparent")
    mixs = g.node("ShaderNodeMixShader")
    m.node_tree.links.new(g.math("GREATER_THAN", g.attr("tuck"), -0.004), mixs.inputs[0])
    m.node_tree.links.new(tr.outputs[0], mixs.inputs[1])
    m.node_tree.links.new(old, mixs.inputs[2])
    m.node_tree.links.new(mixs.outputs[0], surf)


def undershirt(body, rig):
    """A navy long-sleeve undershirt (and socks): everything but the head, neck and hands. The
    cuffs are cut square across the forearm, 0.08 of its length above the wrist."""
    pos, _ = rest_coords(body)
    names = {vg.index: vg.name for vg in body.vertex_groups}
    mw = rig.matrix_world
    fore = {}
    for side in ("l", "r"):
        b = rig.data.bones[f"lowerarm_{side}"]
        fore[side] = (mw @ b.head_local, mw @ b.tail_local)
    headw = np.zeros(len(pos))
    cuff = np.full(len(pos), -1.0)
    for v in body.data.vertices:
        ws = {names[gr.group]: gr.weight for gr in v.groups}
        headw[v.index] = min(1.0, sum(w for k, w in ws.items() if k.startswith(("neck_", "head"))))
        armish = {k: w for k, w in ws.items() if k.startswith(("lowerarm", "hand_", "thumb_", "index_", "middle_", "ring_", "pinky_"))}
        if sum(armish.values()) > 0.3:
            side = max(armish, key=armish.get)[-1]
            e, w = fore[side]
            p = Vector(pos[v.index])
            cuff[v.index] = (p - e).dot(w - e) / (w - e).length_squared - 0.92
    set_attr(body, "headw", headw)
    set_attr(body, "cuff", cuff)
    m = [s.material for s in body.material_slots if s.material and s.material.name.endswith("body")][0]
    g = Graph(m.node_tree)
    surf = g.shader_out()
    old = surf.links[0].from_socket
    p = g.node("ShaderNodeBsdfPrincipled")
    p.inputs["Base Color"].default_value = (*NAVY, 1.0)
    p.inputs["Roughness"].default_value = 0.6
    p.inputs["Sheen Weight"].default_value = 0.4
    mixs = g.node("ShaderNodeMixShader")
    fac = g.math("MULTIPLY", g.math("LESS_THAN", g.attr("headw"), 0.45), g.math("LESS_THAN", g.attr("cuff"), 0.0))
    m.node_tree.links.new(fac, mixs.inputs[0])
    m.node_tree.links.new(old, mixs.inputs[1])
    m.node_tree.links.new(p.outputs[0], mixs.inputs[2])
    m.node_tree.links.new(mixs.outputs[0], surf)
    print("UNDERSHIRT cuff verts", int((cuff > -1).sum()))


# ---------------------------------------------------------------------------------------
# Cap fitted to the head (built after posing, in world space)

def build_cap(rig, body):
    O, X, Y, Z = frame_of(rig, "head")
    R = Matrix((X, Y, Z)).transposed()  # columns X, Y, Z: head frame -> world
    hg = body.vertex_groups["head"].index
    idx = [v.index for v in body.data.vertices if any(gr.group == hg and gr.weight > 0.6 for gr in v.groups)]
    P = posed_coords(body)[idx]
    loc = (P - np.array(O)) @ np.array(R)  # head-frame coordinates
    top = loc[:, 1].max()
    upper = loc[loc[:, 1] > top - 0.10]
    xmin, xmax = upper[:, 0].min(), upper[:, 0].max()
    zmin, zmax = upper[:, 2].min(), upper[:, 2].max()
    cx, cz = (xmin + xmax) / 2, (zmin + zmax) / 2
    rx, rz = (xmax - xmin) / 2 + 0.014, (zmax - zmin) / 2 + 0.014
    ry = 0.118
    cy = top + 0.016 - ry
    print("HEAD top", round(top, 3), "x", round(xmin, 3), round(xmax, 3), "z", round(zmin, 3), round(zmax, 3))
    F = Vector((cx, top - 0.080, cz + rz))
    B = Vector((cx, top - 0.112, cz - rz))
    n = Vector((1, 0, 0)).cross(B - F).normalized()
    if n.y < 0:
        n = -n

    def rim_y(xx, zz):
        # the rim plane through F with normal n: n.x*(x-Fx) + n.y*(y-Fy) + n.z*(z-Fz) = 0
        return F.y - (n.x * (xx - F.x) + n.z * (zz - F.z)) / n.y

    def to_world(p):
        return O + R @ Vector(p)

    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=64, v_segments=32, radius=1.0)
    for v in bm.verts:
        v.co = Vector((cx + v.co.x * rx, cy + v.co.z * ry, cz + v.co.y * rz))
    kill = [v for v in bm.verts if (v.co - F).dot(n) < -0.002]
    bmesh.ops.delete(bm, geom=kill, context="VERTS")
    capuv = []
    for v in bm.verts:
        capuv.append((v.co.x - cx, v.co.y - cy, v.co.z - cz))
        v.co = to_world(v.co)
    crown = mesh_object("cap_crown", bm, cap_material())
    set_attr(crown, "capuv", capuv)
    sol = crown.modifiers.new("thick", "SOLIDIFY")
    sol.thickness = 0.003
    sol.offset = -1.0
    # brim: from the front half of the rim outwards, 0.075 m at the centre, drooping at the sides
    bm = bmesh.new()
    na, nt = 41, 8
    grid = []
    for i in range(na):
        a = math.radians(-78 + 156 * i / (na - 1))
        bx, bz = cx + rx * math.sin(a), cz + rz * math.cos(a)
        out = Vector((math.sin(a) / rx, 0.0, math.cos(a) / rz)).normalized()
        length = 0.075 * max(0.0, math.cos(a)) ** 0.75 + 0.004
        row = []
        for j in range(nt):
            t = j / (nt - 1)
            p = Vector((bx, rim_y(bx, bz) - 0.002, bz)) + out * (t * length)
            p.y -= t * length * math.tan(math.radians(12)) + 0.02 * (math.sin(a) ** 2) * t
            row.append(bm.verts.new(to_world(p)))
        grid.append(row)
    for i in range(na - 1):
        for j in range(nt - 1):
            bm.faces.new((grid[i][j], grid[i + 1][j], grid[i + 1][j + 1], grid[i][j + 1]))
    brim = mesh_object("cap_brim", bm, M["cap_plain"])
    sol = brim.modifiers.new("thick", "SOLIDIFY")
    sol.thickness = 0.004
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=8, radius=0.0075)
    for v in bm.verts:
        v.co = to_world(Vector((cx + v.co.x, cy + ry + v.co.y * 0.5, cz + v.co.z)))
    mesh_object("cap_button", bm, M["cap_plain"])
    return [crown, brim]


def cap_material():
    m = principled("cap2", NAVY, 0.75, sheen=0.5, bump=0.2, bump_scale=900)
    M["cap2"] = m
    M["cap_plain"] = principled("cap_plain", NAVY, 0.7, sheen=0.5, bump=0.2, bump_scale=900)
    g = Graph(m.node_tree)
    p = m.node_tree.nodes["Principled BSDF"]
    x, y, z = g.xyz(g.attr("capuv", "Vector"))
    # six panel seams from the button down
    angle = g.math("ARCTAN2", x, z)
    seam = g.all(g.math("LESS_THAN", g.math("ABSOLUTE", g.math("SINE", g.math("MULTIPLY", angle, 3.0))), 0.035),
                 g.math("GREATER_THAN", y, -0.06))
    c = g.mix(seam, NAVY, (0.006, 0.01, 0.035))
    if DECALS:
        h = 0.055
        u = g.math("ADD", g.math("DIVIDE", x, h * 1.299), 0.5)
        v = g.math("DIVIDE", g.math("SUBTRACT", y, -0.035), h)
        col, alpha = g.image(os.path.join(DECALS, "caplogo.png"), g.vec(u, v))
        c = g.mix(g.math("MULTIPLY", alpha, g.math("GREATER_THAN", z, 0.03)), c, col)
    m.node_tree.links.new(c, p.inputs["Base Color"])
    return m


# ---------------------------------------------------------------------------------------
# Ball with stitched seams, held in a four-seam grip

def build_ball(center, rot):
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=48, v_segments=24, radius=BALL_R)
    for v in bm.verts:
        v.co = center + rot @ v.co
    ball = mesh_object("ball2", bm, principled("ball_leather", (0.86, 0.85, 0.80), 0.5, bump=0.08, bump_scale=600))
    # the seam curve on the unit sphere (exactly on it when c = 2*sqrt(a*b), a + b = 1)
    b_ = 0.16
    a_ = 1.0 - b_
    c_ = 2.0 * math.sqrt(a_ * b_)

    def seam(t):
        return Vector((a_ * math.cos(t) + b_ * math.cos(3 * t), a_ * math.sin(t) - b_ * math.sin(3 * t), c_ * math.sin(2 * t)))

    bm = bmesh.new()
    n = 108
    for k in range(n):
        t = 2 * math.pi * k / n
        p = seam(t).normalized()
        tan = (seam(t + 1e-3) - seam(t - 1e-3)).normalized()
        side = p.cross(tan).normalized()
        for sgn in (1.0, -1.0):
            a = p * (BALL_R + 0.0004) + side * (sgn * 0.0034)
            b = p * (BALL_R + 0.0006) + side * (sgn * 0.0009) + tan * 0.0016
            mid = (a + b) / 2
            length = (b - a).length
            d = (b - a).normalized()
            q = Vector((0, 0, 1)).rotation_difference(d).to_matrix().to_4x4()
            mat = Matrix.Translation(center + rot @ mid) @ (rot.to_4x4() @ q) @ Matrix.Diagonal((0.00055, 0.00055, length / 2, 1.0))
            bmesh.ops.create_uvsphere(bm, u_segments=6, v_segments=4, radius=1.0, matrix=mat)
    stitches = mesh_object("ball_stitches", bm, principled("stitch", (0.55, 0.03, 0.03), 0.6))
    return [ball, stitches]


def finger_bones(f, side):
    return [f"{f}_0{i}_{side}" for i in (1, 2, 3)]


def save_rot(rig, names):
    return {n: rig.pose.bones[n].rotation_quaternion.copy() for n in names}


def load_rot(rig, saved):
    # Rotations only are saved; setting a bone's matrix many times lets rounding creep into its
    # scale and location, so those go back to rest too.
    for n, q in saved.items():
        pb = rig.pose.bones[n]
        pb.rotation_quaternion = q
        pb.scale = (1, 1, 1)
        pb.location = (0, 0, 0)
    upd()


def curl(rig, f, side, ang, ratios=(1.0, 1.15, 0.8)):
    for b, k in zip(finger_bones(f, side), ratios):
        rotate_x(rig, b, ang * k * FINGER_SIGN)


def seg_dist(p, a, b):
    ab = b - a
    t = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
    return (a + ab * t - p).length


def grip(rig):
    """Four-seam grip: index and middle fingertips over the top of the ball, the thumb under it,
    ring finger along the side, pinky tucked, a gap between the ball and the palm.
    Returns the ball centre and an orientation for its seams."""
    fingers = ("index", "middle", "ring", "pinky")
    names = [b for f in fingers + ("thumb",) for b in finger_bones(f, "r")]
    meshes = [o for o in bpy.data.objects if o.type == "MESH"]
    for o in meshes:
        o.hide_viewport = True
    for n in names:
        rig.pose.bones[n].rotation_quaternion = (1, 0, 0, 0)
        rig.pose.bones[n].scale = (1, 1, 1)
        rig.pose.bones[n].location = (0, 0, 0)
    upd()
    base = save_rot(rig, names)
    O, X, Y, Z = frame_of(rig, "hand_r")
    hand_dir = (pb_world(rig, "middle_01_r")[0] - O).normalized()
    tip0 = pb_world(rig, "middle_03_r")[1]
    curl(rig, "middle", "r", 40.0)
    tip1 = pb_world(rig, "middle_03_r")[1]
    load_rot(rig, base)
    palm = tip1 - tip0
    palm = (palm - hand_dir * palm.dot(hand_dir)).normalized()
    for f, a in (("index", 18.0), ("middle", 15.0), ("ring", 30.0), ("pinky", 55.0)):
        rotate_x(rig, f"{f}_01_r", a * FINGER_SIGN)
    pip = pb_world(rig, "middle_02_r")[0]
    fdir = (pb_world(rig, "middle_02_r")[1] - pip).normalized()
    pperp = (palm - fdir * palm.dot(fdir)).normalized()
    C = pip + fdir * 0.012 + pperp * (BALL_R + 0.0105)
    pad = 0.0085
    angles = {}
    for f, extra in (("index", 0.0), ("middle", 0.0), ("ring", 0.001)):
        b2, b3 = f"{f}_02_r", f"{f}_03_r"
        q2, q3 = rig.pose.bones[b2].rotation_quaternion.copy(), rig.pose.bones[b3].rotation_quaternion.copy()
        found = None
        for a in range(0, 121, 2):
            rig.pose.bones[b2].rotation_quaternion = q2
            rig.pose.bones[b3].rotation_quaternion = q3
            upd()
            rotate_x(rig, b2, a * FINGER_SIGN)
            rotate_x(rig, b3, 0.7 * a * FINGER_SIGN)
            tip = pb_world(rig, b3)[1]
            if (tip - C).length <= BALL_R + pad + extra:
                found = a
                break
        a = found if found is not None else 70
        angles[f] = a
        rig.pose.bones[b2].rotation_quaternion = q2
        rig.pose.bones[b3].rotation_quaternion = q3
        upd()
        rotate_x(rig, b2, a * FINGER_SIGN)
        rotate_x(rig, b3, 0.7 * a * FINGER_SIGN)
    rotate_x(rig, "pinky_02_r", 80.0 * FINGER_SIGN)
    rotate_x(rig, "pinky_03_r", 50.0 * FINGER_SIGN)
    worst = min(seg_dist(C, *pb_world(rig, b)) for f in ("index", "middle", "ring", "pinky") for b in finger_bones(f, "r"))
    print("GRIP curl", angles, "closest finger axis to ball centre", round(worst, 4), "ball r", BALL_R)
    pads = (pb_world(rig, "index_03_r")[1] + pb_world(rig, "middle_03_r")[1]) / 2
    target = C - (pads - C).normalized() * (BALL_R + pad)
    tb = finger_bones("thumb", "r")
    tbase = save_rot(rig, tb)

    def thumb(a1, a2, a3):
        load_rot(rig, tbase)
        rotate_about_head(rig, tb[0], Quaternion(hand_dir, math.radians(a1)))
        rotate_about_head(rig, tb[0], Quaternion(palm, math.radians(a2)))
        rotate_x(rig, tb[1], a3 * FINGER_SIGN)
        rotate_x(rig, tb[2], 0.8 * a3 * FINGER_SIGN)
        tip = pb_world(rig, tb[2])[1]
        pen = max(0.0, (BALL_R + 0.0075) - min(seg_dist(C, *pb_world(rig, b)) for b in tb))
        return (tip - target).length + 3.0 * pen

    # within the thumb's own range of motion (about 40 deg each way at its base)
    best = min((thumb(a1, a2, a3), (a1, a2, a3)) for a1 in range(-40, 41, 10) for a2 in range(-40, 41, 10) for a3 in range(0, 81, 10))
    c1, c2, c3 = best[1]
    best = min((thumb(a1, a2, a3), (a1, a2, a3)) for a1 in range(max(-40, c1 - 5), min(40, c1 + 5) + 1, 5)
               for a2 in range(max(-40, c2 - 5), min(40, c2 + 5) + 1, 5) for a3 in range(max(0, c3 - 5), min(80, c3 + 5) + 1, 5))
    thumb(*best[1])
    print("THUMB", best[1], "miss", round(best[0], 4))
    for o in meshes:
        o.hide_viewport = False
    upd()
    rot = Matrix((hand_dir, palm.cross(hand_dir), palm)).transposed()
    return C, rot


# ---------------------------------------------------------------------------------------
# Glove built around the posed glove hand

def build_glove(rig):
    parts = []
    mat_ = principled("leather", (0.30, 0.14, 0.055), 0.42, bump=0.25, bump_scale=500)
    O, X, Y, Z = frame_of(rig, "hand_l")
    hand_dir = (pb_world(rig, "middle_01_l")[0] - O).normalized()
    tip0 = pb_world(rig, "middle_03_l")[1]
    saved = save_rot(rig, finger_bones("middle", "l"))
    curl(rig, "middle", "l", 40.0)
    tip1 = pb_world(rig, "middle_03_l")[1]
    load_rot(rig, saved)
    palm = (tip1 - tip0)
    palm = (palm - hand_dir * palm.dot(hand_dir)).normalized()
    lateral = hand_dir.cross(palm).normalized()
    # fingers inside a glove lie side by side: turn index, ring and pinky parallel to the middle
    mdir = pb_world(rig, "middle_03_l")[1] - pb_world(rig, "middle_01_l")[0]
    mdir = (mdir - palm * mdir.dot(palm)).normalized()
    for f in ("index", "ring", "pinky"):
        h0 = pb_world(rig, f"{f}_01_l")[0]
        fd = pb_world(rig, f"{f}_03_l")[1] - h0
        fd = (fd - palm * fd.dot(palm)).normalized()
        ang = math.atan2(palm.dot(fd.cross(mdir)), fd.dot(mdir))
        rotate_about_head(rig, f"{f}_01_l", Quaternion(palm, 0.9 * ang))

    def capsule(a, b, r_w, r_t, name):
        d = b - a
        y = d.normalized()
        x = (lateral - y * lateral.dot(y)).normalized()
        z = x.cross(y)
        rot = Matrix((x, y, z)).transposed()
        o = ellipsoid(name, (a + b) / 2, rot, (r_w, d.length / 2 + r_w * 0.6, r_t), mat_, segments=24)
        parts.append(o)

    tips = []
    for f, rw, ext in (("index", 0.026, 0.065), ("middle", 0.026, 0.068), ("ring", 0.025, 0.063), ("pinky", 0.024, 0.056)):
        a = pb_world(rig, f"{f}_01_l")[0]
        b = pb_world(rig, f"{f}_03_l")[1]
        d = (b - a).normalized()
        capsule(a - d * 0.02, b + d * ext, rw, 0.0125, "glove_" + f)
        tips.append((b + d * (ext - 0.006), d))
    a = pb_world(rig, "thumb_01_l")[0]
    b = pb_world(rig, "thumb_03_l")[1]
    d = (b - a).normalized()
    capsule(a, b + d * 0.06, 0.021, 0.0165, "glove_thumb")
    hc = O + hand_dir * 0.055
    rot = Matrix((lateral, hand_dir, palm)).transposed()
    # the back of the glove: one leather panel over all four finger stalls
    fb = [pb_world(rig, f"{f}_01_l")[0] for f in ("index", "middle", "ring", "pinky")]
    ft = [t for t, _ in tips]
    bc = (sum(fb, Vector()) + sum(ft, Vector())) / 8 - palm * 0.006
    span = (ft[0] - ft[3]).length / 2 + 0.02
    along = ((sum(ft, Vector()) - sum(fb, Vector())) / 4).length / 2 + 0.01
    parts.append(ellipsoid("glove_back", bc, rot, (span, along, 0.011), mat_, segments=32))
    parts.append(ellipsoid("glove_heel", hc, rot, (0.068, 0.085, 0.027), mat_, segments=32))
    # web between the thumb and the index finger
    t_tip = b + d * 0.05
    i_mid = pb_world(rig, "index_03_l")[1] + hand_dir * 0.03
    wc = (t_tip + i_mid) / 2
    gap = i_mid - t_tip
    y = gap.normalized()
    x = palm.cross(y).normalized()
    z = x.cross(y)
    parts.append(ellipsoid("glove_web", wc, Matrix((x, y, z)).transposed(), (0.05, gap.length / 2 + 0.015, 0.010), mat_, segments=24))
    bpy.ops.object.select_all(action="DESELECT")
    for o in parts:
        o.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.object.join()
    glove = bpy.context.active_object
    glove.name = "glove2"
    rm = glove.modifiers.new("remesh", "REMESH")
    rm.mode = "VOXEL"
    rm.voxel_size = 0.004
    bpy.ops.object.modifier_apply(modifier="remesh")
    sm = glove.modifiers.new("smooth", "SMOOTH")
    sm.factor = 0.8
    sm.iterations = 12
    bpy.ops.object.modifier_apply(modifier="smooth")
    bpy.ops.object.shade_smooth()
    # rawhide laces: a zigzag across the fingertips and along the top of the web
    lace = principled("lace", (0.10, 0.045, 0.02), 0.5)
    bm = bmesh.new()

    def lace_seg(a, b, r=0.0023):
        d = b - a
        q = Vector((0, 0, 1)).rotation_difference(d.normalized()).to_matrix().to_4x4()
        mat = Matrix.Translation((a + b) / 2) @ q @ Matrix.Diagonal((r, r, d.length / 2 + r, 1.0))
        bmesh.ops.create_uvsphere(bm, u_segments=8, v_segments=6, radius=1.0, matrix=mat)

    for (p0, d0), (p1, d1) in zip(tips[:-1], tips[1:]):
        for k in range(3):
            t0, t1 = k / 3, (k + 1) / 3
            q0 = p0.lerp(p1, t0) + palm * (0.013 if k % 2 == 0 else -0.013)
            q1 = p0.lerp(p1, t1) + palm * (-0.013 if k % 2 == 0 else 0.013)
            lace_seg(q0, q1)
    w0, w1 = t_tip, i_mid
    for k in range(5):
        a = w0.lerp(w1, k / 5) + palm * (0.011 if k % 2 == 0 else -0.011)
        b = w0.lerp(w1, (k + 1) / 5) + palm * (-0.011 if k % 2 == 0 else 0.011)
        lace_seg(a, b)
    mesh_object("glove_laces", bm, lace)
    print("GLOVE verts", len(glove.data.vertices))
    return glove


# ---------------------------------------------------------------------------------------

def main_refined():
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
    FINGER_SIGN = float(os.environ.get("FINGER_SIGN", "1"))
    if not os.environ.get("NO_HAIR"):
        HumanService.add_mhclo_asset(os.path.join(DATA, "hair/short02/short02.mhclo"), body, asset_type="Hair", material_type="MAKESKIN")
    suit = [o for o in rig.children if "casualsuit" in o.name][0]
    geo = suit_attributes(suit)
    jersey_shader(suit, geo)
    undershirt(body, rig)
    pose(rig, POSE)
    C, rot = grip(rig)
    build_ball(C, rot)
    build_cap(rig, body)
    build_glove(rig)
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
    sh, _ = pb_world(rig, "upperarm_r")
    el, _ = pb_world(rig, "lowerarm_r")
    aim = (sh + el) * 0.5
    cams["shoulder"] = camera("shoulder", aim + Vector((-1.0, -1.1, 0.55)), aim, 55, (900, 900))
    hand = pb_world(rig, "hand_r")[1]
    cams["hand"] = camera("hand", hand + Vector((0.15, -0.55, 0.10)), hand, 70, (900, 900))
    glove_c = pb_world(rig, "hand_l")[1]
    cams["glove"] = camera("glove", glove_c + Vector((0.25, -0.6, 0.15)), glove_c, 60, (900, 900))
    for view in VIEWS:
        if view in cams:
            render(cams[view], os.path.join(OUT, f"{POSE}_{view}_C_refined.png"))


main_refined()
