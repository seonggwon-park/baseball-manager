import numpy as np
import pytest

import bm_gym


def test_names_and_sizes():
    assert len(bm_gym.PITCH_OBSERVATION_NAMES) == bm_gym.N_PITCH_OBS
    assert len(bm_gym.PITCH_ACTION_NAMES) == bm_gym.N_PITCH_ACTIONS == 24
    assert bm_gym.PITCH_ACTION_NAMES[0] == "four_seam+attack"


def play(env, seed, pick):
    obs, info = env.reset(seed=seed)
    total, called = 0.0, 0
    while True:
        mask = np.asarray(env.action_mask(), dtype=bool)
        action = pick(mask)
        obs, reward, terminated, truncated, info = env.step(action)
        # The call is thrown: the pitch type of the action.
        if info["thrown"] is not None:
            assert info["thrown"][0] == bm_gym.PITCH_ACTION_NAMES[action].split("+")[0]
        total += reward
        called += 1
        assert not truncated
        if terminated:
            return total, called, info


def test_calls_are_thrown_and_rewards_are_runs_for_the_fielders():
    env = bm_gym.PitchingEnv(seed=2, side=0)
    rng = np.random.default_rng(0)
    total, called, info = play(env, 4, lambda m: int(rng.choice(np.flatnonzero(m))))
    assert 80 <= called < 300
    # Over a game the rewards come to about minus the runs allowed (plus about half a run of
    # expectancy for each half inning pitched).
    allowed = info["score"][1]
    assert -allowed - 1.0 < total < -allowed + 6.0


def test_same_seed_same_game_and_bad_arguments():
    env = bm_gym.PitchingEnv(seed=0, side=1, perturb=1.0)
    first = lambda m: int(np.flatnonzero(m)[0])
    assert play(env, 9, first) == play(env, 9, first)
    with pytest.raises(ValueError):
        bm_gym.PitchingEnv(adapt=2.0)
    with pytest.raises(ValueError):
        env.step(bm_gym.N_PITCH_ACTIONS)


def test_gymnasium_pitching_wrapper_passes_the_checker():
    gym = pytest.importorskip("gymnasium")
    from gymnasium.utils.env_checker import check_env

    env = bm_gym.make_pitching(seed=1)
    check_env(env, skip_render_check=True)
    obs, info = env.reset(seed=3)
    assert obs.shape == (bm_gym.N_PITCH_OBS,) and info["action_mask"].shape == (bm_gym.N_PITCH_ACTIONS,)
    made = gym.make("BaseballPitching-v0")
    made.reset(seed=0)
    mask = made.unwrapped.action_masks()
    made.step(int(np.flatnonzero(mask)[0]))
