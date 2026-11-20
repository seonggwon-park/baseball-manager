"""Reinforcement-learning environment on the Baseball Manager match engine.

The agent manages one club's offence: before every pitch to one of its hitters it picks a
batting approach and a steal order. The engine's AI managers run everything else.

    import bm_gym
    env = bm_gym.make()            # Gymnasium environment (needs gymnasium)
    raw = bm_gym.ManagerEnv(seed=1)  # the engine binding alone (numpy-free lists)

Results learned here describe the engine, not baseball: see docs/rl.md.
"""

from ._core import N_ACTIONS, N_OBS, ManagerEnv

OBSERVATION_NAMES = ManagerEnv.observation_names()
ACTION_NAMES = ManagerEnv.action_names()

__all__ = ["ManagerEnv", "N_ACTIONS", "N_OBS", "OBSERVATION_NAMES", "ACTION_NAMES", "make"]


def make(**kwargs):
    """The Gymnasium environment (``BaseballManagerEnv``); keyword arguments go to it."""
    from .env import BaseballManagerEnv

    return BaseballManagerEnv(**kwargs)


try:  # Register with Gymnasium when it is installed.
    import gymnasium as _gym

    if "BaseballManager-v0" not in _gym.registry:
        _gym.register(id="BaseballManager-v0", entry_point="bm_gym.env:BaseballManagerEnv")
except ImportError:  # pragma: no cover - gymnasium is optional
    pass
