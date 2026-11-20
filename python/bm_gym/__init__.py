"""Reinforcement-learning environments on the Baseball Manager match engine.

``ManagerEnv``: the agent manages one club's offence (before every pitch to one of its
hitters, a batting approach and a steal order). ``PitchingEnv``: the agent calls one club's
pitches (pitch type and intent). The engine's AI managers run everything else.

    import bm_gym
    env = bm_gym.make()              # Gymnasium offence environment (needs gymnasium)
    pit = bm_gym.make_pitching()     # Gymnasium pitching environment
    raw = bm_gym.ManagerEnv(seed=1)  # the engine binding alone (numpy-free lists)

Results learned here describe the engine, not baseball: see docs/rl.md.
"""

from ._core import N_ACTIONS, N_OBS, N_PITCH_ACTIONS, N_PITCH_OBS, ManagerEnv, PitchingEnv

OBSERVATION_NAMES = ManagerEnv.observation_names()
ACTION_NAMES = ManagerEnv.action_names()
PITCH_OBSERVATION_NAMES = PitchingEnv.observation_names()
PITCH_ACTION_NAMES = PitchingEnv.action_names()

__all__ = ["ManagerEnv", "PitchingEnv", "N_ACTIONS", "N_OBS", "N_PITCH_ACTIONS", "N_PITCH_OBS", "OBSERVATION_NAMES",
           "ACTION_NAMES", "PITCH_OBSERVATION_NAMES", "PITCH_ACTION_NAMES", "make", "make_pitching"]


def make(**kwargs):
    """The Gymnasium environment (``BaseballManagerEnv``); keyword arguments go to it."""
    from .env import BaseballManagerEnv

    return BaseballManagerEnv(**kwargs)


def make_pitching(**kwargs):
    """The Gymnasium pitching environment (``BaseballPitchingEnv``); keyword arguments go to it."""
    from .env import BaseballPitchingEnv

    return BaseballPitchingEnv(**kwargs)


try:  # Register with Gymnasium when it is installed.
    import gymnasium as _gym

    if "BaseballManager-v0" not in _gym.registry:
        _gym.register(id="BaseballManager-v0", entry_point="bm_gym.env:BaseballManagerEnv")
    if "BaseballPitching-v0" not in _gym.registry:
        _gym.register(id="BaseballPitching-v0", entry_point="bm_gym.env:BaseballPitchingEnv")
except ImportError:  # pragma: no cover - gymnasium is optional
    pass
