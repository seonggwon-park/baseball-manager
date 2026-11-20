"""Gymnasium wrapper of the engine binding."""

from __future__ import annotations

from typing import Any

import gymnasium as gym
import numpy as np
from gymnasium import spaces

from ._core import N_ACTIONS, N_OBS, ManagerEnv

OBS_BOUND = 20.0


class BaseballManagerEnv(gym.Env):
    """Manage a club's offence pitch by pitch; an episode is one game.

    Actions: ``approach * 3 + steal`` with approach in (hit, sac_bunt, bunt_for_hit,
    hit_and_run, take) and steal in (auto, hold, steal); ``ACTION_NAMES`` lists them.
    Actions that make no sense in the situation (a steal order with nobody to send, a
    sacrifice with the bases empty) are played as the nearest sensible one; the mask of the
    sensible ones is in ``info["action_mask"]`` and ``action_masks()`` (the convention of
    sb3-contrib's MaskablePPO).

    Reward ``"win_expectancy"`` (default): the change of the club's win expectancy over the
    step, ending at 1 for a win and 0 for a loss. ``"win"``: 1 at the end of a won game.
    """

    metadata = {"render_modes": []}

    def __init__(self, reward: str = "win_expectancy", side: int | None = None, league_seed: int = 2025,
                 opponent: str = "mlb", own: str = "mlb", seed: int | None = None):
        super().__init__()
        self._env = ManagerEnv(seed=0 if seed is None else int(seed), reward=reward, side=side,
                               league_seed=league_seed, opponent=opponent, own=own)
        # The features are of order one; the bounds only rule out nonsense (values are clipped to them).
        self.observation_space = spaces.Box(low=-OBS_BOUND, high=OBS_BOUND, shape=(N_OBS,), dtype=np.float32)
        self.action_space = spaces.Discrete(N_ACTIONS)

    def _out(self, obs: list[float], info: dict[str, Any]) -> tuple[np.ndarray, dict[str, Any]]:
        info["action_mask"] = np.asarray(self._env.action_mask(), dtype=bool)
        return np.clip(np.asarray(obs, dtype=np.float32), -OBS_BOUND, OBS_BOUND), info

    def reset(self, *, seed: int | None = None, options: dict[str, Any] | None = None):
        super().reset(seed=seed)
        obs, info = self._env.reset(seed)
        return self._out(obs, info)

    def step(self, action):
        obs, reward, terminated, truncated, info = self._env.step(int(action))
        obs, info = self._out(obs, info)
        return obs, float(reward), bool(terminated), bool(truncated), info

    def action_masks(self) -> np.ndarray:
        return np.asarray(self._env.action_mask(), dtype=bool)

    @property
    def engine(self) -> ManagerEnv:
        """The binding underneath (``copy()`` it to try an action several ways)."""
        return self._env
