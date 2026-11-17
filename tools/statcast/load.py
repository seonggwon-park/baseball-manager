"""Load the downloaded Statcast CSV files (DIR/statcast/*.csv) into column arrays, cached
as DIR/sample.npz. usage: load.py DIR"""
import csv, glob, os, sys
import numpy as np

NUM = ['release_speed','release_pos_x','release_pos_z','release_pos_y','pfx_x','pfx_z','plate_x','plate_z','vx0','vy0','vz0','ax','ay','az',
       'sz_top','sz_bot','hit_distance_sc','launch_speed','launch_angle','release_spin_rate','release_extension','spin_axis','hc_x','hc_y',
       'bat_speed','swing_length','miss_distance','arm_angle','attack_angle','attack_direction','swing_path_tilt',
       'intercept_ball_minus_batter_pos_x_inches','intercept_ball_minus_batter_pos_y_inches','balls','strikes','zone','launch_speed_angle',
       'outs_when_up','inning','at_bat_number','pitch_number','game_pk','pitcher','batter','effective_speed','api_break_z_with_gravity','api_break_x_arm',
       'estimated_ba_using_speedangle','on_1b','on_2b','on_3b']
STR = ['pitch_type','pitch_name','events','description','stand','p_throws','bb_type','type','inning_topbot','game_date','if_fielding_alignment','hit_location']

def load(data_dir):
    cache = os.path.join(data_dir, 'sample.npz')
    if os.path.exists(cache):
        z = np.load(cache, allow_pickle=True)
        return {k: z[k] for k in z.files}
    cols = {k: [] for k in NUM + STR}
    for fn in sorted(glob.glob(os.path.join(data_dir, 'statcast', '*.csv'))):
        with open(fn, encoding='utf-8-sig') as f:
            for row in csv.DictReader(f):
                for k in NUM:
                    v = row.get(k, '')
                    try: cols[k].append(float(v))
                    except (TypeError, ValueError): cols[k].append(np.nan)
                for k in STR:
                    cols[k].append(row.get(k, '') or '')
    out = {k: np.array(cols[k], dtype=float) for k in NUM}
    out.update({k: np.array(cols[k], dtype=object) for k in STR})
    np.savez_compressed(cache, **out)
    return out

if __name__ == '__main__':
    d = load(sys.argv[1])
    print({k: len(v) for k, v in list(d.items())[:3]})
