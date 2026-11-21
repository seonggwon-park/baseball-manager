import math

import numpy as np
import pytest

import bm_gym


def play(env, policy, seed):
    obs, info = env.reset(seed=seed)
    start = info["win_expectancy"]
    total, steps = 0.0, 0
    while True:
        mask = np.asarray(env.action_mask(), dtype=bool) if isinstance(env, bm_gym.ManagerEnv) else info["action_mask"]
        obs, reward, terminated, truncated, info = env.step(policy(mask))
        total += reward
        steps += 1
        assert not truncated
        if terminated:
            return start, total, steps, info


def test_names_and_sizes():
    assert len(bm_gym.OBSERVATION_NAMES) == bm_gym.N_OBS
    assert len(bm_gym.ACTION_NAMES) == bm_gym.N_ACTIONS == 15
    assert bm_gym.ACTION_NAMES[0] == "hit+auto"


def test_raw_episode_rewards_add_up_to_the_result():
    env = bm_gym.ManagerEnv(seed=3)
    for seed in range(4):
        start, total, steps, info = play(env, lambda m: 0, seed)
        assert info["won"] in (True, False)
        assert math.isclose(total, float(info["won"]) - start, abs_tol=1e-9)
        assert 60 <= steps < 400


def test_reset_with_a_seed_repeats_the_game():
    env = bm_gym.ManagerEnv(seed=0, side=1)
    policy = lambda m: int(np.flatnonzero(m)[-1])  # the last sensible action
    a = play(env, policy, 42)
    b = play(env, policy, 42)
    assert a == b


def test_bad_arguments_raise():
    with pytest.raises(ValueError):
        bm_gym.ManagerEnv(reward="runs")
    with pytest.raises(ValueError):
        bm_gym.ManagerEnv(side=2)
    env = bm_gym.ManagerEnv()
    with pytest.raises(ValueError):
        env.step(0)  # no game yet
    env.reset(seed=1)
    with pytest.raises(ValueError):
        env.step(bm_gym.N_ACTIONS)


def test_copy_is_independent():
    env = bm_gym.ManagerEnv(seed=5, side=0)
    env.reset(seed=9)
    for _ in range(20):
        env.step(0)
    snap = env.copy()
    before = env.step(0)[4]
    after = snap.step(0)[4]
    # Same state and the same random numbers: the copy plays the same pitch.
    assert before == after
    snap.finish_with_ai()
    # Finishing the copy leaves the original in its game.
    _, _, terminated, _, _ = env.step(0)
    assert isinstance(terminated, bool)


def test_gymnasium_wrapper_passes_the_checker():
    gym = pytest.importorskip("gymnasium")
    from gymnasium.utils.env_checker import check_env

    env = bm_gym.make(seed=1)
    check_env(env, skip_render_check=True)
    obs, info = env.reset(seed=3)
    assert obs.dtype == np.float32 and obs.shape == (bm_gym.N_OBS,)
    assert info["action_mask"].dtype == bool and info["action_mask"].shape == (bm_gym.N_ACTIONS,)
    assert np.array_equal(env.action_masks(), info["action_mask"])
    made = gym.make("BaseballManager-v0")
    made.reset(seed=0)
    made.step(made.action_space.sample())
