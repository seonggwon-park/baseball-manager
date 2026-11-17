"""Statcast balls in play and fouls with bat tracking and launch data, as rows for
`bm contacts --invert` (`res`: X in play, F foul).
Velocity at the front of the plate from the nine-parameter fit; transverse spin from the
fitted acceleration with the engine's lift model (Nathan's C_L(S), mean game-time air).
usage: export_contacts.py SAMPLE.npz OUT.csv   (SAMPLE.npz from load.py)"""
import sys
import numpy as np

FT = 0.3048
MPH = 0.44704
d = np.load(sys.argv[1], allow_pickle=True)
d = {k: d[k] for k in d.files}
ty = d['type']; bs = d['bat_speed']
ok = ((ty == 'X') | (d['description'] == 'foul')) & np.isfinite(bs) & (bs >= 60) & np.isfinite(d['attack_angle']) & np.isfinite(d['attack_direction']) \
    & np.isfinite(d['swing_path_tilt']) & np.isfinite(d['launch_angle']) & np.isfinite(d['launch_speed']) & np.isfinite(d['vy0'])
idx = np.where(ok)[0]
vx0, vy0, vz0 = d['vx0'][idx], d['vy0'][idx], d['vz0'][idx]
ax, ay, az = d['ax'][idx], d['ay'][idx], d['az'][idx]
y0, yf = 50.0, 17.0 / 12.0
t = (-vy0 - np.sqrt(vy0 ** 2 - 2 * ay * (y0 - yf))) / ay
vel = np.stack([vx0 + ax * t, vy0 + ay * t, vz0 + az * t], 1) * FT
vmid = np.stack([vx0 + ax * t / 2, vy0 + ay * t / 2, vz0 + az * t / 2], 1) * FT
acc = np.stack([ax, ay, az], 1) * FT
g = np.array([0, 0, -9.80665])
vhat = vmid / np.linalg.norm(vmid, axis=1, keepdims=True)
am = acc - g
am = am - (am * vhat).sum(1, keepdims=True) * vhat
amag = np.linalg.norm(am, axis=1)
rho, r, m = 1.155, 0.036888, 0.145291
k = 0.5 * rho * np.pi * r * r / m
cl0, cl1, cl2 = 0.583, 2.333, 1.12
v = np.linalg.norm(vmid, axis=1)
den = r * (k * cl2 * v * v - amag * cl1)
omega = np.where(den > 0, amag * cl0 * v / np.maximum(den, 1e-9), np.nan)
what = np.cross(vhat, am / np.maximum(amag, 1e-9)[:, None])
spin = what * omega[:, None]
stand = d['stand'][idx]
hx, hy = d['hc_x'][idx], d['hc_y'][idx]
spray = np.radians(np.arctan2(hx - 125.42, 198.27 - hy) * 180 / np.pi)
pull = np.where(stand == 'R', -1.0, 1.0)
iy = d['intercept_ball_minus_batter_pos_y_inches'][idx]
depth = (iy - np.nanmean(iy)) * 0.0254
cols = {
    'hand': stand, 'kind': d['pitch_type'][idx], 'px': d['plate_x'][idx] * FT, 'pz': d['plate_z'][idx] * FT,
    'zb': d['sz_bot'][idx] * FT, 'zt': d['sz_top'][idx] * FT,
    'vx': vel[:, 0], 'vy': vel[:, 1], 'vz': vel[:, 2], 'wx': spin[:, 0], 'wy': spin[:, 1], 'wz': spin[:, 2],
    'bs': bs[idx] * MPH, 'aa': np.radians(d['attack_angle'][idx]), 'dir': -np.radians(d['attack_direction'][idx]),
    'tilt': np.radians(d['swing_path_tilt'][idx]), 'depth': depth, 'la': np.radians(d['launch_angle'][idx]),
    'ev': d['launch_speed'][idx] * MPH, 'spray': spray * pull, 'bb': d['bb_type'][idx], 'batter': d['batter'][idx].astype(int),
    'events': d['events'][idx], 'res': np.where(ty[idx] == 'X', 'X', 'F'), 'release_spin': d['release_spin_rate'][idx], 'spin_axis': d['spin_axis'][idx],
}
# Spin-induced movement over the whole flight, as the engine defines it: deviation at the
# front of the plate from a drag-free, spinless parabola leaving the release point.
yr = 60.5 - d['release_extension'][idx]
tr = (-vy0 - np.sqrt(vy0 ** 2 - 2 * ay * (y0 - yr))) / ay  # time from y0 back to release (negative)
tflight = t - tr
cols['move_z'] = 0.5 * (az + 32.174) * tflight ** 2 * FT
cols['move_x'] = 0.5 * ax * tflight ** 2 * FT
cols['time'] = tflight
names = list(cols)
with open(sys.argv[2], 'w') as f:
    f.write(','.join(names) + '\n')
    for i in range(len(idx)):
        f.write(','.join(('%.6g' % cols[n][i]) if isinstance(cols[n][i], (float, np.floating)) else str(cols[n][i]) for n in names) + '\n')
rpm = omega * 60 / (2 * np.pi)
kinds = d['pitch_type'][idx]
for kd in ['FF', 'SI', 'FC', 'SL', 'ST', 'CU', 'CH', 'FS']:
    s = kinds == kd
    print(kd, s.sum(), 'transverse spin %.0f rpm, release spin %.0f rpm, ratio %.2f' % (np.nanmean(rpm[s]), np.nanmean(d['release_spin_rate'][idx][s]), np.nanmean(rpm[s]) / np.nanmean(d['release_spin_rate'][idx][s])))
print(len(idx), 'rows')
